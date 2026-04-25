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

    if current_version < 12 {
        migrate_to_v12(conn)?;
        set_db_version(conn, 12)?;
    }

    if current_version < 13 {
        migrate_to_v13(conn)?;
        set_db_version(conn, 13)?;
    }

    if current_version < 14 {
        migrate_to_v14(conn)?;
        set_db_version(conn, 14)?;
    }

    if current_version < 15 {
        migrate_to_v15(conn)?;
        set_db_version(conn, 15)?;
    }

    if current_version < 16 {
        migrate_to_v16(conn)?;
        set_db_version(conn, 16)?;
    }

    if current_version < 17 {
        migrate_to_v17(conn)?;
        set_db_version(conn, 17)?;
    }

    if current_version < 18 {
        migrate_to_v18(conn)?;
        set_db_version(conn, 18)?;
    }

    if current_version < 19 {
        migrate_to_v19(conn)?;
        set_db_version(conn, 19)?;
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

fn migrate_to_v12(conn: &Connection) -> Result<(), DBError> {
    if !column_exists(conn, "mailboxes", "separator")? {
        conn.execute(
            "ALTER TABLE mailboxes ADD COLUMN separator TEXT DEFAULT '/'",
            [],
        )?;
    }
    Ok(())
}

fn migrate_to_v13(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS saved_searches (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            query_json TEXT NOT NULL,
            icon TEXT NOT NULL DEFAULT 'bookmark',
            position INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_saved_searches_account ON saved_searches(account_id, position)",
        [],
    )?;

    Ok(())
}

fn migrate_to_v14(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS message_bodies_fts
         USING fts5(body_plain, content='message_bodies', content_rowid='message_id')",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS message_bodies_fts_insert
         AFTER INSERT ON message_bodies BEGIN
           INSERT INTO message_bodies_fts(rowid, body_plain) VALUES (NEW.message_id, NEW.body_plain);
         END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS message_bodies_fts_update
         AFTER UPDATE ON message_bodies BEGIN
           UPDATE message_bodies_fts SET body_plain = NEW.body_plain WHERE rowid = NEW.message_id;
         END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS message_bodies_fts_delete
         AFTER DELETE ON message_bodies BEGIN
           DELETE FROM message_bodies_fts WHERE rowid = OLD.message_id;
         END",
        [],
    )?;

    // Backfill existing body data
    conn.execute(
        "INSERT INTO message_bodies_fts(rowid, body_plain)
         SELECT message_id, body_plain FROM message_bodies WHERE body_plain != ''",
        [],
    )?;

    Ok(())
}

fn migrate_to_v15(conn: &Connection) -> Result<(), DBError> {
    conn.execute("DROP TRIGGER IF EXISTS messages_fts_insert", [])?;
    conn.execute("DROP TRIGGER IF EXISTS messages_fts_update", [])?;
    conn.execute("DROP TRIGGER IF EXISTS messages_fts_delete", [])?;
    conn.execute("DROP TABLE IF EXISTS messages_fts", [])?;

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(subject, from_addr, to_json, snippet, content='messages', content_rowid='id')",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS messages_fts_insert
         AFTER INSERT ON messages BEGIN
            INSERT INTO messages_fts(rowid, subject, from_addr, to_json, snippet)
            VALUES (NEW.id, NEW.subject, NEW.from_addr, NEW.to_json, NEW.snippet);
         END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS messages_fts_update
         AFTER UPDATE ON messages BEGIN
            UPDATE messages_fts
            SET subject = NEW.subject, from_addr = NEW.from_addr, to_json = NEW.to_json, snippet = NEW.snippet
            WHERE rowid = NEW.id;
         END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS messages_fts_delete
         AFTER DELETE ON messages BEGIN
            DELETE FROM messages_fts WHERE rowid = OLD.id;
         END",
        [],
    )?;

    conn.execute(
        "INSERT INTO messages_fts(messages_fts) VALUES ('rebuild')",
        [],
    )?;

    Ok(())
}

fn migrate_to_v16(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS signatures (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            html_content TEXT NOT NULL DEFAULT '',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_signatures_account ON signatures(account_id)",
        [],
    )?;

    Ok(())
}

fn migrate_to_v17(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            subject TEXT NOT NULL DEFAULT '',
            html_body TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_templates_account ON templates(account_id)",
        [],
    )?;

    Ok(())
}

fn migrate_to_v18(conn: &Connection) -> Result<(), DBError> {
    // Extend contacts table with new profile fields
    let new_columns: &[(&str, &str)] = &[
        ("phone", "TEXT"),
        ("company", "TEXT"),
        ("notes", "TEXT"),
        ("avatar_url", "TEXT"),
        ("birthday", "INTEGER"),
    ];
    for (col, def) in new_columns {
        if !column_exists(conn, "contacts", col)? {
            conn.execute(
                &format!("ALTER TABLE contacts ADD COLUMN {} {}", col, def),
                [],
            )?;
        }
    }

    // Rebuild contacts_fts to include company and notes.
    // Drop triggers first, then the table, then recreate both.
    conn.execute("DROP TRIGGER IF EXISTS contacts_fts_insert", [])?;
    conn.execute("DROP TRIGGER IF EXISTS contacts_fts_update", [])?;
    conn.execute("DROP TRIGGER IF EXISTS contacts_fts_delete", [])?;
    conn.execute("DROP TABLE IF EXISTS contacts_fts", [])?;

    conn.execute(
        "CREATE VIRTUAL TABLE contacts_fts USING fts5(email, name, company, notes, content='contacts', content_rowid='id')",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER contacts_fts_insert AFTER INSERT ON contacts BEGIN          INSERT INTO contacts_fts(rowid, email, name, company, notes)          VALUES (NEW.id, NEW.email, NEW.name, NEW.company, NEW.notes); END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER contacts_fts_update AFTER UPDATE ON contacts BEGIN          UPDATE contacts_fts SET email = NEW.email, name = NEW.name,          company = NEW.company, notes = NEW.notes WHERE rowid = NEW.id; END",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER contacts_fts_delete AFTER DELETE ON contacts BEGIN          DELETE FROM contacts_fts WHERE rowid = OLD.id; END",
        [],
    )?;

    // Backfill FTS from existing contacts rows
    conn.execute(
        "INSERT INTO contacts_fts(rowid, email, name, company, notes)          SELECT id, email, name, company, notes FROM contacts",
        [],
    )?;

    Ok(())
}

fn migrate_to_v19(conn: &Connection) -> Result<(), DBError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contact_groups (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS contact_group_members (
            group_id INTEGER NOT NULL,
            contact_id INTEGER NOT NULL,
            PRIMARY KEY(group_id, contact_id),
            FOREIGN KEY(group_id) REFERENCES contact_groups(id) ON DELETE CASCADE,
            FOREIGN KEY(contact_id) REFERENCES contacts(id) ON DELETE CASCADE
        )",
        [],
    )?;

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
    "message_bodies_fts",
    "messages",
    "outbox",
    "saved_searches",
    "schema_versions",
    "settings",
    "message_tags",
    "tag_colors",
    "filter_rules",
    "signatures",
    "templates",
    "contact_groups",
    "contact_group_members",
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
