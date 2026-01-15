use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::DBError;
use crate::security::db_encryption::DbEncryption;

pub fn check_db_encrypted(conn: &Connection) -> bool {
    match conn.query_row("PRAGMA cipher_version", [], |_| Ok(())) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn check_db_has_tables(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='accounts' LIMIT 1",
        [],
        |_| Ok(()),
    )
    .is_ok()
}

pub fn migrate_unencrypted_db(db_path: &Path, encryption: &DbEncryption) -> Result<(), DBError> {
    if !db_path.exists() {
        return Ok(());
    }

    let backup_path = db_path.with_extension("bak");
    let encrypted_path = db_path.with_extension("encrypted.db");

    eprintln!("[DB] Starting database encryption migration...");
    eprintln!("[DB] Creating backup at: {}", backup_path.display());

    if let Err(e) = fs::copy(db_path, &backup_path) {
        return Err(DBError::Io(e));
    }

    eprintln!("[DB] Exporting database to encrypted format...");

    {
        let conn = Connection::open(&backup_path)?;
        let key = encryption.hex_key();

        let sql = format!(
            r#"
            ATTACH DATABASE '{}' AS encrypted KEY "x'{}'";
            SELECT sqlcipher_export('encrypted');
            DETACH DATABASE encrypted;
        "#,
            encrypted_path.display(),
            key
        );

        conn.execute(&sql, ())
            .map_err(|e| DBError::Migration(e.to_string()))?;
    }

    if !encrypted_path.exists() {
        return Err(DBError::Migration(
            "Encrypted database file was not created".to_string(),
        ));
    }

    eprintln!("[DB] Replacing original database with encrypted version...");

    fs::rename(encrypted_path, db_path).map_err(|e| DBError::Io(e))?;

    eprintln!("[DB] Verifying encrypted database...");

    {
        let conn = Connection::open(db_path)?;
        conn.execute(&format!("PRAGMA key = \"x'{}'\"", encryption.hex_key()), ())?;

        let integrity: i32 = conn
            .query_row("PRAGMA cipher_integrity_check", [], |row| row.get(0))
            .map_err(|e| DBError::Migration(e.to_string()))?;

        if integrity != 0 {
            return Err(DBError::Migration(
                "Encrypted database integrity check failed".to_string(),
            ));
        }

        if !check_db_has_tables(&conn) {
            return Err(DBError::Migration(
                "Tables not found after migration".to_string(),
            ));
        }
    }

    eprintln!("[DB] Database encryption migration completed successfully!");
    eprintln!("[DB] Backup saved at: {}", backup_path.display());

    Ok(())
}

pub fn get_db_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("postail");
    data_dir.join("postail.db")
}

pub fn run_encryption_migration_if_needed() -> Result<(), DBError> {
    let db_path = get_db_path();

    if !db_path.exists() {
        return Ok(());
    }

    let conn = Connection::open(&db_path)?;

    if check_db_encrypted(&conn) {
        eprintln!("[DB] Database is already encrypted");
        return Ok(());
    }

    eprintln!("[DB] Database is not encrypted, starting migration...");

    match DbEncryption::global().as_ref() {
        Ok(encryption) => {
            migrate_unencrypted_db(&db_path, encryption)?;
            Ok(())
        }
        Err(e) => Err(DBError::Security(
            crate::error::SecurityError::KeyDerivation(format!(
                "Failed to initialize encryption: {}",
                e
            )),
        )),
    }
}
