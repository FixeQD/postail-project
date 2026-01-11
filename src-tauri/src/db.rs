use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
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

pub fn init_db() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?; // in-memory; later file-based
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
