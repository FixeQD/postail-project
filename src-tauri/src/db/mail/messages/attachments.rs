use rusqlite::{Connection, params};
use crate::error::DBError;

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
