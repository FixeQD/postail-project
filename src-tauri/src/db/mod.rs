pub mod accounts;
pub mod attachments;
pub mod backup;
pub mod contacts;
pub mod drafts;
pub mod imap;
pub mod mailbox;
pub mod message_bodies;
pub mod messages;
pub mod migration;
pub mod migrations;
pub mod outbox;
pub mod outbox_db;
pub mod search;
pub mod settings;
pub mod sql_helpers;
pub mod tables;

use std::fs;

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub use crate::db::accounts::*;
pub use crate::db::backup::{export_backup, import_backup, run_maintenance};
pub use crate::db::contacts::*;
pub use crate::db::drafts::*;
pub use crate::db::imap::*;
pub use crate::db::mailbox::{fetch_mailboxes, upsert_mailbox};
pub use crate::db::message_bodies::*;
pub use crate::db::messages::{
    batch_insert_messages, fetch_headers, fetch_message_full, mark_read, move_to_trash,
    upsert_message, MessageBatchItem, MessageUpsertData, DEFAULT_BATCH_SIZE,
};
pub use crate::db::migration::run_encryption_migration_if_needed;
pub use crate::db::migrations::{get_db_version, run_migrations};
pub use crate::db::outbox::*;
pub use crate::db::outbox_db::{enqueue_message, list_outbox};
pub use crate::db::search::*;
pub use crate::db::sql_helpers::*;
pub use crate::db::tables::*;
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
pub struct AccountInput {
    pub name: String,
    pub email: String,
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

pub fn init_db() -> Result<Connection, DBError> {
    let key = crate::security::DbEncryption::get_hex_key();
    if key.is_empty() {
        return Err(DBError::Security(
            crate::error::SecurityError::KeyDerivation(
                "Failed to get database encryption key".to_string(),
            ),
        ));
    }
    init_db_with_key(&key)
}

fn apply_sqlcipher_key(conn: &Connection, hex_key: &str) -> Result<(), DBError> {
    tracing::info!(target: "postail", "[DB] Setting PRAGMA key...");

    let key_stmt = format!("PRAGMA key = \"x'{hex_key}'\"");
    execute_pragma(conn, &key_stmt)?;
    tracing::info!(target: "postail", "[DB] Key set, setting journal_mode...");

    execute_pragma(conn, "PRAGMA journal_mode = WAL")?;
    tracing::info!(target: "postail", "[DB] Setting synchronous...");

    execute_pragma(conn, "PRAGMA synchronous = NORMAL")?;
    tracing::info!(target: "postail", "[DB] Setting cache_size...");

    execute_pragma(conn, "PRAGMA cache_size = -64000")?;
    tracing::info!(target: "postail", "[DB] Setting mmap_size...");

    execute_pragma(conn, "PRAGMA mmap_size = 268435456")?;
    tracing::info!(target: "postail", "[DB] All pragmas set successfully!");

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
    let dir = crate::utils::config::get_data_dir()
        .join("creds");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.enc", id));
    fs::write(&path, data)?;
    Ok(path.to_string_lossy().to_string())
}
