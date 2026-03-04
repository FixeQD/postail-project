pub mod account;
pub mod compose;
pub mod mail;
pub mod schema;
pub mod search;
pub mod sql_helpers;

use std::fs;

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub use crate::db::account::*;
pub use crate::db::compose::*;
pub use crate::db::mail::*;
pub use crate::db::schema::*;
pub use crate::db::search::*;
pub use crate::db::sql_helpers::*;
use crate::error::DBError;

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
    pub expires_at: i64,
    pub auth_type: String,
    pub provider_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    Password(PasswordCredentials),
    OAuth(OAuthCredentials),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualServerConfig {
    pub account_name: String,
    pub email: String,
    pub use_separate_username: bool,
    pub username: Option<String>,
    pub password: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: bool,
}

impl Default for ManualServerConfig {
    fn default() -> Self {
        Self {
            account_name: String::new(),
            email: String::new(),
            use_separate_username: false,
            username: None,
            password: String::new(),
            imap_host: String::new(),
            imap_port: 993,
            imap_tls: true,
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_tls: true,
        }
    }
}

impl ManualServerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.account_name.trim().is_empty() {
            return Err("Account name is required".to_string());
        }
        if self.email.trim().is_empty() {
            return Err("Email is required".to_string());
        }
        if self.use_separate_username
            && self
                .username
                .as_ref()
                .map(|u| u.trim().is_empty())
                .unwrap_or(true)
        {
            return Err("Username is required when using separate username".to_string());
        }
        if self.password.is_empty() {
            return Err("Password is required".to_string());
        }
        if self.imap_host.trim().is_empty() {
            return Err("IMAP host is required".to_string());
        }
        if self.imap_port == 0 {
            return Err("IMAP port must be between 1 and 65535".to_string());
        }
        if self.smtp_host.trim().is_empty() {
            return Err("SMTP host is required".to_string());
        }
        if self.smtp_port == 0 {
            return Err("SMTP port must be between 1 and 65535".to_string());
        }
        if !self.imap_tls {
            return Err("Non-TLS IMAP connections are not currently su// I'm f**kin done with this 😭pported. Please enable TLS.".to_string());
        }
        Ok(())
    }

    pub fn get_username(&self) -> &str {
        if self.use_separate_username {
            self.username.as_deref().unwrap_or(&self.email)
        } else {
            &self.email
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInput {
    pub name: String,
    pub email: String,
    pub provider_type: String,
    pub auth_type: String,
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
    pub display_name: String,
    pub role: String, // "inbox", "sent", "trash", "drafts", "archive", "other"
    pub uid_validity: Option<u32>,
    pub highest_modseq: Option<i64>,
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
    pub cc: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
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

pub async fn init_db() -> Result<(), DBError> {
    // Acquire SECURITY briefly to extract the master key, then drop the guard.
    let master_key = {
        let security = crate::globals::SECURITY.lock().await;
        security.get_master_key_raw()
    };
    let key = crate::security::DbEncryption::get_hex_key(&master_key);
    if key.is_empty() {
        return Err(DBError::Security(
            crate::error::SecurityError::KeyDerivation(
                "Failed to get database encryption key".to_string(),
            ),
        ));
    }
    let conn = init_db_with_key(&key)?;

    // Save to global DB_CONN
    let mut db_guard = crate::globals::DB_CONN.lock().await;
    *db_guard = Some(conn);

    Ok(())
}

fn apply_sqlcipher_key(conn: &Connection, hex_key: &str) -> Result<(), DBError> {
    let pragmas = [
        format!("PRAGMA key = \"x'{hex_key}'\""),
        "PRAGMA journal_mode = WAL".to_string(),
        "PRAGMA synchronous = NORMAL".to_string(),
        "PRAGMA cache_size = -64000".to_string(),
    ];

    for pragma in pragmas {
        execute_pragma(conn, &pragma)?;
    }

    tracing::info!(target: "postail", "[DB] All database pragmas applied");
    Ok(())
}

fn execute_pragma(conn: &Connection, pragma: &str) -> Result<(), DBError> {
    match conn.execute(pragma, ()) {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::ExecuteReturnedResults) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn init_db_with_key(hex_key: &str) -> Result<Connection, DBError> {
    tracing::info!(target: "postail", "[DB] Creating data directory...");
    let data_dir = crate::utils::config::get_data_dir();
    fs::create_dir_all(&data_dir).map_err(DBError::Io)?;
    let db_path = data_dir.join("postail.db");
    tracing::info!(target: "postail", "[DB] Opening database at {:?}", db_path);

    let conn = Connection::open(&db_path)?;
    tracing::info!(target: "postail", "[DB] Database opened, applying key...");

    apply_sqlcipher_key(&conn, hex_key)?;
    tracing::info!(target: "postail", "[DB] Key applied, creating tables...");

    tables::create_tables(&conn)?;
    tracing::info!(target: "postail", "[DB] Tables created, creating indexes...");

    tables::create_indexes(&conn)?;
    tracing::info!(target: "postail", "[DB] Indexes created, creating FTS triggers...");

    tables::create_fts_triggers(&conn)?;
    tracing::info!(target: "postail", "[DB] FTS triggers created, running migrations...");

    run_migrations(&conn)?;
    tracing::info!(target: "postail", "[DB] Migrations complete!");

    Ok(conn)
}

pub fn connect_db_with_key(hex_key: &str) -> Result<Connection, DBError> {
    let data_dir = crate::utils::config::get_data_dir();
    let db_path = data_dir.join("postail.db");

    let conn = Connection::open(&db_path)?;
    apply_sqlcipher_key(&conn, hex_key)?;
    Ok(conn)
}

fn save_creds_blob(id: &str, data: &[u8]) -> Result<String, DBError> {
    let dir = crate::utils::config::get_data_dir().join("creds");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.enc", id));
    fs::write(&path, data)?;
    // Store only the relative path so it survives data-dir migrations
    Ok(format!("creds/{}.enc", id))
}

/// Resolve a creds_blob_path from the DB to an absolute filesystem path
pub fn resolve_creds_path(stored: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(stored);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        crate::utils::config::get_data_dir().join(stored)
    }
}
