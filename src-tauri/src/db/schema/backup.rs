use crate::db::compose::outbox::cleanup_old_sent_messages;
use crate::db::mail::message_bodies;
use crate::error::DBError;
use crate::security::SecurityManager;
use rusqlite::{Connection, Result as SqlResult, params};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing;
use uuid::Uuid;

pub fn add_performance_indexes(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_account_mailbox
         ON messages(account_id, mailbox)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_uid
         ON messages(account_id, mailbox, uid)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_internal_date
         ON messages(internal_date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_outbox_status_retry
         ON outbox(status, next_retry)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_attachments_message
         ON attachments(message_table_id)",
        [],
    )?;

    Ok(())
}

pub fn configure_pragma(conn: &Connection) -> SqlResult<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "cache_size", "-64000")?;
    conn.pragma_update(None, "temp_store", "memory")?;
    Ok(())
}

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub up: fn(&Connection) -> SqlResult<()>,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "Initial schema",
        up: |conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            super::create_tables(conn).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(())
        },
    },
    Migration {
        version: 2,
        name: "Add message_bodies table",
        up: |conn| {
            message_bodies::create_message_bodies_table(conn)?;
            Ok(())
        },
    },
    Migration {
        version: 3,
        name: "Add performance indexes and pragmas",
        up: |conn| {
            add_performance_indexes(conn)?;
            configure_pragma(conn)?;
            Ok(())
        },
    },
    Migration {
        version: 4,
        name: "Add FTS triggers",
        up: |conn| {
            conn.execute(
                "CREATE TRIGGER IF NOT EXISTS messages_fts_insert
                 AFTER INSERT ON messages BEGIN
                    INSERT INTO messages_fts(rowid, subject, from_addr, to_json, snippet)
                    VALUES (NEW.id, NEW.subject, NEW.from_addr, NEW.to_json, NEW.snippet);
                 END",
                [],
            )?;

            conn.execute(
                "CREATE TRIGGER IF NOT EXISTS messages_fts_update
                 AFTER UPDATE ON messages BEGIN
                    UPDATE messages_fts
                    SET subject = NEW.subject, from_addr = NEW.from_addr, to_json = NEW.to_json, snippet = NEW.snippet
                    WHERE rowid = NEW.id;
                 END",
                [],
            )?;

            conn.execute(
                "CREATE TRIGGER IF NOT EXISTS messages_fts_delete
                 AFTER DELETE ON messages BEGIN
                    DELETE FROM messages_fts WHERE rowid = OLD.id;
                 END",
                [],
            )?;

            Ok(())
        },
    },
    Migration {
        version: 5,
        name: "Add role column to mailboxes",
        up: |conn| {
            conn.execute(
                "ALTER TABLE mailboxes ADD COLUMN role TEXT NOT NULL DEFAULT 'other'",
                [],
            )?;
            Ok(())
        },
    },
];

pub fn run_migrations(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, name TEXT)",
        [],
    )?;

    let current_version: Option<i32> = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .ok();

    for migration in MIGRATIONS {
        if current_version.unwrap_or(0) < migration.version {
            tracing::info!(target: "postail", "Running migration {}: {}", migration.version, migration.name);
            (migration.up)(conn)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, name) VALUES (?, ?)",
                params![migration.version, migration.name],
            )?;
        }
    }

    Ok(())
}

pub fn export_backup(
    conn: &Connection,
    secutity: &SecurityManager,
    passphrase: Option<String>,
) -> Result<PathBuf, DBError> {
    use zip::{ZipWriter, write::FileOptions};

    let temp_dir = env::temp_dir().join(format!("postail_backup_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)?;
    let backup_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail_backup.zip");

    // Export full DB to plaintext via sqlcipher_export (handles SQLCipher encryption)
    let db_path = temp_dir.join("postail.db");
    {
        let path_str = db_path.to_string_lossy();
        conn.execute_batch(&format!(
            "ATTACH DATABASE '{path_str}' AS _export KEY '';
             SELECT sqlcipher_export('_export');
             DETACH DATABASE _export;"
        ))?;
    }

    // Copy encrypted credential blobs
    let creds_dir = temp_dir.join("creds");
    fs::create_dir_all(&creds_dir)?;

    let mut stmt = conn.prepare("SELECT id, creds_blob_path FROM accounts")?;
    let account_creds = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(DBError::Sqlite)?;

    for (account_id, creds_path) in account_creds.flatten() {
        let resolved = crate::db::resolve_creds_path(&creds_path);
        let encrypted_creds = fs::read(&resolved).map_err(DBError::Io)?;
        let backup_encrypted = if let Some(ref pass) = passphrase {
            secutity
                .encrypt_with_passphrase(&encrypted_creds, pass)
                .map_err(DBError::Security)?
        } else {
            encrypted_creds
        };
        fs::write(
            creds_dir.join(format!("{}.enc", account_id)),
            backup_encrypted,
        )
        .map_err(DBError::Io)?;
    }

    // Pack everything into a zip
    let file = fs::File::create(&backup_path).map_err(DBError::Io)?;
    let mut zip = ZipWriter::new(file);
    let options: zip::write::FileOptions<()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("postail.db", options)
        .map_err(|e| DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
    let db_bytes = fs::read(&db_path).map_err(DBError::Io)?;
    zip.write_all(&db_bytes)
        .map_err(|e| DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

    for entry in fs::read_dir(&creds_dir).map_err(DBError::Io)? {
        let entry = entry.map_err(DBError::Io)?;
        let path = entry.path();
        if path.is_file() {
            let name = format!(
                "creds/{}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| DBError::Sqlite(rusqlite::Error::InvalidQuery))?
            );
            zip.start_file(&name, options).map_err(|e| {
                DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
            let contents = fs::read(&path).map_err(DBError::Io)?;
            zip.write_all(&contents).map_err(|e| {
                DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
        }
    }

    zip.finish()
        .map_err(|e| DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

    let _ = fs::remove_dir_all(temp_dir);
    Ok(backup_path)
}

pub fn import_backup(
    conn: &Connection,
    security: &SecurityManager,
    backup_path: &PathBuf,
    passphrase: Option<String>,
) -> Result<(), DBError> {
    use zip::ZipArchive;

    let file = fs::File::open(backup_path).map_err(DBError::Io)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

    let temp_dir = env::temp_dir().join(format!("postail_restore_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)?;
    archive
        .extract(&temp_dir)
        .map_err(|e| DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

    // Restore: attach plaintext backup into the encrypted conn, rebuild all tables
    let backup_db = temp_dir.join("postail.db");
    {
        conn.execute(
            "ATTACH DATABASE ? AS _import KEY ''",
            params![backup_db.to_string_lossy()],
        )?;

        conn.execute_batch(
            "DROP TABLE IF EXISTS messages_fts;
             DROP TABLE IF EXISTS messages_fts_data;
             DROP TABLE IF EXISTS messages_fts_idx;
             DROP TABLE IF EXISTS messages_fts_docsize;
             DROP TABLE IF EXISTS messages_fts_config;
             DROP TABLE IF EXISTS contacts_fts;
             DROP TABLE IF EXISTS contacts_fts_data;
             DROP TABLE IF EXISTS contacts_fts_idx;
             DROP TABLE IF EXISTS contacts_fts_docsize;
             DROP TABLE IF EXISTS contacts_fts_config;
             DROP TABLE IF EXISTS message_bodies;
             DROP TABLE IF EXISTS attachments;
             DROP TABLE IF EXISTS flag_sync_queue;
             DROP TABLE IF EXISTS drafts;
             DROP TABLE IF EXISTS messages;
             DROP TABLE IF EXISTS mailboxes;
             DROP TABLE IF EXISTS accounts;
             DROP TABLE IF EXISTS settings;
             DROP TABLE IF EXISTS contacts;
             DROP TABLE IF EXISTS outbox;
             DROP TABLE IF EXISTS schema_migrations;
             DROP TABLE IF EXISTS schema_versions;",
        )?;

        // Copy each table that exists in the backup
        for table in &[
            "accounts",
            "mailboxes",
            "messages",
            "message_bodies",
            "attachments",
            "flag_sync_queue",
            "drafts",
            "settings",
            "contacts",
            "outbox",
            "schema_migrations",
            "schema_versions",
        ] {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM _import.sqlite_master WHERE type='table' AND name=?",
                    params![table],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap_or(0)
                > 0;

            if exists {
                conn.execute_batch(&format!(
                    "CREATE TABLE {table} AS SELECT * FROM _import.{table};"
                ))?;
            }
        }

        conn.execute_batch("DETACH DATABASE _import;")?;
    }

    if temp_dir.join("creds").exists() {
        let creds_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("postail")
            .join("creds");
        fs::create_dir_all(&creds_dir).map_err(DBError::Io)?;

        let backup_creds = temp_dir.join("creds");
        for entry in fs::read_dir(&backup_creds).map_err(DBError::Io)? {
            let entry = entry.map_err(DBError::Io)?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "enc") {
                let backup_encrypted = fs::read(&path).map_err(DBError::Io)?;
                let decrypted = if let Some(ref pass) = passphrase {
                    security
                        .decrypt_with_passphrase(&backup_encrypted, pass)
                        .map_err(DBError::Security)?
                } else {
                    security
                        .decrypt(&backup_encrypted)
                        .map_err(DBError::Security)?
                };
                let reencrypted = security.encrypt(&decrypted).map_err(DBError::Security)?;
                let dest_path = creds_dir.join(
                    path.file_name()
                        .ok_or_else(|| DBError::Sqlite(rusqlite::Error::InvalidQuery))?,
                );
                fs::write(&dest_path, reencrypted).map_err(DBError::Io)?;
            }
        }
    }

    run_migrations(conn)?;

    run_maintenance(conn)?;

    let _ = fs::remove_dir_all(temp_dir);
    Ok(())
}

pub fn run_maintenance(conn: &Connection) -> Result<(), DBError> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    conn.execute("VACUUM", [])?;
    conn.execute("ANALYZE", [])?;

    if let Ok(count) = cleanup_old_sent_messages(conn, 30) {
        if count > 0 {
            tracing::info!(target: "postail", "[DB] Cleaned up {} old sent messages (>30 days)", count);
        }
    }

    Ok(())
}
