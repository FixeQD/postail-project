use crate::db::sql_helpers::*;
use crate::error::DBError;
use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), DBError> {
    // NOTE: journal_mode, synchronous, cache_size are already applied by apply_sqlcipher_key before this function is called
    create_table_if_not_exists(
        conn,
        "accounts",
        &[
            ("id", "TEXT PRIMARY KEY"),
            ("name", "TEXT NOT NULL"),
            ("email", "TEXT NOT NULL"),
            ("provider_type", "TEXT NOT NULL"),
            ("auth_type", "TEXT NOT NULL"),
            ("imap_host", "TEXT NOT NULL"),
            ("imap_port", "INTEGER NOT NULL"),
            ("imap_tls", "INTEGER NOT NULL"),
            ("smtp_host", "TEXT NOT NULL"),
            ("smtp_port", "INTEGER NOT NULL"),
            ("smtp_tls", "INTEGER NOT NULL"),
            ("creds_blob_path", "TEXT NOT NULL"),
            ("encryption_mode", "TEXT NOT NULL"),
            ("created_at", "INTEGER NOT NULL"),
        ],
    )?;

    create_table_if_not_exists(
        conn,
        "mailboxes",
        &[
            ("id", "INTEGER PRIMARY KEY"),
            ("account_id", "TEXT NOT NULL"),
            ("name", "TEXT NOT NULL"),
            ("uid_validity", "INTEGER"),
            ("highest_modseq", "INTEGER"),
            ("last_synced_uid", "INTEGER"),
            ("UNIQUE(account_id, name)", ""),
            (
                "FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    create_table_if_not_exists(
        conn,
        "messages",
        &[
            ("id", "INTEGER PRIMARY KEY"),
            ("account_id", "TEXT NOT NULL"),
            ("mailbox", "TEXT NOT NULL"),
            ("uid", "INTEGER NOT NULL"),
            ("message_id", "TEXT"),
            ("internal_date", "INTEGER NOT NULL"),
            ("from_addr", "TEXT"),
            ("to_json", "TEXT"),
            ("subject", "TEXT"),
            ("snippet", "TEXT"),
            ("flags_json", "TEXT"),
            ("cached_structure_json", "TEXT"),
            ("UNIQUE(account_id, mailbox, uid)", ""),
            (
                "FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    create_fts_table(
        conn,
        "messages_fts",
        &["subject", "from_addr", "snippet"],
        "messages",
        "id",
    )?;

    create_table_if_not_exists(
        conn,
        "drafts",
        &[
            ("id", "TEXT PRIMARY KEY"),
            ("account_id", "TEXT NOT NULL"),
            ("subject", "TEXT"),
            ("body", "TEXT"),
            ("to_json", "TEXT"),
            ("cc_json", "TEXT"),
            ("bcc_json", "TEXT"),
            ("attachments_json", "TEXT"),
            ("created_at", "INTEGER NOT NULL"),
            ("updated_at", "INTEGER NOT NULL"),
            (
                "FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    create_table_if_not_exists(
        conn,
        "outbox",
        &[
            ("id", "TEXT PRIMARY KEY"),
            ("account_id", "TEXT NOT NULL"),
            ("raw_eml_path", "TEXT NOT NULL"),
            ("subject", "TEXT"),
            ("recipient", "TEXT"),
            ("status", "TEXT NOT NULL"),
            ("attempts", "INTEGER DEFAULT 0"),
            ("last_error", "TEXT"),
            ("created_at", "INTEGER NOT NULL"),
            ("next_retry", "INTEGER"),
            (
                "FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    create_table_if_not_exists(
        conn,
        "attachments",
        &[
            ("id", "INTEGER PRIMARY KEY"),
            ("message_table_id", "INTEGER NOT NULL"),
            ("part_id", "TEXT NOT NULL"),
            ("filename", "TEXT"),
            ("mime_type", "TEXT NOT NULL"),
            ("size", "INTEGER NOT NULL"),
            ("cached_path", "TEXT"),
            ("is_inline", "INTEGER NOT NULL DEFAULT 0"),
            ("cid", "TEXT"),
            (
                "FOREIGN KEY(message_table_id) REFERENCES messages(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    create_table_if_not_exists(
        conn,
        "contacts",
        &[
            ("id", "INTEGER PRIMARY KEY"),
            ("email", "TEXT NOT NULL UNIQUE"),
            ("name", "TEXT"),
            ("last_contact_at", "INTEGER"),
            ("frequency", "INTEGER DEFAULT 1"),
        ],
    )?;

    create_fts_table(conn, "contacts_fts", &["email", "name"], "contacts", "id")?;

    create_table_if_not_exists(
        conn,
        "message_bodies",
        &[
            ("message_id", "INTEGER PRIMARY KEY"),
            ("body_html_safe", "TEXT"),
            ("body_plain", "TEXT NOT NULL DEFAULT ''"),
            ("parse_error", "TEXT"),
            (
                "FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    create_table_if_not_exists(
        conn,
        "settings",
        &[("key", "TEXT PRIMARY KEY"), ("value", "TEXT NOT NULL")],
    )?;

    create_table_if_not_exists(
        conn,
        "flag_sync_queue",
        &[
            ("id", "INTEGER PRIMARY KEY"),
            ("account_id", "TEXT NOT NULL"),
            ("mailbox", "TEXT NOT NULL"),
            ("uid", "INTEGER NOT NULL"),
            ("operation", "TEXT NOT NULL"),
            ("flags", "TEXT NOT NULL"),
            ("created_at", "INTEGER NOT NULL"),
            ("attempts", "INTEGER DEFAULT 0"),
            ("last_error", "TEXT"),
            (
                "FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE",
                "",
            ),
        ],
    )?;

    Ok(())
}

pub fn create_indexes(conn: &Connection) -> Result<(), DBError> {
    create_index_if_not_exists(
        conn,
        "idx_messages_account_mailbox",
        "messages",
        &["account_id", "mailbox"],
        false,
    )?;
    create_index_if_not_exists(
        conn,
        "idx_messages_uid",
        "messages",
        &["account_id", "mailbox", "uid"],
        false,
    )?;
    create_index_if_not_exists(
        conn,
        "idx_messages_internal_date",
        "messages",
        &["internal_date DESC"],
        false,
    )?;
    create_index_if_not_exists(
        conn,
        "idx_outbox_status_retry",
        "outbox",
        &["status", "next_retry"],
        false,
    )?;
    create_index_if_not_exists(
        conn,
        "idx_attachments_message",
        "attachments",
        &["message_table_id"],
        false,
    )?;

    create_index_if_not_exists(
        conn,
        "idx_contacts_frequency_date",
        "contacts",
        &["frequency DESC", "last_contact_at DESC"],
        false,
    )?;

    create_index_if_not_exists(
        conn,
        "idx_flag_sync_queue_account",
        "flag_sync_queue",
        &["account_id", "attempts"],
        false,
    )?;

    Ok(())
}

pub fn create_fts_triggers(conn: &Connection) -> Result<(), DBError> {
    create_trigger_if_not_exists(
        conn,
        "messages_fts_insert",
        "AFTER",
        "INSERT",
        "messages",
        "INSERT INTO messages_fts(rowid, subject, from_addr, snippet) VALUES (NEW.id, NEW.subject, NEW.from_addr, NEW.snippet);",
    )?;

    create_trigger_if_not_exists(
        conn,
        "messages_fts_update",
        "AFTER",
        "UPDATE",
        "messages",
        "UPDATE messages_fts SET subject = NEW.subject, from_addr = NEW.from_addr, snippet = NEW.snippet WHERE rowid = NEW.id;",
    )?;

    create_trigger_if_not_exists(
        conn,
        "messages_fts_delete",
        "AFTER",
        "DELETE",
        "messages",
        "DELETE FROM messages_fts WHERE rowid = OLD.id;",
    )?;

    create_trigger_if_not_exists(
        conn,
        "contacts_fts_insert",
        "AFTER",
        "INSERT",
        "contacts",
        "INSERT INTO contacts_fts(rowid, email, name) VALUES (NEW.id, NEW.email, NEW.name);",
    )?;

    create_trigger_if_not_exists(
        conn,
        "contacts_fts_update",
        "AFTER",
        "UPDATE",
        "contacts",
        "UPDATE contacts_fts SET email = NEW.email, name = NEW.name WHERE rowid = NEW.id;",
    )?;

    create_trigger_if_not_exists(
        conn,
        "contacts_fts_delete",
        "AFTER",
        "DELETE",
        "contacts",
        "DELETE FROM contacts_fts WHERE rowid = OLD.id;",
    )?;

    Ok(())
}
