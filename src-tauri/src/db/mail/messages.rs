use crate::db::upsert_from_address_string;
use crate::db::AttachmentMeta;
use crate::db::MailHeader;
use crate::db::MessageFull;
use crate::error::DBError;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};

pub const DEFAULT_BATCH_SIZE: usize = 50;

pub(crate) fn safe_timestamp_from_utc(seconds: i64) -> Option<DateTime<Utc>> {
    if seconds == 0 {
        Some(Utc::now())
    } else {
        Utc.timestamp_opt(seconds, 0).single()
    }
}

pub struct MessageBatchItem {
    pub uid: u32,
    pub message_id: Option<String>,
    pub internal_date: DateTime<Utc>,
    pub from: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub flags: Vec<String>,
    pub structure_json: Option<String>,
}

pub fn batch_insert_messages(
    conn: &mut Connection,
    account_id: &str,
    mailbox: &str,
    items: &[MessageBatchItem],
    transaction_size: usize,
) -> Result<usize, DBError> {
    if items.is_empty() {
        return Ok(0);
    }

    let mut total_inserted = 0;
    let mut tx = conn.transaction()?;

    for (idx, item) in items.iter().enumerate() {
        let flags_json = serde_json::to_string(&item.flags).unwrap_or_default();
        let to_json = serde_json::to_string(&item.to).unwrap_or_default();

        tx.execute(
            "INSERT OR IGNORE INTO messages (account_id, mailbox, uid, message_id, internal_date, from_addr, to_json, subject, snippet, flags_json, cached_structure_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                account_id,
                mailbox,
                item.uid,
                item.message_id.as_deref(),
                item.internal_date.timestamp(),
                item.from.as_deref(),
                to_json,
                item.subject.as_deref(),
                item.snippet.as_deref(),
                flags_json,
                item.structure_json.as_deref(),
            ],
        )?;

        if let Some(from) = &item.from {
            let _ = upsert_from_address_string(&tx, from);
        }
        for recipient in &item.to {
            let _ = upsert_from_address_string(&tx, recipient);
        }

        total_inserted += 1;

        if (idx + 1) % transaction_size == 0 {
            tx.commit()?;
            tx = conn.transaction()?;
        }
    }

    tx.commit()?;
    Ok(total_inserted)
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
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments
             FROM messages WHERE account_id = ? AND mailbox = ? AND uid > ? ORDER BY uid DESC LIMIT ?",
            vec![account_id.to_string(), mailbox.to_string(), anchor.to_string(), limit.to_string()],
        )
    } else {
        (
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments
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
        let cc_json: Option<String> = row.get(6)?;
        let cc: Vec<String> = cc_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let flags_json: Option<String> = row.get(7)?;
        let flags: Vec<String> = flags_json
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        Ok(MailHeader {
            uid: row.get::<_, u32>(0)?,
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
        })
    })?;

    let headers: Result<Vec<MailHeader>, _> = headers_iter.collect();
    headers.map_err(DBError::Sqlite)
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

pub fn fetch_message_full(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<MessageFull>, DBError> {
    // Load header only
    let header = conn
        .query_row(
            "SELECT id, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments
             FROM messages
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
                Ok(MailHeader {
                    uid,
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
    use rusqlite::OptionalExtension;
    let message_table_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![account_id, mailbox, uid],
            |row| row.get(0),
        )
        .optional()?;

    let Some(id) = message_table_id else {
        return Ok(vec![]);
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

pub struct MessageUpsertData {
    pub uid: u32,
    pub message_id: Option<String>,
    pub internal_date: DateTime<Utc>,
    pub from: Option<String>,
    pub to_json: Option<String>,
    pub subject: Option<String>,
    pub snippet: Option<String>,
    pub flags_json: Option<String>,
    pub structure_json: Option<String>,
}

pub fn upsert_message(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    data: &MessageUpsertData,
) -> Result<i64, DBError> {
    conn.execute(
        "INSERT OR REPLACE INTO messages (account_id, mailbox, uid, message_id, internal_date, from_addr, to_json, subject, snippet, flags_json, cached_structure_json)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            account_id,
            mailbox,
            data.uid,
            data.message_id.as_deref(),
            data.internal_date.timestamp(),
            data.from.as_deref(),
            data.to_json.as_deref(),
            data.subject.as_deref(),
            data.snippet.as_deref(),
            data.flags_json.as_deref(),
            data.structure_json.as_deref(),
        ],
    )?;

    if let Some(from) = &data.from {
        let _ = upsert_from_address_string(conn, from);
        if let Some(to) = &data.to_json {
            let to_list: Vec<String> = serde_json::from_str(to).unwrap_or_default();
            for recipient in to_list {
                let _ = upsert_from_address_string(conn, &recipient);
            }
        }
    }

    Ok(conn.last_insert_rowid())
}

pub fn update_message_flags(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
    add_flags: Option<&[&str]>,
    remove_flags: Option<&[&str]>,
) -> Result<usize, DBError> {
    if uids.is_empty() {
        return Ok(0);
    }

    for uid in uids {
        let current_flags: Option<String> = conn.query_row(
            "SELECT flags_json FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![account_id, mailbox, uid],
            |row| row.get(0),
        )?;

        let mut flags: Vec<String> = current_flags
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        if let Some(add) = add_flags {
            for flag in add {
                if !flags.contains(&flag.to_string()) {
                    flags.push(flag.to_string());
                }
            }
        }

        if let Some(remove) = remove_flags {
            for flag in remove {
                flags.retain(|f| f != flag);
            }
        }

        conn.execute(
            "UPDATE messages SET flags_json = ? WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![
                serde_json::to_string(&flags).unwrap_or_default(),
                account_id,
                mailbox,
                uid
            ],
        )?;
    }

    Ok(uids.len())
}

pub fn mark_read(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
    read: bool,
) -> Result<usize, DBError> {
    if read {
        update_message_flags(conn, account_id, mailbox, uids, Some(&["\\Seen"]), None)
    } else {
        update_message_flags(conn, account_id, mailbox, uids, None, Some(&["\\Seen"]))
    }
}

pub fn move_to_trash(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
) -> Result<usize, DBError> {
    update_message_flags(conn, account_id, mailbox, uids, Some(&["\\Deleted"]), None)
}

pub fn sync_message_attachments_flag(
    message_table_id: i64,
    conn: &Connection,
) -> Result<(), DBError> {
    let has_attachments: bool = conn.query_row(
        "SELECT COUNT(*) FROM attachments WHERE message_table_id = ?",
        params![message_table_id],
        |row| row.get::<_, i64>(0).map(|c| c > 0),
    )?;

    conn.execute(
        "UPDATE messages SET has_attachments = ? WHERE id = ?",
        params![if has_attachments { 1 } else { 0 }, message_table_id],
    )?;
    Ok(())
}

pub fn refresh_all_attachments_flags(conn: &Connection) -> Result<usize, DBError> {
    conn.execute(
        "UPDATE messages SET has_attachments = 1 WHERE id IN (SELECT DISTINCT message_table_id FROM attachments)",
        [],
    )?;
    conn.execute(
        "UPDATE messages SET has_attachments = 0 WHERE id NOT IN (SELECT DISTINCT message_table_id FROM attachments) AND has_attachments = 1",
        [],
    )?;
    Ok(conn.changes() as usize)
}
