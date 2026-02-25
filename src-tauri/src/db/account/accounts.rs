use std::fs;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::db::messages::safe_timestamp_from_utc;
use crate::db::save_creds_blob;
use crate::db::sql_helpers::{delete_where, insert_or_replace_into};
use crate::db::AccountInput;
use crate::db::AccountMeta;
use crate::error::DBError;
use crate::security::SecurityManager;

fn timestamp_from_utc_or_default(seconds: i64) -> DateTime<Utc> {
    safe_timestamp_from_utc(seconds).unwrap_or_else(Utc::now)
}

pub fn add_account(
    conn: &Connection,
    input: AccountInput,
    security: &SecurityManager,
) -> Result<AccountMeta, DBError> {
    let id = Uuid::new_v4().to_string();
    let email = input.email.clone();
    let provider_type = input.provider_type.clone();
    let creds_json = serde_json::to_string(&input.credentials).map_err(DBError::Json)?;
    let encrypted = security
        .encrypt(creds_json.as_bytes())
        .map_err(DBError::Security)?;
    let creds_path = save_creds_blob(&id, &encrypted)?;
    let encryption_mode = "aes-gcm".to_string();
    let created_at = Utc::now();

    insert_or_replace_into(
        conn,
        "accounts",
        &[
            "id",
            "name",
            "email",
            "provider_type",
            "auth_type",
            "imap_host",
            "imap_port",
            "imap_tls",
            "smtp_host",
            "smtp_port",
            "smtp_tls",
            "creds_blob_path",
            "encryption_mode",
            "created_at",
        ],
        &[
            &id,
            &input.name,
            &email,
            &provider_type,
            &input.auth_type,
            &input.imap_config.host,
            &(input.imap_config.port as i64).to_string(),
            &(input.imap_config.tls as i64).to_string(),
            &input.smtp_config.host,
            &(input.smtp_config.port as i64).to_string(),
            &(input.smtp_config.tls as i64).to_string(),
            &creds_path,
            &encryption_mode,
            &created_at.timestamp().to_string(),
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
        "SELECT id, name, email, provider_type, auth_type, imap_host, imap_port, imap_tls, smtp_host, smtp_port, smtp_tls, encryption_mode, created_at FROM accounts"
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
            created_at: timestamp_from_utc_or_default(row.get::<_, i64>(12)?),
        })
    })?;

    let accounts: Result<Vec<AccountMeta>, _> = accounts_iter.collect();
    accounts.map_err(DBError::Sqlite)
}

pub fn remove_account(conn: &Connection, id: &str) -> Result<(), DBError> {
    let mut stmt = conn.prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")?;
    let creds_path: Option<String> = stmt.query_row(params![id], |row| row.get(0)).ok();
    if let Some(path) = creds_path {
        let _ = fs::remove_file(path);
    }
    delete_where::<String>(conn, "accounts", "id = ?", &[&id.to_string()])?;
    Ok(())
}

pub fn get_account_email(conn: &Connection, id: &str) -> Result<Option<String>, DBError> {
    let mut stmt = conn.prepare("SELECT email FROM accounts WHERE id = ?")?;
    let email: Option<String> = stmt.query_row(params![id], |row| row.get(0)).ok();
    Ok(email)
}
