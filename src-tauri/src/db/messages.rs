use rusqlite::{params, Connection, Result as SqlResult};
use chrono::{DateTime, Utc};
use crate::error::DBError;
use super::MailHeader;
use super::MessageFull;

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
            has_attachments: false,
        })
    })?;

    let headers: Result<Vec<MailHeader>, _> = headers_iter.collect();
    headers.map_err(DBError::Sqlite)
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
                has_attachments: false,
            })
        })
        .optional()?;

    if let Some(header) = header {
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
