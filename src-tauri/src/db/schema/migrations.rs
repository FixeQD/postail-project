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

    if current_version < 5 {
        migrate_to_v5(conn)?;
        set_db_version(conn, 5)?;
    }

    if current_version < 6 {
        migrate_to_v6(conn)?;
        set_db_version(conn, 6)?;
    }

    if current_version < 7 {
        migrate_to_v7(conn)?;
        set_db_version(conn, 7)?;
    }

    if current_version < 8 {
        migrate_to_v8(conn)?;
        set_db_version(conn, 8)?;
    }

    if current_version < 9 {
        migrate_to_v9(conn)?;
        set_db_version(conn, 9)?;
    }

    if current_version < 10 {
        migrate_to_v10(conn)?;
        set_db_version(conn, 10)?;
    }

    if current_version < 11 {
        migrate_to_v11(conn)?;
        set_db_version(conn, 11)?;
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

fn migrate_to_v5(conn: &Connection) -> Result<(), DBError> {
    // Ensure message_bodies table exists for existing installs.
    // Raw EML is stored as encrypted files on disk (eml_cache), NOT as BLOB in DB.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_bodies (
            message_id INTEGER PRIMARY KEY,
            body_html_safe TEXT,
            body_plain TEXT NOT NULL DEFAULT '',
            parse_error TEXT,
            FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_bodies_message_id ON message_bodies(message_id)",
        [],
    )?;

    // Add is_inline and cid columns to attachments for inline image support
    if !column_exists(conn, "attachments", "is_inline")? {
        conn.execute(
            "ALTER TABLE attachments ADD COLUMN is_inline INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    if !column_exists(conn, "attachments", "cid")? {
        conn.execute("ALTER TABLE attachments ADD COLUMN cid TEXT", [])?;
    }

    Ok(())
}

fn migrate_to_v6(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "mailboxes", "role_customized")? {
        conn.execute(
            "ALTER TABLE mailboxes ADD COLUMN role_customized INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

fn migrate_to_v7(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "messages", "starred")? {
        conn.execute(
            "ALTER TABLE messages ADD COLUMN starred INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_starred ON messages(account_id, starred) WHERE starred = 1",
        [],
    )?;

    Ok(())
}

fn migrate_to_v8(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_tags (
            message_id INTEGER NOT NULL,
            tag TEXT NOT NULL,
            PRIMARY KEY(message_id, tag),
            FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_tags_tag ON message_tags(tag)",
        [],
    )?;

    Ok(())
}

fn migrate_to_v9(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag_colors (
            tag TEXT PRIMARY KEY,
            hue INTEGER NOT NULL DEFAULT 200
        )",
        [],
    )?;
    Ok(())
}

fn migrate_to_v10(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS filter_rules (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            match_mode TEXT NOT NULL DEFAULT 'all',
            conditions_json TEXT NOT NULL DEFAULT '[]',
            actions_json TEXT NOT NULL DEFAULT '[]',
            position INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_filter_rules_account ON filter_rules(account_id, position)",
        [],
    )?;

    Ok(())
}

fn migrate_to_v11(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "mailboxes", "hidden")? {
        conn.execute(
            "ALTER TABLE mailboxes ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

// Whitelist of allowed table names to prevent SQL injection
const ALLOWED_TABLES: &[&str] = &[
    "messages",
    "flag_sync_queue",
    "mailboxes",
    "accounts",
    "attachments",
    "contacts",
    "drafts",
    "message_bodies",
    "messages",
    "outbox",
    "schema_versions",
    "settings",
    "message_tags",
    "tag_colors",
    "filter_rules",
];

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, DBError> {
    if !ALLOWED_TABLES.contains(&table) {
        return Err(DBError::Migration(format!("Invalid table name: {}", table)));
    }

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
