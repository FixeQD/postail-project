use crate::error::DBError;
use rusqlite::{Connection, OptionalExtension};

pub fn get_db_version(conn: &Connection) -> Result<u32, DBError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_versions'",
            [],
            |_| Ok(1),
        )
        .optional()?
        .is_some();

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

    if current_version < 2 {
        migrate_to_v2(conn)?;
        set_db_version(conn, 2)?;
    }

    if current_version < 3 {
        migrate_to_v3(conn)?;
        set_db_version(conn, 3)?;
    }

    if current_version < 4 {
        migrate_to_v4(conn)?;
        set_db_version(conn, 4)?;
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

fn migrate_to_v2(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "flag_sync_queue", "synced_at")? {
        conn.execute(
            "ALTER TABLE flag_sync_queue ADD COLUMN synced_at INTEGER",
            [],
        )?;
    }

    Ok(())
}

fn migrate_to_v3(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "mailboxes", "role")? {
        conn.execute(
            "ALTER TABLE mailboxes ADD COLUMN role TEXT DEFAULT 'other'",
            [],
        )?;
    }

    if !column_exists(conn, "mailboxes", "attributes_json")? {
        conn.execute("ALTER TABLE mailboxes ADD COLUMN attributes_json TEXT", [])?;
    }

    Ok(())
}

fn migrate_to_v4(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "flag_sync_queue", "operation_type")? {
        conn.execute(
            "ALTER TABLE flag_sync_queue ADD COLUMN operation_type TEXT DEFAULT 'flag'",
            [],
        )?;
    }

    if !column_exists(conn, "flag_sync_queue", "target_mailbox")? {
        conn.execute(
            "ALTER TABLE flag_sync_queue ADD COLUMN target_mailbox TEXT",
            [],
        )?;
    }

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, DBError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}
