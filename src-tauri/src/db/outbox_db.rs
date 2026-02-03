use crate::db::OutboxItem;
use crate::error::DBError;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

/// Insert a new outbox record with status `PENDING` and return its generated ID.
///
/// The function generates a new UUID v4, stores a row in the `outbox` table with the
/// provided `account_id` and `raw_eml_path`, sets `status` to `PENDING`, and records
/// the current timestamp as `created_at`.
///
/// # Returns
///
/// The generated UUID string for the newly inserted outbox row on success; returns a `DBError` on failure.
///
/// # Examples
///
/// ```no_run
/// let id = enqueue_message(&conn, "account-123", "/var/mail/msg.eml").unwrap();
/// assert_eq!(id.len(), 36);
/// ```
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

/// Lists outbox entries for the given account from the database.
///
/// Retrieves rows from the outbox table for `account_id` and maps them into `OutboxItem`
/// structures. Mapped items will contain `id`, `status`, `error_log`, `attempts`, and
/// `raw_eml_path`; `subject` will be `None` and `recipient` will be an empty string.
///
/// # Parameters
///
/// - `account_id`: account identifier used to filter outbox entries.
///
/// # Returns
///
/// `Ok(Vec<OutboxItem>)` with the matching outbox items on success, or `Err(DBError::Sqlite(_))`
/// if a database error occurs.
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
/// // setup an in-memory DB and call the function (schema and data omitted for brevity)
/// let conn = Connection::open_in_memory().unwrap();
/// let items = list_outbox(&conn, "account-123").unwrap();
/// assert!(items.is_empty() || items.iter().all(|i| i.id.len() > 0));
/// ```
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