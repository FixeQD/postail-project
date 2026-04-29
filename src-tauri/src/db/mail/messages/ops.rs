use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};
use crate::error::DBError;
use crate::db::upsert_from_address_string;

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
    pub tags: Vec<String>,
    pub structure_json: Option<String>,
}

pub fn batch_insert_messages(
    conn: &mut Connection,
    account_id: &str,
    mailbox: &str,
    items: &[MessageBatchItem],
    transaction_size: usize,
) -> Result<Vec<u32>, DBError> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let mut new_uids = Vec::new();
    let mut tx = conn.transaction()?;

    for (idx, item) in items.iter().enumerate() {
        if let Some(ref msg_id) = item.message_id {
            let _ = tx.execute(
                "DELETE FROM messages WHERE account_id = ? AND mailbox = ? AND message_id = ? AND uid < 0",
                rusqlite::params![account_id, mailbox, msg_id],
            );
        }

        let flags_json = serde_json::to_string(&item.flags).unwrap_or_default();
        let to_json = serde_json::to_string(&item.to).unwrap_or_default();

        let changes = tx.execute(
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

        if changes > 0 {
            new_uids.push(item.uid);
        }

        // Backfill snippet for rows that already existed with NULL snippet
        if item.snippet.is_some() {
            tx.execute(
                "UPDATE messages SET snippet = ? WHERE account_id = ? AND mailbox = ? AND uid = ? AND (snippet IS NULL OR snippet = '')",
                params![
                    item.snippet.as_deref(),
                    account_id,
                    mailbox,
                    item.uid,
                ],
            )?;
        }

        // Sync Tags to local DB
        if !item.tags.is_empty() {
            let message_id: i64 = tx.query_row(
                "SELECT id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
                params![account_id, mailbox, item.uid],
                |row| row.get(0),
            )?;

            for tag in &item.tags {
                tx.execute(
                    "INSERT OR IGNORE INTO message_tags (message_id, tag) VALUES (?, ?)",
                    params![message_id, tag],
                )?;
            }
        }

        if let Some(from) = &item.from {
            let _ = upsert_from_address_string(&tx, from);
        }
        for recipient in &item.to {
            let _ = upsert_from_address_string(&tx, recipient);
        }

        if (idx + 1) % transaction_size == 0 {
            tx.commit()?;
            tx = conn.transaction()?;
        }
    }

    tx.commit()?;
    Ok(new_uids)
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
