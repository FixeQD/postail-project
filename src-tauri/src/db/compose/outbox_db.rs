use crate::db::OutboxItem;
use crate::error::DBError;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn enqueue_message(
    conn: &Connection,
    account_id: &str,
    raw_eml_path: &str,
) -> Result<String, DBError> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO outbox (id, account_id, raw_eml_path, status, created_at)
         VALUES (?, ?, ?, 'PENDING', ?)",
        params![id, account_id, raw_eml_path, Utc::now().timestamp()],
    )?;
    Ok(id)
}

pub fn list_outbox(conn: &Connection, account_id: &str) -> Result<Vec<OutboxItem>, DBError> {
    tracing::info!(target: "postail", "[OutboxDB] Listing outbox for account: {}", account_id);

    let mut stmt = conn.prepare(
        "SELECT id, raw_eml_path, status, attempts, last_error FROM outbox WHERE account_id = ?",
    )?;
    let items_iter = stmt.query_map([account_id], |row| {
        Ok(OutboxItem {
            id: row.get(0)?,
            subject: None,
            recipient: String::new(),
            status: row.get(2)?,
            error_log: row.get(4)?,
            attempts: row.get::<_, i64>(3)? as u32,
        })
    })?;
    let items: Result<Vec<OutboxItem>, _> = items_iter.collect();

    match &items {
        Ok(vec) => tracing::info!(target: "postail", "[OutboxDB] Found {} items", vec.len()),
        Err(e) => tracing::error!(target: "postail", "[OutboxDB] Error listing outbox: {}", e),
    }

    items.map_err(DBError::Sqlite)
}
