use crate::error::DBError;
use ammonia;
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};

pub fn create_message_bodies_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_bodies (
            message_id INTEGER PRIMARY KEY,
            body_html_safe TEXT,
            body_plain TEXT,
            FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_bodies_message_id 
         ON message_bodies(message_id)",
        [],
    )?;

    Ok(())
}

pub fn save_message_body(
    conn: &Connection,
    message_table_id: i64,
    body_html: Option<&str>,
    body_plain: Option<&str>,
) -> Result<(), DBError> {
    let body_html_safe = body_html.map(ammonia::clean).unwrap_or_default();

    let body_text = body_plain.unwrap_or_else(|| body_html.unwrap_or(""));
    let snippet = body_text.chars().take(200).collect::<String>();

    conn.execute(
        "UPDATE messages SET snippet = ? WHERE id = ?",
        params![snippet, message_table_id],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO message_bodies (message_id, body_html_safe, body_plain)
         VALUES (?, ?, ?)",
        params![message_table_id, body_html_safe, body_plain.unwrap_or("")],
    )?;

    Ok(())
}

pub fn load_message_body(
    conn: &Connection,
    message_table_id: i64,
) -> Result<(Option<String>, String), DBError> {
    conn.query_row(
        "SELECT body_html_safe, body_plain FROM message_bodies WHERE message_id = ?",
        params![message_table_id],
        |row| {
            let html: Option<String> = row.get(0)?;
            let plain: String = row.get(1)?;
            Ok((html, plain))
        },
    )
    .optional()?
    .ok_or_else(|| DBError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}
