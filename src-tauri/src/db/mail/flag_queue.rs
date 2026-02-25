use crate::error::DBError;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagOperation {
    pub id: i64,
    pub account_id: String,
    pub mailbox: String,
    pub uid: u32,
    pub operation: String,
    pub flags: Vec<String>,
    pub target_mailbox: Option<String>, // For MOVE operations
    pub operation_type: String,         // "flag" or "move"
    pub created_at: i64,
    pub attempts: i32,
    pub last_error: Option<String>,
}

pub fn enqueue_flag_change(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    operation: &str,
    flags: &[String],
) -> Result<i64, DBError> {
    let flags_json = serde_json::to_string(flags)?;
    conn.execute(
        "INSERT INTO flag_sync_queue (account_id, mailbox, uid, operation, flags, operation_type, created_at, attempts)
         VALUES (?, ?, ?, ?, ?, 'flag', ?, 0)",
        params![
            account_id,
            mailbox,
            uid,
            operation,
            flags_json,
            Utc::now().timestamp()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn enqueue_move_operation(
    conn: &Connection,
    account_id: &str,
    source_mailbox: &str,
    target_mailbox: &str,
    uid: u32,
) -> Result<i64, DBError> {
    conn.execute(
        "INSERT INTO flag_sync_queue (account_id, mailbox, uid, operation, flags, operation_type, target_mailbox, created_at, attempts)
         VALUES (?, ?, ?, 'move', '[]', 'move', ?, ?, 0)",
        params![
            account_id,
            source_mailbox,
            uid,
            target_mailbox,
            Utc::now().timestamp()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_pending_flag_operations(
    conn: &Connection,
    account_id: &str,
    max_attempts: i32,
) -> Result<Vec<FlagOperation>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, mailbox, uid, operation, flags, operation_type, target_mailbox, created_at, attempts, last_error
         FROM flag_sync_queue
         WHERE account_id = ? AND attempts < ? AND synced_at IS NULL
         ORDER BY created_at ASC",
    )?;

    let ops = stmt
        .query_map(params![account_id, max_attempts], |row| {
            let flags_json: String = row.get(5)?;
            let flags: Vec<String> = serde_json::from_str(&flags_json).unwrap_or_default();
            Ok(FlagOperation {
                id: row.get(0)?,
                account_id: row.get(1)?,
                mailbox: row.get(2)?,
                uid: row.get(3)?,
                operation: row.get(4)?,
                flags,
                operation_type: row
                    .get::<_, Option<String>>(6)?
                    .unwrap_or_else(|| "flag".to_string()),
                target_mailbox: row.get(7)?,
                created_at: row.get(8)?,
                attempts: row.get(9)?,
                last_error: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ops)
}

pub fn mark_flag_operation_success(conn: &Connection, id: i64) -> Result<(), DBError> {
    conn.execute(
        "UPDATE flag_sync_queue SET synced_at = ? WHERE id = ?",
        params![Utc::now().timestamp(), id],
    )?;
    Ok(())
}

pub fn mark_flag_operation_failed(conn: &Connection, id: i64, error: &str) -> Result<(), DBError> {
    conn.execute(
        "UPDATE flag_sync_queue SET attempts = attempts + 1, last_error = ? WHERE id = ?",
        params![error, id],
    )?;
    Ok(())
}

pub fn cleanup_old_synced_operations(conn: &Connection) -> Result<usize, DBError> {
    let threshold = Utc::now().timestamp() - 10;
    let deleted = conn.execute(
        "DELETE FROM flag_sync_queue WHERE synced_at IS NOT NULL AND synced_at < ?",
        params![threshold],
    )?;
    Ok(deleted)
}

pub fn clear_flag_queue(conn: &Connection, account_id: &str) -> Result<(), DBError> {
    conn.execute(
        "DELETE FROM flag_sync_queue WHERE account_id = ?",
        params![account_id],
    )?;
    Ok(())
}
