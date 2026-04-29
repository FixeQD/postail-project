pub mod ops;
pub mod flags;
pub mod tags;
pub mod attachments;

pub use ops::*;
pub use flags::*;
pub use tags::*;
pub use attachments::*;

use crate::db::AttachmentMeta;
use crate::db::MailHeader;
use crate::db::MessageFull;
use crate::error::DBError;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, OptionalExtension, params};

pub const DEFAULT_BATCH_SIZE: usize = 50;

pub(crate) fn safe_timestamp_from_utc(seconds: i64) -> Option<DateTime<Utc>> {
    if seconds == 0 {
        Some(Utc::now())
    } else {
        Utc.timestamp_opt(seconds, 0).single()
    }
}

pub fn get_message_table_id(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<i64>, DBError> {
    conn.query_row(
        "SELECT id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
        params![account_id, mailbox, uid],
        |row| row.get(0),
    )
    .optional()
    .map_err(DBError::Sqlite)
}

pub fn fetch_headers(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    anchor: Option<u32>,
    limit: u32,
) -> Result<Vec<MailHeader>, DBError> {
    let (query, params) = if let Some(anchor) = anchor {
        (
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments, starred, mailbox,
             (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
             FROM messages m WHERE account_id = ? AND mailbox = ? AND uid > ? ORDER BY uid DESC LIMIT ?",
            vec![account_id.to_string(), mailbox.to_string(), anchor.to_string(), limit.to_string()],
        )
    } else {
        (
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments, starred, mailbox,
             (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
             FROM messages m WHERE account_id = ? AND mailbox = ? ORDER BY uid DESC LIMIT ?",
            vec![account_id.to_string(), mailbox.to_string(), limit.to_string()],
        )
    };

    let mut stmt = conn.prepare(query)?;
    let headers_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        let to_json: Option<String> = row.get(5)?;
        let to: Vec<String> = to_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let cc_json: Option<String> = row.get(6)?;
        let cc: Vec<String> = cc_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let flags_json: Option<String> = row.get(7)?;
        let flags: Vec<String> = flags_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let tags_json: Option<String> = row.get(12)?;
        let tags: Vec<String> = tags_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        Ok(MailHeader {
            uid: row.get::<_, u32>(0)?,
            mailbox: row.get(11)?,
            message_id: row.get(1)?,
            internal_date: safe_timestamp_from_utc(row.get::<_, i64>(2)?)
                .ok_or_else(|| rusqlite::Error::InvalidColumnName("internal_date".into()))?,
            subject: row.get(3)?,
            from: vec![row.get::<_, Option<String>>(4)?.unwrap_or_default()],
            to,
            cc,
            flags,
            snippet: row.get(8)?,
            has_attachments: row.get::<_, i64>(9)? != 0,
            starred: row.get::<_, i64>(10)? != 0,
            tags,
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
    // Load header only
    let header = conn
        .query_row(
            "SELECT id, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments, starred, mailbox,
             (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
             FROM messages m
             WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![account_id, mailbox, uid],
            |row| {
                let to_json: Option<String> = row.get(5)?;
                let to: Vec<String> = to_json
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();
                let cc_json: Option<String> = row.get(6)?;
                let cc: Vec<String> = cc_json
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();
                let flags_json: Option<String> = row.get(7)?;
                let flags: Vec<String> = flags_json
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();
                let tags_json: Option<String> = row.get(12)?;
                let tags: Vec<String> = tags_json
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();

                Ok(MailHeader {
                    uid,
                    mailbox: row.get(11)?,
                    message_id: row.get(1)?,
                    internal_date: safe_timestamp_from_utc(row.get::<_, i64>(2)?)
                        .ok_or_else(|| rusqlite::Error::InvalidColumnName("internal_date".into()))?,
                    subject: row.get(3)?,
                    from: vec![row.get::<_, Option<String>>(4)?.unwrap_or_default()],
                    to,
                    cc,
                    flags,
                    snippet: row.get(8)?,
                    has_attachments: row.get::<_, i64>(9)? != 0,
                    starred: row.get::<_, i64>(10)? != 0,
                    tags,
                })
            },
        )
        .optional()?;

    if let Some(header) = header {
        let all_attachments = load_message_attachments(conn, account_id, mailbox, uid)?;
        let (attachments, inline_images): (Vec<_>, Vec<_>) = all_attachments
            .into_iter()
            .partition(|(is_inline, _)| !is_inline);
        let attachments = attachments.into_iter().map(|(_, a)| a).collect();
        let inline_images = inline_images.into_iter().map(|(_, a)| a).collect();

        // body_html / body_plain start empty - caller injects from file cache
        Ok(Some(MessageFull {
            header,
            body_html_safe: String::new(),
            body_plain: String::new(),
            attachments,
            inline_images,
            read_receipt_to: None,
        }))
    } else {
        Ok(None)
    }
}

fn load_message_attachments(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Vec<(bool, AttachmentMeta)>, DBError> {
    let id = match get_message_table_id(conn, account_id, mailbox, uid)? {
        Some(id) => id,
        None => return Ok(vec![]),
    };

    let mut stmt = conn.prepare(
        "SELECT part_id, filename, mime_type, size, is_inline, cached_path, cid FROM attachments WHERE message_table_id = ?",
    )?;
    let rows = stmt.query_map(params![id], |row| {
        Ok((
            row.get::<_, i64>(4)? != 0,
            AttachmentMeta {
                part_id: row.get(0)?,
                filename: row.get(1)?,
                mime_type: row.get(2)?,
                size: row.get::<_, i64>(3)? as u64,
                cached_path: row.get(5)?,
                cid: row.get(6)?,
            },
        ))
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DBError::Sqlite)
}

/// Fetch all starred messages for an account
pub fn fetch_starred_headers(
    conn: &Connection,
    account_id: &str,
    limit: u32,
) -> Result<Vec<MailHeader>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments, starred, mailbox,
         (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
         FROM messages m
         WHERE account_id = ? AND starred = 1
         ORDER BY internal_date DESC
         LIMIT ?",
    )?;

    let headers_iter = stmt.query_map(params![account_id, limit], |row| {
        let to_json: Option<String> = row.get(5)?;
        let to: Vec<String> = to_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let cc_json: Option<String> = row.get(6)?;
        let cc: Vec<String> = cc_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let flags_json: Option<String> = row.get(7)?;
        let flags: Vec<String> = flags_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let tags_json: Option<String> = row.get(12)?;
        let tags: Vec<String> = tags_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        Ok(MailHeader {
            uid: row.get::<_, u32>(0)?,
            mailbox: row.get(11)?,
            message_id: row.get(1)?,
            internal_date: safe_timestamp_from_utc(row.get::<_, i64>(2)?)
                .ok_or_else(|| rusqlite::Error::InvalidColumnName("internal_date".into()))?,
            subject: row.get(3)?,
            from: vec![row.get::<_, Option<String>>(4)?.unwrap_or_default()],
            to,
            cc,
            flags,
            snippet: row.get(8)?,
            has_attachments: row.get::<_, i64>(9)? != 0,
            starred: row.get::<_, i64>(10)? != 0,
            tags,
        })
    })?;

    let headers: Result<Vec<MailHeader>, _> = headers_iter.collect();
    headers.map_err(DBError::Sqlite)
}
