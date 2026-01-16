use rusqlite::{Connection, OptionalExtension};
use crate::error::DBError;

const DB_VERSION: u32 = 1;

pub fn get_db_version(conn: &Connection) -> Result<u32, DBError> {
    let version: Option<i64> = conn.query_row(
        "SELECT version FROM schema_versions ORDER BY version DESC LIMIT 1",
        [],
        |row| row.get(0)
    )
    .optional()
    .map_err(DBError::Sqlite)?;
    Ok(version.unwrap_or(0) as u32)
}

pub fn set_db_version(conn: &Connection, version: u32) -> Result<(), DBError> {
    conn.execute(
        "INSERT INTO schema_versions (version, applied_at) VALUES (?, ?)",
        [version as i64, chrono::Utc::now().timestamp()]
    )?;
    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<(), DBError> {
    let current_version = get_db_version(conn)?;

    if current_version < 1 {
        migrate_to_v1(conn)?;
        set_db_version(conn, 1)?;
    }

    Ok(())
}

fn migrate_to_v1(conn: &Connection) -> Result<(), DBError> {
    conn.execute("CREATE TABLE IF NOT EXISTS schema_versions (
        version INTEGER PRIMARY KEY,
        applied_at INTEGER NOT NULL
    )", [])?;

    if !column_exists(conn, "messages", "has_attachments")? {
        conn.execute("ALTER TABLE messages ADD COLUMN has_attachments INTEGER DEFAULT 0", [])?;
    }

    conn.execute("CREATE INDEX IF NOT EXISTS idx_messages_has_attachments ON messages(account_id, mailbox, has_attachments) WHERE has_attachments = 1", [])?;

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, DBError> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        &format!("SELECT {} FROM {} LIMIT 1", column, table),
        [],
        |row| row.get(0)
    );
    Ok(result.is_ok())
}
