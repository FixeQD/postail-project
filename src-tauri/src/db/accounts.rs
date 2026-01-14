use rusqlite::params;
use crate::error::DBError;
use crate::security::SecurityManager;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqlResult};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub fn add_account(
    conn: &Connection,
    input: crate::db::AccountInput,
    security: &SecurityManager,
) -> Result<crate::db::AccountMeta, DBError> {
    let id = Uuid::new_v4().to_string();
    let email = input.email.clone();
    let provider_type = match input.auth_type.as_str() {
        "oauth2" => "gmail".to_string(),
        _ => "generic".to_string(),
    };
    let creds_json = serde_json::to_string(&input.credentials).unwrap();
    let encrypted = security
        .encrypt(creds_json.as_bytes())
        .map_err(DBError::Security)?;
    let creds_path = super::save_creds_blob(&id, &encrypted)?;
    let encryption_mode = "aes-gcm".to_string();
    let created_at = Utc::now();

    crate::db::sql_helpers::insert_or_replace_into(
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
            &(created_at.timestamp() as i64).to_string(),
        ],
    )?;

    Ok(crate::db::AccountMeta {
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

pub fn list_accounts(conn: &Connection) -> Result<Vec<crate::db::AccountMeta>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, email, provider_type, auth_type, imap_host, imap_port, imap_tls, smtp_host, smtp_port, smtp_tls, encryption_mode, created_at FROM accounts"
    )?;

    let accounts_iter = stmt.query_map([], |row| {
        Ok(crate::db::AccountMeta {
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

    let accounts: Result<Vec<crate::db::AccountMeta>, _> = accounts_iter.collect();
    accounts.map_err(DBError::Sqlite)
}

pub fn remove_account(conn: &Connection, id: &str) -> Result<(), DBError> {
    let mut stmt = conn.prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")?;
    let creds_path: Option<String> = stmt.query_row(params![id], |row| row.get(0)).ok();
    if let Some(path) = creds_path {
        let _ = fs::remove_file(path);
    }
    crate::db::sql_helpers::delete_where::<String>(conn, "accounts", "id = ?", &[&id.to_string()])?;
    Ok(())
}
