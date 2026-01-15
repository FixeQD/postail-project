use crate::db::message_bodies;
use crate::error::DBError;
use crate::security::SecurityManager;
use rusqlite::{params, Connection, Result as SqlResult};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
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
    conn.pragma_update(None, "mmap_size", "268435456")?;
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
                    INSERT INTO messages_fts(rowid, subject, from_addr, snippet)
                    VALUES (NEW.id, NEW.subject, NEW.from_addr, NEW.snippet);
                 END",
                [],
            )?;

            conn.execute(
                "CREATE TRIGGER IF NOT EXISTS messages_fts_update
                 AFTER UPDATE OF subject, from_addr, snippet ON messages BEGIN
                    UPDATE messages_fts
                    SET subject = NEW.subject, from_addr = NEW.from_addr, snippet = NEW.snippet
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
            println!(
                "Running migration {}: {}",
                migration.version, migration.name
            );
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
    security: &SecurityManager,
    passphrase: Option<String>,
) -> Result<PathBuf, DBError> {
    use zip::{write::FileOptions, ZipWriter};

    let temp_dir = env::temp_dir().join(format!("postail_backup_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)?;
    let backup_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail_backup.zip");

    let partial_db = temp_dir.join("postail.db");
    {
        let backup_conn = Connection::open(&partial_db)?;
        backup_conn.execute("CREATE TABLE accounts AS SELECT * FROM accounts", [])?;
        backup_conn.execute("CREATE TABLE mailboxes AS SELECT * FROM mailboxes", [])?;
        backup_conn.execute("CREATE TABLE messages AS SELECT * FROM messages", [])?;
        backup_conn.execute(
            "CREATE TABLE message_bodies AS SELECT * FROM message_bodies",
            [],
        )?;
    }

    let creds_dir = temp_dir.join("creds");
    fs::create_dir_all(&creds_dir)?;

    let mut stmt = conn.prepare("SELECT id, creds_blob_path FROM accounts")?;
    let account_creds = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(DBError::Sqlite)?;

    for (account_id, creds_path) in account_creds.flatten() {
        let encrypted_creds = fs::read(&creds_path).map_err(DBError::Io)?;
        let backup_encrypted = if let Some(ref pass) = passphrase {
            security
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

    let file = fs::File::create(&backup_path).map_err(DBError::Io)?;
    let mut zip = ZipWriter::new(file);
    let options: zip::write::FileOptions<()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in fs::read_dir(&temp_dir).map_err(DBError::Io)? {
        let entry = entry.map_err(DBError::Io)?;
        let path = entry.path();
        if path.is_file() && path != backup_path {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| DBError::Sqlite(rusqlite::Error::InvalidQuery))?;
            zip.start_file(name, options).map_err(|e| {
                DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
            let contents = fs::read(&path).map_err(DBError::Io)?;
            zip.write_all(&contents).map_err(|e| {
                DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
        }
    }

    let creds_entries = fs::read_dir(&creds_dir).map_err(DBError::Io)?;
    for entry in creds_entries {
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

    let backup_db = temp_dir.join("postail.db");
    let _backup_conn = Connection::open(&backup_db)?;

    conn.execute("DROP TABLE IF EXISTS messages_fts", [])?;
    conn.execute("DROP TABLE IF EXISTS message_bodies", [])?;
    conn.execute("DROP TABLE IF EXISTS messages", [])?;
    conn.execute("DROP TABLE IF EXISTS mailboxes", [])?;
    conn.execute("DROP TABLE IF EXISTS accounts", [])?;

    conn.execute(
        "ATTACH DATABASE ? AS backup",
        params![backup_db.to_string_lossy()],
    )?;

    conn.execute("CREATE TABLE accounts AS SELECT * FROM backup.accounts", [])?;
    conn.execute(
        "CREATE TABLE mailboxes AS SELECT * FROM backup.mailboxes",
        [],
    )?;
    conn.execute("CREATE TABLE messages AS SELECT * FROM backup.messages", [])?;
    conn.execute(
        "CREATE TABLE message_bodies AS SELECT * FROM backup.message_bodies",
        [],
    )?;

    conn.execute("DETACH DATABASE backup", [])?;

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
    conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])?;
    conn.execute("VACUUM", [])?;
    conn.execute("ANALYZE", [])?;
    Ok(())
}
