use chrono::Utc;
use postail_project_lib::db::{
    attachments::{add_attachment, add_attachment_bytes, add_inline_attachment_bytes, remove_attachment},
    contacts::{search_contacts, upsert_contact, upsert_from_address_string},
    drafts::{delete_draft, list_drafts, load_draft, save_draft, Draft, DraftAttachment},
    mailbox::{fetch_mailboxes, upsert_mailbox},
    messages::{
        batch_insert_messages, fetch_headers, fetch_message_full, mark_read, move_to_trash,
        refresh_all_attachments_flags, sync_message_attachments_flag, upsert_message,
        update_message_flags, MessageBatchItem, MessageUpsertData, DEFAULT_BATCH_SIZE,
    },
    outbox_db::{enqueue_message, list_outbox},
    tables::{create_fts_triggers, create_indexes, create_tables},
    Mailbox,
};
use postail_project_lib::security::SecurityManager;
use rusqlite::Connection;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

mod test_helpers {
    use super::*;
    use postail_project_lib::security::{
        manager::SecurityManager,
        master_key::MasterKey,
        stores::{argon2::Argon2Store, StorageTier},
    };
    use std::sync::Arc;

    pub fn init_temp_db() -> Connection {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path();
        fs::remove_file(db_path).unwrap();
        let conn = Connection::open(db_path).unwrap();
        create_tables(&conn).unwrap();
        create_indexes(&conn).unwrap();
        create_fts_triggers(&conn).unwrap();
        conn
    }

    pub fn test_manager() -> SecurityManager {
        use std::env::temp_dir;

        let temp_path = temp_dir().join(format!("test_postail_key_{}", uuid::Uuid::new_v4()));
        let store = Argon2Store::new(temp_path, "test_passphrase".to_string());
        let mut manager = SecurityManager::with_store(Arc::new(store), StorageTier::Passphrase);
        let fixed_key = MasterKey::from_bytes(&[0u8; 32]).unwrap();
        manager.initialize_with_key(fixed_key).unwrap();
        manager.unlock().unwrap();
        manager
    }

    pub fn create_test_account(conn: &Connection, security: &SecurityManager) -> String {
        use postail_project_lib::db::{
            add_account, AccountInput, Credentials, ImapConfig, PasswordCredentials, SmtpConfig,
        };

        let input = AccountInput {
            name: "Test Account".to_string(),
            email: "test@example.com".to_string(),
            auth_type: "password".to_string(),
            imap_config: ImapConfig {
                host: "imap.example.com".to_string(),
                port: 993,
                tls: true,
            },
            smtp_config: SmtpConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                tls: true,
            },
            credentials: Credentials::Password(PasswordCredentials {
                username: "test@example.com".to_string(),
                password: "secret".to_string(),
            }),
        };

        let meta = add_account(conn, input, security).unwrap();
        meta.id
    }
}

use test_helpers::*;

// ============================================================================
// Mailbox tests
// ============================================================================

#[test]
fn test_upsert_mailbox_insert() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let mailbox = Mailbox {
        name: "INBOX".to_string(),
        display_name: "Inbox".to_string(),
        role: "inbox".to_string(),
        uid_validity: Some(12345),
        highest_modseq: Some(100),
        last_synced_uid: Some(50),
    };

    upsert_mailbox(&conn, &account_id, &mailbox).unwrap();

    let mailboxes = fetch_mailboxes(&conn, &account_id).unwrap();
    assert_eq!(mailboxes.len(), 1);
    assert_eq!(mailboxes[0].name, "INBOX");
    assert_eq!(mailboxes[0].uid_validity, Some(12345));
    assert_eq!(mailboxes[0].highest_modseq, Some(100));
    assert_eq!(mailboxes[0].last_synced_uid, Some(50));
}

#[test]
fn test_upsert_mailbox_update() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let mailbox1 = Mailbox {
        name: "INBOX".to_string(),
        display_name: "Inbox".to_string(),
        role: "inbox".to_string(),
        uid_validity: Some(12345),
        highest_modseq: Some(100),
        last_synced_uid: Some(50),
    };
    upsert_mailbox(&conn, &account_id, &mailbox1).unwrap();

    let mailbox2 = Mailbox {
        name: "INBOX".to_string(),
        display_name: "Inbox".to_string(),
        role: "inbox".to_string(),
        uid_validity: Some(12345),
        highest_modseq: Some(200),
        last_synced_uid: Some(100),
    };
    upsert_mailbox(&conn, &account_id, &mailbox2).unwrap();

    let mailboxes = fetch_mailboxes(&conn, &account_id).unwrap();
    assert_eq!(mailboxes.len(), 1);
    assert_eq!(mailboxes[0].highest_modseq, Some(200));
    assert_eq!(mailboxes[0].last_synced_uid, Some(100));
}

#[test]
fn test_fetch_mailboxes_empty() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let mailboxes = fetch_mailboxes(&conn, &account_id).unwrap();
    assert_eq!(mailboxes.len(), 0);
}

#[test]
fn test_fetch_mailboxes_multiple() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let mailboxes = vec![
        Mailbox {
            name: "INBOX".to_string(),
            display_name: "Inbox".to_string(),
            role: "inbox".to_string(),
            uid_validity: Some(1),
            highest_modseq: None,
            last_synced_uid: None,
        },
        Mailbox {
            name: "Sent".to_string(),
            display_name: "Sent".to_string(),
            role: "sent".to_string(),
            uid_validity: Some(2),
            highest_modseq: None,
            last_synced_uid: None,
        },
        Mailbox {
            name: "Drafts".to_string(),
            display_name: "Drafts".to_string(),
            role: "drafts".to_string(),
            uid_validity: Some(3),
            highest_modseq: None,
            last_synced_uid: None,
        },
    ];

    for mailbox in &mailboxes {
        upsert_mailbox(&conn, &account_id, mailbox).unwrap();
    }

    let fetched = fetch_mailboxes(&conn, &account_id).unwrap();
    assert_eq!(fetched.len(), 3);
}

// ============================================================================
// Messages tests
// ============================================================================

#[test]
fn test_batch_insert_messages() {
    let mut conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let items = vec![
        MessageBatchItem {
            uid: 1,
            message_id: Some("msg1@example.com".to_string()),
            internal_date: Utc::now(),
            from: Some("sender@example.com".to_string()),
            to: vec!["recipient@example.com".to_string()],
            subject: Some("Test Subject 1".to_string()),
            snippet: Some("Test snippet 1".to_string()),
            flags: vec!["\\Seen".to_string()],
            structure_json: None,
        },
        MessageBatchItem {
            uid: 2,
            message_id: Some("msg2@example.com".to_string()),
            internal_date: Utc::now(),
            from: Some("sender2@example.com".to_string()),
            to: vec!["recipient2@example.com".to_string()],
            subject: Some("Test Subject 2".to_string()),
            snippet: Some("Test snippet 2".to_string()),
            flags: vec!["\\Flagged".to_string()],
            structure_json: None,
        },
    ];

    let inserted = batch_insert_messages(&mut conn, &account_id, mailbox, &items, 100).unwrap();
    assert_eq!(inserted, 2);

    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert_eq!(headers.len(), 2);
}

#[test]
fn test_batch_insert_messages_empty() {
    let mut conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let items: Vec<MessageBatchItem> = vec![];
    let inserted = batch_insert_messages(&mut conn, &account_id, mailbox, &items, 100).unwrap();
    assert_eq!(inserted, 0);
}

#[test]
fn test_upsert_message() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: Some(r#"["recipient@example.com"]"#.to_string()),
        subject: Some("Test Subject".to_string()),
        snippet: Some("Test snippet".to_string()),
        flags_json: Some(r#"["\Seen"]"#.to_string()),
        structure_json: None,
    };

    let rowid = upsert_message(&conn, &account_id, mailbox, &data).unwrap();
    assert!(rowid > 0);

    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].subject, Some("Test Subject".to_string()));
}

#[test]
fn test_fetch_headers_with_anchor() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    for i in 1..=10 {
        let data = MessageUpsertData {
            uid: i,
            message_id: Some(format!("msg{}@example.com", i)),
            internal_date: Utc::now(),
            from: Some("sender@example.com".to_string()),
            to_json: Some(r#"["recipient@example.com"]"#.to_string()),
            subject: Some(format!("Subject {}", i)),
            snippet: None,
            flags_json: Some(r#"[]"#.to_string()),
            structure_json: None,
        };
        upsert_message(&conn, &account_id, mailbox, &data).unwrap();
    }

    let headers = fetch_headers(&conn, &account_id, mailbox, Some(5), 3).unwrap();
    assert!(headers.len() <= 3);
    for header in &headers {
        assert!(header.uid > 5);
    }
}

#[test]
fn test_fetch_message_full() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";
    let uid = 42;

    let data = MessageUpsertData {
        uid,
        message_id: Some("full@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: Some(r#"["recipient@example.com"]"#.to_string()),
        subject: Some("Full Message".to_string()),
        snippet: Some("Snippet text".to_string()),
        flags_json: Some(r#"["\Seen"]"#.to_string()),
        structure_json: None,
    };
    upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    let message = fetch_message_full(&conn, &account_id, mailbox, uid).unwrap();
    assert!(message.is_some());
    let msg = message.unwrap();
    assert_eq!(msg.header.uid, uid);
    assert_eq!(msg.header.subject, Some("Full Message".to_string()));
}

#[test]
fn test_fetch_message_full_nonexistent() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let message = fetch_message_full(&conn, &account_id, mailbox, 999).unwrap();
    assert!(message.is_none());
}

#[test]
fn test_update_message_flags() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: None,
        subject: Some("Test".to_string()),
        snippet: None,
        flags_json: Some(r#"[]"#.to_string()),
        structure_json: None,
    };
    upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    let updated = update_message_flags(&conn, &account_id, mailbox, &[1], Some(&["\\Seen"]), None).unwrap();
    assert_eq!(updated, 1);

    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert!(headers[0].flags.contains(&"\\Seen".to_string()));
}

#[test]
fn test_mark_read() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: None,
        subject: Some("Test".to_string()),
        snippet: None,
        flags_json: Some(r#"[]"#.to_string()),
        structure_json: None,
    };
    upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    mark_read(&conn, &account_id, mailbox, &[1], true).unwrap();
    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert!(headers[0].flags.contains(&"\\Seen".to_string()));

    mark_read(&conn, &account_id, mailbox, &[1], false).unwrap();
    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert!(!headers[0].flags.contains(&"\\Seen".to_string()));
}

#[test]
fn test_move_to_trash() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: None,
        subject: Some("Test".to_string()),
        snippet: None,
        flags_json: Some(r#"[]"#.to_string()),
        structure_json: None,
    };
    upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    move_to_trash(&conn, &account_id, mailbox, &[1]).unwrap();
    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert!(headers[0].flags.contains(&"\\Deleted".to_string()));
}

#[test]
fn test_sync_message_attachments_flag() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: None,
        subject: Some("Test".to_string()),
        snippet: None,
        flags_json: Some(r#"[]"#.to_string()),
        structure_json: None,
    };
    let message_table_id = upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    conn.execute(
        "INSERT INTO attachments (message_table_id, part_id, filename, mime_type, size) VALUES (?, ?, ?, ?, ?)",
        (message_table_id, "1", "test.pdf", "application/pdf", 1024),
    ).unwrap();

    sync_message_attachments_flag(message_table_id, &conn).unwrap();

    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert!(headers[0].has_attachments);
}

#[test]
fn test_refresh_all_attachments_flags() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: None,
        subject: Some("Test".to_string()),
        snippet: None,
        flags_json: Some(r#"[]"#.to_string()),
        structure_json: None,
    };
    let message_table_id = upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    conn.execute(
        "INSERT INTO attachments (message_table_id, part_id, filename, mime_type, size) VALUES (?, ?, ?, ?, ?)",
        (message_table_id, "1", "test.pdf", "application/pdf", 1024),
    ).unwrap();

    refresh_all_attachments_flags(&conn).unwrap();

    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert!(headers[0].has_attachments);
}

// ============================================================================
// Drafts tests
// ============================================================================

#[test]
fn test_save_and_load_draft() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let draft = Draft {
        id: "draft-1".to_string(),
        account_id: account_id.clone(),
        subject: Some("Test Draft".to_string()),
        body: Some("<p>Draft body</p>".to_string()),
        to: vec!["recipient@example.com".to_string()],
        cc: vec![],
        bcc: vec![],
        attachments: vec![],
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };

    save_draft(&conn, &draft).unwrap();

    let loaded = load_draft(&conn, "draft-1").unwrap();
    assert!(loaded.is_some());
    let loaded_draft = loaded.unwrap();
    assert_eq!(loaded_draft.id, "draft-1");
    assert_eq!(loaded_draft.subject, Some("Test Draft".to_string()));
    assert_eq!(loaded_draft.to, vec!["recipient@example.com".to_string()]);
}

#[test]
fn test_load_nonexistent_draft() {
    let conn = init_temp_db();

    let loaded = load_draft(&conn, "nonexistent").unwrap();
    assert!(loaded.is_none());
}

#[test]
fn test_list_drafts() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    for i in 1..=3 {
        let draft = Draft {
            id: format!("draft-{}", i),
            account_id: account_id.clone(),
            subject: Some(format!("Draft {}", i)),
            body: None,
            to: vec![],
            cc: vec![],
            bcc: vec![],
            attachments: vec![],
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        };
        save_draft(&conn, &draft).unwrap();
    }

    let drafts = list_drafts(&conn, &account_id).unwrap();
    assert_eq!(drafts.len(), 3);
}

#[test]
fn test_delete_draft() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let draft = Draft {
        id: "draft-to-delete".to_string(),
        account_id: account_id.clone(),
        subject: Some("To Delete".to_string()),
        body: None,
        to: vec![],
        cc: vec![],
        bcc: vec![],
        attachments: vec![],
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    save_draft(&conn, &draft).unwrap();

    delete_draft(&conn, "draft-to-delete").unwrap();

    let loaded = load_draft(&conn, "draft-to-delete").unwrap();
    assert!(loaded.is_none());
}

#[test]
fn test_save_draft_with_attachments() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let draft = Draft {
        id: "draft-with-att".to_string(),
        account_id: account_id.clone(),
        subject: Some("Draft with attachments".to_string()),
        body: None,
        to: vec![],
        cc: vec![],
        bcc: vec![],
        attachments: vec![
            DraftAttachment {
                id: "att-1".to_string(),
                filename: "file1.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                size: 1024,
                hash: "hash1".to_string(),
                path: "/path/to/file1".to_string(),
                cid: None,
                inline: false,
            },
            DraftAttachment {
                id: "att-2".to_string(),
                filename: "image.png".to_string(),
                content_type: "image/png".to_string(),
                size: 2048,
                hash: "hash2".to_string(),
                path: "/path/to/image".to_string(),
                cid: Some("cid123@postail.local".to_string()),
                inline: true,
            },
        ],
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };

    save_draft(&conn, &draft).unwrap();

    let loaded = load_draft(&conn, "draft-with-att").unwrap().unwrap();
    assert_eq!(loaded.attachments.len(), 2);
    assert_eq!(loaded.attachments[0].filename, "file1.pdf");
    assert_eq!(loaded.attachments[1].inline, true);
}

// ============================================================================
// Outbox tests
// ============================================================================

#[test]
fn test_enqueue_message() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let eml_path = "/path/to/test.eml";
    let outbox_id = enqueue_message(&conn, &account_id, eml_path).unwrap();

    assert!(!outbox_id.is_empty());

    let items = list_outbox(&conn, &account_id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, outbox_id);
    assert_eq!(items[0].status, "PENDING");
}

#[test]
fn test_list_outbox_empty() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let items = list_outbox(&conn, &account_id).unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_list_outbox_multiple() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    for i in 1..=5 {
        enqueue_message(&conn, &account_id, &format!("/path/to/test{}.eml", i)).unwrap();
    }

    let items = list_outbox(&conn, &account_id).unwrap();
    assert_eq!(items.len(), 5);
}

// ============================================================================
// Contacts tests
// ============================================================================

#[test]
fn test_upsert_contact() {
    let conn = init_temp_db();

    upsert_contact(&conn, "test@example.com", Some("Test User")).unwrap();

    let contacts = search_contacts(&conn, "test", 10).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email, "test@example.com");
    assert_eq!(contacts[0].name, Some("Test User".to_string()));
    assert_eq!(contacts[0].frequency, 1);
}

#[test]
fn test_upsert_contact_increment_frequency() {
    let conn = init_temp_db();

    upsert_contact(&conn, "test@example.com", Some("Test User")).unwrap();
    upsert_contact(&conn, "test@example.com", Some("Test User")).unwrap();
    upsert_contact(&conn, "test@example.com", Some("Test User")).unwrap();

    let contacts = search_contacts(&conn, "test", 10).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].frequency, 3);
}

#[test]
fn test_upsert_from_address_string_with_name() {
    let conn = init_temp_db();

    upsert_from_address_string(&conn, "John Doe <john@example.com>").unwrap();

    let contacts = search_contacts(&conn, "john", 10).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email, "john@example.com");
    assert_eq!(contacts[0].name, Some("John Doe".to_string()));
}

#[test]
fn test_upsert_from_address_string_without_name() {
    let conn = init_temp_db();

    upsert_from_address_string(&conn, "jane@example.com").unwrap();

    let contacts = search_contacts(&conn, "jane", 10).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email, "jane@example.com");
    assert_eq!(contacts[0].name, None);
}

#[test]
fn test_upsert_from_address_string_with_quotes() {
    let conn = init_temp_db();

    upsert_from_address_string(&conn, "\"Smith, Alice\" <alice@example.com>").unwrap();

    let contacts = search_contacts(&conn, "alice", 10).unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email, "alice@example.com");
    assert_eq!(contacts[0].name, Some("Smith, Alice".to_string()));
}

#[test]
fn test_search_contacts() {
    let conn = init_temp_db();

    upsert_contact(&conn, "alice@example.com", Some("Alice")).unwrap();
    upsert_contact(&conn, "bob@example.com", Some("Bob")).unwrap();
    upsert_contact(&conn, "charlie@example.com", Some("Charlie")).unwrap();

    let results = search_contacts(&conn, "alice", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].email, "alice@example.com");

    let results = search_contacts(&conn, "example", 10).unwrap();
    assert!(results.len() >= 1);
}

#[test]
fn test_search_contacts_with_limit() {
    let conn = init_temp_db();

    for i in 1..=10 {
        upsert_contact(&conn, &format!("user{}@example.com", i), Some(&format!("User {}", i))).unwrap();
    }

    let results = search_contacts(&conn, "user", 5).unwrap();
    assert!(results.len() <= 5);
}

#[test]
fn test_search_contacts_frequency_order() {
    let conn = init_temp_db();

    upsert_contact(&conn, "frequent@example.com", Some("Frequent")).unwrap();
    upsert_contact(&conn, "frequent@example.com", Some("Frequent")).unwrap();
    upsert_contact(&conn, "frequent@example.com", Some("Frequent")).unwrap();

    upsert_contact(&conn, "rare@example.com", Some("Rare")).unwrap();

    let results = search_contacts(&conn, "example", 10).unwrap();
    assert_eq!(results[0].email, "frequent@example.com");
}

// ============================================================================
// Attachments tests
// ============================================================================

#[test]
fn test_add_attachment_bytes() {
    let bytes = vec![0u8, 1, 2, 3, 4, 5];
    let filename = "test.bin".to_string();
    let content_type = "application/octet-stream".to_string();

    let attachment = add_attachment_bytes(bytes, filename.clone(), content_type.clone()).unwrap();

    assert_eq!(attachment.filename, filename);
    assert_eq!(attachment.content_type, content_type);
    assert_eq!(attachment.size, 6);
    assert!(!attachment.inline);
    assert!(attachment.cid.is_none());
    assert!(!attachment.hash.is_empty());

    // Cleanup
    remove_attachment(&attachment.id).unwrap();
}

#[test]
fn test_add_inline_attachment_bytes() {
    let bytes = vec![0xFF, 0xD8, 0xFF]; // JPEG magic bytes
    let filename = "image.jpg".to_string();
    let content_type = "image/jpeg".to_string();

    let attachment = add_inline_attachment_bytes(bytes, filename.clone(), content_type.clone()).unwrap();

    assert_eq!(attachment.filename, filename);
    assert_eq!(attachment.content_type, content_type);
    assert_eq!(attachment.size, 3);
    assert!(attachment.inline);
    assert!(attachment.cid.is_some());
    assert!(attachment.cid.unwrap().ends_with("@postail.local"));

    // Cleanup
    remove_attachment(&attachment.id).unwrap();
}

#[test]
fn test_remove_attachment() {
    let bytes = vec![1, 2, 3];
    let attachment = add_attachment_bytes(bytes, "test.bin".to_string(), "application/octet-stream".to_string()).unwrap();

    let path = std::path::Path::new(&attachment.path);
    assert!(path.exists());

    remove_attachment(&attachment.id).unwrap();

    assert!(!path.exists());
}

#[test]
fn test_add_attachment_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, b"Hello, World!").unwrap();

    let attachment = add_attachment(file_path.to_str().unwrap()).unwrap();

    assert_eq!(attachment.filename, "test_file.txt");
    assert_eq!(attachment.content_type, "text/plain");
    assert_eq!(attachment.size, 13);

    // Cleanup
    remove_attachment(&attachment.id).unwrap();
}

#[test]
fn test_add_attachment_nonexistent_file() {
    let result = add_attachment("/nonexistent/path/file.txt");
    assert!(result.is_err());
}

// ============================================================================
// Edge cases and stress tests
// ============================================================================

#[test]
fn test_batch_insert_messages_with_transaction_boundary() {
    let mut conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let mut items = Vec::new();
    for i in 1..=150 {
        items.push(MessageBatchItem {
            uid: i,
            message_id: Some(format!("msg{}@example.com", i)),
            internal_date: Utc::now(),
            from: Some("sender@example.com".to_string()),
            to: vec!["recipient@example.com".to_string()],
            subject: Some(format!("Subject {}", i)),
            snippet: None,
            flags: vec![],
            structure_json: None,
        });
    }

    let inserted = batch_insert_messages(&mut conn, &account_id, mailbox, &items, 50).unwrap();
    assert_eq!(inserted, 150);
}

#[test]
fn test_messages_with_special_characters() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);
    let mailbox = "INBOX";

    let data = MessageUpsertData {
        uid: 1,
        message_id: Some("test@example.com".to_string()),
        internal_date: Utc::now(),
        from: Some("sender@example.com".to_string()),
        to_json: None,
        subject: Some("Test with émojis 🎉 and special chars: <>&\"'".to_string()),
        snippet: Some("Snippet with 日本語 and Кириллица".to_string()),
        flags_json: Some(r#"[]"#.to_string()),
        structure_json: None,
    };
    upsert_message(&conn, &account_id, mailbox, &data).unwrap();

    let headers = fetch_headers(&conn, &account_id, mailbox, None, 10).unwrap();
    assert_eq!(headers[0].subject, Some("Test with émojis 🎉 and special chars: <>&\"'".to_string()));
}

#[test]
fn test_draft_update_changes_updated_at() {
    let conn = init_temp_db();
    let security = test_manager();
    let account_id = create_test_account(&conn, &security);

    let draft1 = Draft {
        id: "draft-1".to_string(),
        account_id: account_id.clone(),
        subject: Some("Original".to_string()),
        body: None,
        to: vec![],
        cc: vec![],
        bcc: vec![],
        attachments: vec![],
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    save_draft(&conn, &draft1).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    let draft2 = Draft {
        id: "draft-1".to_string(),
        account_id: account_id.clone(),
        subject: Some("Updated".to_string()),
        body: None,
        to: vec![],
        cc: vec![],
        bcc: vec![],
        attachments: vec![],
        created_at: draft1.created_at,
        updated_at: Utc::now().timestamp(),
    };
    save_draft(&conn, &draft2).unwrap();

    let loaded = load_draft(&conn, "draft-1").unwrap().unwrap();
    assert!(loaded.updated_at >= draft2.updated_at);
    assert_eq!(loaded.subject, Some("Updated".to_string()));
}