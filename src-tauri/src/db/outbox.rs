use crate::error::DBError;
use crate::security::SecurityManager;
use chrono::Utc;
use mailparse::{parse_mail, MailHeaderMap};
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use tracing;

pub fn extract_headers_from_eml(eml_path: &str) -> Result<(Option<String>, String), DBError> {
    let eml_bytes = fs::read(eml_path).map_err(DBError::Io)?;
    let mail = parse_mail(&eml_bytes)
        .map_err(|e| DBError::Sqlite(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;

    let subject = mail
        .get_headers()
        .get_first_header("Subject")
        .map(|s| s.get_value());

    let recipient = mail
        .get_headers()
        .get_first_header("To")
        .map(|s| s.get_value())
        .unwrap_or_default();

    Ok((subject, recipient))
}

pub fn update_outbox_status(
    conn: &Connection,
    outbox_id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), DBError> {
    conn.execute(
        "UPDATE outbox SET status = ?, last_error = ? WHERE id = ?",
        params![status, error.unwrap_or(""), outbox_id],
    )?;
    Ok(())
}

pub fn increment_outbox_attempts(
    conn: &Connection,
    outbox_id: &str,
    next_retry: i64,
) -> Result<u32, DBError> {
    conn.execute(
        "UPDATE outbox SET attempts = attempts + 1, next_retry = ? WHERE id = ?",
        params![next_retry, outbox_id],
    )?;

    conn.query_row(
        "SELECT attempts FROM outbox WHERE id = ?",
        params![outbox_id],
        |row| row.get::<_, i64>(0).map(|a| a as u32),
    )
    .map_err(DBError::Sqlite)
}

pub fn calculate_backoff(attempts: u32) -> i64 {
    const BACKOFF_SCHEDULE: [i64; 5] = [5, 30, 300, 900, 3600];
    let idx = std::cmp::min(attempts as usize, BACKOFF_SCHEDULE.len() - 1);
    Utc::now().timestamp() + BACKOFF_SCHEDULE[idx]
}

pub fn cleanup_old_sent_messages(conn: &Connection, days_old: u32) -> Result<usize, DBError> {
    let cutoff = Utc::now().timestamp() - (days_old as i64 * 86400);
    let mut stmt = conn.prepare(
        "SELECT raw_eml_path FROM outbox
         WHERE status = 'SENT' AND created_at < ?",
    )?;

    let paths: Vec<String> = stmt
        .query_map([cutoff], |row| row.get(0))
        .map_err(DBError::Sqlite)?
        .filter_map(|r| r.ok())
        .collect();

    for path in paths {
        let _ = fs::remove_file(path);
    }

    let deleted = conn.execute(
        "DELETE FROM outbox WHERE status = 'SENT' AND created_at < ?",
        params![cutoff],
    )?;
    Ok(deleted)
}

pub fn get_attachment_cache_path(message_table_id: i64, part_id: &str) -> PathBuf {
    let hash = format!("{}{}", message_table_id, part_id);
    let prefix = &hash[0..2];
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("attachments")
        .join(prefix)
        .join(format!("{}.enc", hash))
}

pub fn save_attachment(
    conn: &Connection,
    security: &SecurityManager,
    message_table_id: i64,
    part_id: &str,
    filename: Option<&str>,
    mime_type: &str,
    data: &[u8],
) -> Result<i64, DBError> {
    let cache_path = get_attachment_cache_path(message_table_id, part_id);
    fs::create_dir_all(cache_path.parent().unwrap()).map_err(DBError::Io)?;

    let encrypted = security.encrypt(data).map_err(DBError::Security)?;
    fs::write(&cache_path, encrypted).map_err(DBError::Io)?;

    conn.execute(
        "INSERT INTO attachments (message_table_id, part_id, filename, mime_type, size, cached_path)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            message_table_id,
            part_id,
            filename,
            mime_type,
            data.len() as i64,
            cache_path.to_string_lossy(),
        ],
    )?;

    if let Err(e) = super::messages::sync_message_attachments_flag(message_table_id, conn) {
        tracing::warn!(target: "postail", "[DB] Failed to sync attachments flag: {}", e);
    }

    Ok(conn.last_insert_rowid())
}

pub fn load_attachment(
    conn: &Connection,
    security: &SecurityManager,
    message_table_id: i64,
    part_id: &str,
) -> Result<(Vec<u8>, String), DBError> {
    let (cache_path, mime_type): (String, String) = conn.query_row(
        "SELECT cached_path, mime_type FROM attachments
         WHERE message_table_id = ? AND part_id = ?",
        params![message_table_id, part_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let encrypted = fs::read(&cache_path).map_err(DBError::Io)?;
    let decrypted = security.decrypt(&encrypted).map_err(DBError::Security)?;
    Ok((decrypted, mime_type))
}

fn enforce_attachment_cache_limit_rec(
    dir: &PathBuf,
    files: &mut Vec<(PathBuf, u64)>,
) -> Result<u64, DBError> {
    let mut size = 0u64;
    for entry in fs::read_dir(dir).map_err(DBError::Io)? {
        let entry = entry.map_err(DBError::Io)?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "enc") {
            let file_size = entry.metadata().map_err(DBError::Io)?.len();
            size += file_size;
            files.push((path, file_size));
        } else if path.is_dir() {
            size += enforce_attachment_cache_limit_rec(&path, files)?;
        }
    }
    Ok(size)
}

pub fn enforce_attachment_cache_limit(limit_bytes: u64) -> Result<u64, DBError> {
    let cache_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("attachments");

    let mut files: Vec<(PathBuf, u64)> = Vec::new();
    let total_size = enforce_attachment_cache_limit_rec(&cache_dir, &mut files)?;

    if total_size > limit_bytes {
        files.sort_by_key(|f| {
            f.0.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::UNIX_EPOCH)
        });

        let mut removed = 0u64;
        for (path, size) in files {
            if total_size - removed <= limit_bytes {
                break;
            }
            fs::remove_file(&path)?;
            removed += size;
        }
        Ok(removed)
    } else {
        Ok(0)
    }
}
