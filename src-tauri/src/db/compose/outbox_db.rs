use crate::db::OutboxItem;
use crate::error::DBError;
use chrono::Utc;
use mailparse::{parse_mail, MailHeaderMap};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn enqueue_message(
    conn: &Connection,
    account_id: &str,
    raw_eml_path: &str,
    subject: Option<&str>,
    recipient: &str,
) -> Result<String, DBError> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO outbox (id, account_id, raw_eml_path, subject, recipient, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'PENDING', ?)",
        params![
            id,
            account_id,
            raw_eml_path,
            subject,
            recipient,
            Utc::now().timestamp()
        ],
    )?;
    Ok(id)
}

pub fn extract_headers_from_raw(raw_eml: &[u8]) -> (Option<String>, String) {
    let Ok(mail) = parse_mail(raw_eml) else {
        return (None, String::new());
    };
    let subject = mail
        .get_headers()
        .get_first_header("Subject")
        .map(|h| h.get_value());
    let recipient = mail
        .get_headers()
        .get_first_header("To")
        .map(|h| h.get_value())
        .unwrap_or_default();
    (subject, recipient)
}

pub fn list_outbox(conn: &Connection, account_id: &str) -> Result<Vec<OutboxItem>, DBError> {
    tracing::info!(target: "postail", "[OutboxDB] Listing outbox for account: {}", account_id);

    let mut stmt = conn.prepare(
        "SELECT id, subject, recipient, status, attempts, last_error FROM outbox WHERE account_id = ? ORDER BY created_at DESC",
    )?;
    let items_iter = stmt.query_map([account_id], |row| {
        Ok(OutboxItem {
            id: row.get(0)?,
            subject: row.get(1)?,
            recipient: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            status: row.get(3)?,
            last_error: row.get(5)?,
            attempts: row.get::<_, i64>(4)? as u32,
        })
    })?;
    let items: Result<Vec<OutboxItem>, _> = items_iter.collect();

    match &items {
        Ok(vec) => tracing::info!(target: "postail", "[OutboxDB] Found {} items", vec.len()),
        Err(e) => tracing::error!(target: "postail", "[OutboxDB] Error listing outbox: {}", e),
    }

    items.map_err(DBError::Sqlite)
}
