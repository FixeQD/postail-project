use rusqlite::{Connection, params};
use crate::error::DBError;
use crate::db::MailHeader;
use super::safe_timestamp_from_utc;

pub fn add_tag(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    tag: &str,
) -> Result<(), DBError> {
    let message_id = super::get_message_table_id(conn, account_id, mailbox, uid)?
        .ok_or_else(|| DBError::Migration("Message not found".to_string()))?;

    conn.execute(
        "INSERT OR IGNORE INTO message_tags (message_id, tag) VALUES (?, ?)",
        params![message_id, tag],
    )?;

    Ok(())
}

pub fn remove_tag(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    tag: &str,
) -> Result<(), DBError> {
    let message_id = super::get_message_table_id(conn, account_id, mailbox, uid)?
        .ok_or_else(|| DBError::Migration("Message not found".to_string()))?;

    conn.execute(
        "DELETE FROM message_tags WHERE message_id = ? AND tag = ?",
        params![message_id, tag],
    )?;

    Ok(())
}

/// Fetch all messages with a specific tag for an account
pub fn fetch_tag_headers(
    conn: &Connection,
    account_id: &str,
    tag: &str,
    limit: u32,
) -> Result<Vec<MailHeader>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT m.uid, m.message_id, m.internal_date, m.subject, m.from_addr, m.to_json, m.cc_json, m.flags_json, m.snippet, m.has_attachments, m.starred, m.mailbox,
         (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
         FROM messages m
         JOIN message_tags mt_outer ON m.id = mt_outer.message_id
         WHERE m.account_id = ? AND mt_outer.tag = ?
         ORDER BY m.internal_date DESC
         LIMIT ?",
    )?;

    let headers_iter = stmt.query_map(params![account_id, tag, limit], |row| {
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
