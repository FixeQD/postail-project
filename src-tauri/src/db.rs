use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::DBError;
use crate::security::SecurityManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub tls: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub tls: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    Password(PasswordCredentials),
    OAuth(OAuthCredentials),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInput {
    pub name: String,
    pub email: String,
    pub auth_type: String, // "password" or "oauth2"
    pub imap_config: ImapConfig,
    pub smtp_config: SmtpConfig,
    pub credentials: Credentials,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountMeta {
    pub id: String,
    pub name: String,
    pub email: String,
    pub provider_type: String,
    pub auth_type: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: bool,
    pub encryption_mode: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mailbox {
    pub name: String,
    pub uid_validity: Option<u32>,
    pub highest_modseq: Option<u64>,
    pub last_synced_uid: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MailHeader {
    pub uid: u32,
    pub message_id: Option<String>,
    pub internal_date: DateTime<Utc>,
    pub subject: Option<String>,
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub flags: Vec<String>,
    pub snippet: Option<String>,
    pub has_attachments: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub part_id: String,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageFull {
    pub header: MailHeader,
    pub body_html_safe: String,
    pub body_plain: String,
    pub attachments: Vec<AttachmentMeta>,
    pub inline_images: Vec<AttachmentMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxItem {
    pub id: String,
    pub subject: Option<String>,
    pub recipient: String,
    pub status: String,
    pub error_log: Option<String>,
    pub attempts: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncStatusEnum {
    Idle,
    Syncing,
    Error(String),
}

pub fn init_db() -> SqlResult<Connection> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail");
    fs::create_dir_all(&data_dir).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let db_path = data_dir.join("postail.db");
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    create_tables(&conn)?;
    Ok(conn)
}

fn create_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            imap_host TEXT NOT NULL,
            imap_port INTEGER NOT NULL,
            imap_tls INTEGER NOT NULL,
            smtp_host TEXT NOT NULL,
            smtp_port INTEGER NOT NULL,
            smtp_tls INTEGER NOT NULL,
            creds_blob_path TEXT NOT NULL,
            encryption_mode TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS mailboxes (
            id INTEGER PRIMARY KEY,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            uid_validity INTEGER,
            highest_modseq INTEGER,
            last_synced_uid INTEGER,
            UNIQUE(account_id, name),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            account_id TEXT NOT NULL,
            mailbox TEXT NOT NULL,
            uid INTEGER NOT NULL,
            message_id TEXT,
            internal_date INTEGER NOT NULL,
            from_addr TEXT,
            to_json TEXT,
            subject TEXT,
            snippet TEXT,
            flags_json TEXT,
            cached_structure_json TEXT,
            UNIQUE(account_id, mailbox, uid),
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            subject, from_addr, snippet, body_plain,
            content='messages', content_rowid='id'
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS outbox (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            raw_eml_path TEXT NOT NULL,
            status TEXT NOT NULL,
            attempts INTEGER DEFAULT 0,
            last_error TEXT,
            created_at INTEGER NOT NULL,
            next_retry INTEGER,
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY,
            message_table_id INTEGER NOT NULL,
            part_id TEXT NOT NULL,
            filename TEXT,
            mime_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            cached_path TEXT,
            FOREIGN KEY(message_table_id) REFERENCES messages(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

pub fn add_account(
    conn: &Connection,
    input: AccountInput,
    security: &SecurityManager,
) -> Result<AccountMeta, DBError> {
    let id = Uuid::new_v4().to_string();
    let email = input.email.clone();
    let provider_type = match input.auth_type.as_str() {
        "oauth2" => {
            // Assume from input, "gmail" or "outlook"
            "gmail".to_string() // TODO: derive from provider
        }
        _ => "generic".to_string(),
    };
    let creds_json = serde_json::to_string(&input.credentials).unwrap();
    let encrypted = security
        .encrypt(creds_json.as_bytes())
        .map_err(|e| DBError::Security(e))?;
    let creds_path = save_creds_blob(&id, &encrypted)?;
    let encryption_mode = "aes-gcm".to_string(); // TODO: from security
    let created_at = Utc::now();

    conn.execute(
        "INSERT INTO accounts (id, name, email, provider_type, auth_type, imap_host, imap_port, imap_tls, smtp_host, smtp_port, smtp_tls, creds_blob_path, encryption_mode, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            id,
            input.name,
            email,
            provider_type,
            input.auth_type,
            input.imap_config.host,
            input.imap_config.port as i64,
            input.imap_config.tls as i64,
            input.smtp_config.host,
            input.smtp_config.port as i64,
            input.smtp_config.tls as i64,
            creds_path,
            encryption_mode,
            created_at.timestamp(),
        ],
    )?;

    Ok(AccountMeta {
        id,
        name: input.name,
        email,
        provider_type,
        auth_type: input.auth_type,
        imap_host: input.imap_config.host,
        imap_port: input.imap_config.port,
        imap_tls: input.imap_config.tls,
        smtp_host: input.smtp_config.host,
        smtp_port: input.smtp_config.port,
        smtp_tls: input.smtp_config.tls,
        encryption_mode,
        created_at,
    })
}

pub fn list_accounts(conn: &Connection) -> Result<Vec<AccountMeta>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, email, provider_type, auth_type, imap_host, imap_port, imap_tls, smtp_host, smtp_port, smtp_tls, encryption_mode, created_at FROM accounts",
    )?;
    let accounts_iter = stmt.query_map([], |row| {
        Ok(AccountMeta {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            provider_type: row.get(3)?,
            auth_type: row.get(4)?,
            imap_host: row.get(5)?,
            imap_port: row.get::<_, i64>(6)? as u16,
            imap_tls: row.get::<_, i64>(7)? != 0,
            smtp_host: row.get(8)?,
            smtp_port: row.get::<_, i64>(9)? as u16,
            smtp_tls: row.get::<_, i64>(10)? != 0,
            encryption_mode: row.get(11)?,
            created_at: DateTime::from_timestamp(row.get::<_, i64>(12)?, 0).unwrap(),
        })
    })?;
    let accounts: Result<Vec<AccountMeta>, _> = accounts_iter.collect();
    accounts.map_err(DBError::Sqlite)
}

pub fn remove_account(conn: &Connection, id: &str) -> Result<(), DBError> {
    // First, get creds_path and delete file
    let mut stmt = conn.prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")?;
    let creds_path: Option<String> = stmt.query_row([id], |row| row.get(0)).ok();
    if let Some(path) = creds_path {
        let _ = fs::remove_file(path);
    }
    conn.execute("DELETE FROM accounts WHERE id = ?", [id])?;
    Ok(())
}

fn save_creds_blob(id: &str, data: &[u8]) -> Result<String, DBError> {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("creds");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.enc", id));
    fs::write(&path, data)?;
    Ok(path.to_string_lossy().to_string())
}

// Mailbox functions
pub fn fetch_mailboxes(conn: &Connection, account_id: &str) -> Result<Vec<Mailbox>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT name, uid_validity, highest_modseq, last_synced_uid FROM mailboxes WHERE account_id = ?",
    )?;
    let mailboxes_iter = stmt.query_map([account_id], |row| {
        Ok(Mailbox {
            name: row.get(0)?,
            uid_validity: row.get(1)?,
            highest_modseq: row.get(2)?,
            last_synced_uid: row.get(3)?,
        })
    })?;
    let mailboxes: Result<Vec<Mailbox>, _> = mailboxes_iter.collect();
    mailboxes.map_err(DBError::Sqlite)
}

pub fn upsert_mailbox(
    conn: &Connection,
    account_id: &str,
    mailbox: &Mailbox,
) -> Result<(), DBError> {
    conn.execute(
        "INSERT OR REPLACE INTO mailboxes (account_id, name, uid_validity, highest_modseq, last_synced_uid)
         VALUES (?, ?, ?, ?, ?)",
        params![
            account_id,
            mailbox.name,
            mailbox.uid_validity,
            mailbox.highest_modseq,
            mailbox.last_synced_uid,
        ],
    )?;
    Ok(())
}

// Message functions
pub fn fetch_headers(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    anchor: Option<u32>,
    limit: u32,
) -> Result<Vec<MailHeader>, DBError> {
    let (query, params) = if let Some(anchor) = anchor {
        (
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, flags_json, snippet
             FROM messages WHERE account_id = ? AND mailbox = ? AND uid > ? ORDER BY uid DESC LIMIT ?",
            vec![account_id.to_string(), mailbox.to_string(), anchor.to_string(), limit.to_string()],
        )
    } else {
        (
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, flags_json, snippet
             FROM messages WHERE account_id = ? AND mailbox = ? ORDER BY uid DESC LIMIT ?",
            vec![account_id.to_string(), mailbox.to_string(), limit.to_string()],
        )
    };

    let mut stmt = conn.prepare(query)?;
    let headers_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        let to_json: Option<String> = row.get(5)?;
        let to: Vec<String> = to_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let flags_json: Option<String> = row.get(6)?;
        let flags: Vec<String> = flags_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        Ok(MailHeader {
            uid: row.get::<_, u32>(0)?,
            message_id: row.get(1)?,
            internal_date: DateTime::from_timestamp(row.get::<_, i64>(2)?, 0).unwrap(),
            subject: row.get(3)?,
            from: vec![row.get::<_, Option<String>>(4)?.unwrap_or_default()],
            to,
            flags,
            snippet: row.get(7)?,
            has_attachments: false, // TODO: check attachments table
        })
    })?;

    let headers: Result<Vec<MailHeader>, _> = headers_iter.collect();
    headers.map_err(DBError::Sqlite)
}

pub fn upsert_message(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    message_id: Option<&str>,
    internal_date: DateTime<Utc>,
    from: Option<&str>,
    to_json: Option<&str>,
    subject: Option<&str>,
    snippet: Option<&str>,
    flags_json: Option<&str>,
    structure_json: Option<&str>,
) -> Result<i64, DBError> {
    conn.execute(
        "INSERT OR REPLACE INTO messages (account_id, mailbox, uid, message_id, internal_date, from_addr, to_json, subject, snippet, flags_json, cached_structure_json)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            account_id,
            mailbox,
            uid,
            message_id,
            internal_date.timestamp(),
            from,
            to_json,
            subject,
            snippet,
            flags_json,
            structure_json,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn fetch_message_full(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<MessageFull>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT message_id, internal_date, subject, from_addr, to_json, flags_json, snippet FROM messages
         WHERE account_id = ? AND mailbox = ? AND uid = ?",
    )?;
    let header = stmt
        .query_row(params![account_id, mailbox, uid], |row| {
            let to_json: Option<String> = row.get(4)?;
            let to: Vec<String> = to_json
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            let flags_json: Option<String> = row.get(5)?;
            let flags: Vec<String> = flags_json
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            Ok(MailHeader {
                uid,
                message_id: row.get(0)?,
                internal_date: DateTime::from_timestamp(row.get::<_, i64>(1)?, 0).unwrap(),
                subject: row.get(2)?,
                from: vec![row.get::<_, Option<String>>(3)?.unwrap_or_default()],
                to,
                flags,
                snippet: row.get(6)?,
                has_attachments: false, // TODO
            })
        })
        .optional()?;

    if let Some(header) = header {
        // TODO: fetch body_html_safe, body_plain, attachments, inline_images
        Ok(Some(MessageFull {
            header,
            body_html_safe: String::new(),
            body_plain: String::new(),
            attachments: vec![],
            inline_images: vec![],
        }))
    } else {
        Ok(None)
    }
}

// Outbox functions
pub fn enqueue_message(
    conn: &Connection,
    account_id: &str,
    raw_eml_path: &str,
) -> Result<String, DBError> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO outbox (id, account_id, raw_eml_path, status, created_at)
         VALUES (?, ?, ?, 'PENDING', ?)",
        params![id, account_id, raw_eml_path, Utc::now().timestamp()],
    )?;
    Ok(id)
}

pub fn list_outbox(conn: &Connection, account_id: &str) -> Result<Vec<OutboxItem>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, raw_eml_path, status, attempts, last_error FROM outbox WHERE account_id = ?",
    )?;
    let items_iter = stmt.query_map([account_id], |row| {
        // TODO: extract subject and recipient from EML
        Ok(OutboxItem {
            id: row.get(0)?,
            subject: None,
            recipient: String::new(),
            status: row.get(2)?,
            error_log: row.get(4)?,
            attempts: row.get::<_, i64>(3)? as u32,
        })
    })?;
    let items: Result<Vec<OutboxItem>, _> = items_iter.collect();
    items.map_err(DBError::Sqlite)
}
