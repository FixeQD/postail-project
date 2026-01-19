use crate::error::DBError;
use rusqlite::{Connection, OptionalExtension};

pub fn get_db_version(conn: &Connection) -> Result<u32, DBError> {
    let table_exists: Result<Option<String>, _> = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_versions'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional();

    let table_exists = matches!(table_exists, Ok(Some(_)));

    if !table_exists {
        return Ok(0);
    }

    if !table_exists {
        return Ok(0);
    }

    let version: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_versions ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(DBError::Sqlite)?;
    Ok(version.unwrap_or(0) as u32)
}

pub fn set_db_version(conn: &Connection, version: u32) -> Result<(), DBError> {
    conn.execute(
        "INSERT INTO schema_versions (version, applied_at) VALUES (?, ?)",
        [version as i64, chrono::Utc::now().timestamp()],
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_versions (
        version INTEGER PRIMARY KEY,
        applied_at INTEGER NOT NULL
    )",
        [],
    )?;

    if !column_exists(conn, "messages", "has_attachments")? {
        conn.execute(
            "ALTER TABLE messages ADD COLUMN has_attachments INTEGER DEFAULT 0",
            [],
        )?;
    }

    conn.execute("CREATE INDEX IF NOT EXISTS idx_messages_has_attachments ON messages(account_id, mailbox, has_attachments) WHERE has_attachments = 1", [])?;

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, DBError> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        &format!("SELECT {} FROM {} LIMIT 1", column, table),
        [],
        |row| row.get(0),
    );
    Ok(result.is_ok())
}
