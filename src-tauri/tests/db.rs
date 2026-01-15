use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use tempfile::NamedTempFile;

use postail_project_lib::db::tables::create_tables;
use postail_project_lib::db::{
    add_account, list_accounts, remove_account, AccountInput, Credentials, ImapConfig,
    OAuthCredentials, PasswordCredentials, SmtpConfig,
};
use postail_project_lib::security::SecurityManager;

fn init_temp_db() -> Connection {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    fs::remove_file(db_path).unwrap();
    let conn = Connection::open(db_path).unwrap();
    create_tables(&conn).unwrap();
    conn
}

#[test]
fn test_init_db() {
    let conn = init_temp_db();
    // Check if table exists
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='accounts'")
        .unwrap();
    let exists: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(exists, Some("accounts".to_string()));
}

#[test]
fn test_add_account_password() {
    let conn = init_temp_db();
    let security = test_manager();

    let input = AccountInput {
        name: "Test Account".to_string(),
        email: "user@example.com".to_string(),
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
            username: "user@example.com".to_string(),
            password: "secret".to_string(),
        }),
    };

    let meta = add_account(&conn, input, &security).unwrap();
    assert_eq!(meta.name, "Test Account");
    assert_eq!(meta.email, "user@example.com");
    assert_eq!(meta.auth_type, "password");
    assert_eq!(meta.provider_type, "generic");
    assert!(meta.created_at <= Utc::now());
}

#[test]
fn test_add_account_oauth() {
    let conn = init_temp_db();
    let security = test_manager();

    let input = AccountInput {
        name: "OAuth Account".to_string(),
        email: "oauth@example.com".to_string(),
        auth_type: "oauth2".to_string(),
        imap_config: ImapConfig {
            host: "imap.gmail.com".to_string(),
            port: 993,
            tls: true,
        },
        smtp_config: SmtpConfig {
            host: "smtp.gmail.com".to_string(),
            port: 587,
            tls: true,
        },
        credentials: Credentials::OAuth(OAuthCredentials {
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_in: 3600,
        }),
    };

    let meta = add_account(&conn, input, &security).unwrap();
    assert_eq!(meta.name, "OAuth Account");
    assert_eq!(meta.email, "oauth@example.com"); // placeholder
    assert_eq!(meta.auth_type, "oauth2");
    assert_eq!(meta.provider_type, "gmail");
}

#[test]
fn test_list_accounts() {
    let conn = init_temp_db();
    let security = test_manager();

    // Add two accounts
    let input1 = AccountInput {
        name: "Account 1".to_string(),
        email: "user1@example.com".to_string(),
        auth_type: "password".to_string(),
        imap_config: ImapConfig {
            host: "imap1.com".to_string(),
            port: 993,
            tls: true,
        },
        smtp_config: SmtpConfig {
            host: "smtp1.com".to_string(),
            port: 587,
            tls: true,
        },
        credentials: Credentials::Password(PasswordCredentials {
            username: "user1@example.com".to_string(),
            password: "pass1".to_string(),
        }),
    };
    add_account(&conn, input1, &security).unwrap();

    let input2 = AccountInput {
        name: "Account 2".to_string(),
        email: "user2@example.com".to_string(),
        auth_type: "oauth2".to_string(),
        imap_config: ImapConfig {
            host: "imap2.com".to_string(),
            port: 993,
            tls: true,
        },
        smtp_config: SmtpConfig {
            host: "smtp2.com".to_string(),
            port: 587,
            tls: true,
        },
        credentials: Credentials::OAuth(OAuthCredentials {
            access_token: "token2".to_string(),
            refresh_token: None,
            expires_in: 3600,
        }),
    };
    add_account(&conn, input2, &security).unwrap();

    let accounts = list_accounts(&conn).unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].name, "Account 1");
    assert_eq!(accounts[1].name, "Account 2");
}

#[test]
fn test_remove_account() {
    let conn = init_temp_db();
    let security = test_manager();

    let input = AccountInput {
        name: "To Remove".to_string(),
        email: "user@example.com".to_string(),
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
            username: "user@example.com".to_string(),
            password: "secret".to_string(),
        }),
    };

    let meta = add_account(&conn, input, &security).unwrap();
    let id = meta.id.clone();

    remove_account(&conn, &id).unwrap();

    let accounts = list_accounts(&conn).unwrap();
    assert_eq!(accounts.len(), 0);
}

pub fn test_manager() -> SecurityManager {
    use std::env::temp_dir;
    use std::sync::Arc;

    use postail_project_lib::security::{
        manager::SecurityManager,
        master_key::MasterKey,
        stores::{argon2::Argon2Store, StorageTier},
    };

    let temp_path = temp_dir().join("test_postail_key");
    let store = Argon2Store::new(temp_path, "test_passphrase".to_string());
    let mut manager = SecurityManager::with_store(Arc::new(store), StorageTier::Passphrase);
    // Initialize with fixed key for tests
    let fixed_key = MasterKey::from_bytes(&[0u8; 32]).unwrap();
    manager.initialize_with_key(fixed_key).unwrap();
    manager.unlock().unwrap();
    manager
}

#[test]
fn test_fts_search() {
    use postail_project_lib::db::tables::create_fts_triggers;
    use postail_project_lib::db::{
        add_account, escape_fts_query, search_messages, upsert_message, AccountInput, Credentials,
        ImapConfig, PasswordCredentials, SmtpConfig,
    };

    let conn = init_temp_db();
    create_fts_triggers(&conn).unwrap();
    let security = test_manager();
    let mailbox = "INBOX";
    let uid1 = 1;
    let uid2 = 2;

    let account = add_account(
        &conn,
        AccountInput {
            name: "Test Account FTS".to_string(),
            email: "test-fts@example.com".to_string(),
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
                username: "test-fts@example.com".to_string(),
                password: "secret".to_string(),
            }),
        },
        &security,
    )
    .unwrap();
    let account_id = account.id;

    let created_at = Utc::now();
    upsert_message(
        &conn,
        &account_id,
        mailbox,
        uid1,
        Some("msg1@example.com"),
        created_at,
        Some("sender1@example.com"),
        Some("recipient@example.com"),
        Some("Test subject with keyword"),
        Some("Body text with searchable content"),
        Some(r#"["\Seen", "\Flagged"]"#),
        Some("structure"),
    )
    .unwrap();

    upsert_message(
        &conn,
        &account_id,
        mailbox,
        uid2,
        Some("msg2@example.com"),
        created_at,
        Some("sender2@example.com"),
        Some("recipient@example.com"),
        Some("Another subject"),
        Some("Different body content"),
        Some(r#"["\Seen"]"#),
        Some("structure"),
    )
    .unwrap();

    let results = search_messages(&conn, Some(&account_id), Some(mailbox), "keyword", 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].subject,
        Some("Test subject with keyword".to_string())
    );

    let escaped = escape_fts_query("test -query");
    assert!(escaped.contains(r"\-"));
}

#[test]
fn test_uidvalidity_mismatch() {
    use postail_project_lib::db::{
        add_account, check_uidvalidity, upsert_mailbox, AccountInput, Credentials, ImapConfig,
        PasswordCredentials, SmtpConfig,
    };

    let conn = init_temp_db();
    let security = test_manager();
    let mailbox = "INBOX";

    let account = add_account(
        &conn,
        AccountInput {
            name: "Test Account UID".to_string(),
            email: "test-uid@example.com".to_string(),
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
                username: "test-uid@example.com".to_string(),
                password: "secret".to_string(),
            }),
        },
        &security,
    )
    .unwrap();
    let account_id = account.id;

    upsert_mailbox(
        &conn,
        &account_id,
        &postail_project_lib::db::Mailbox {
            name: mailbox.to_string(),
            uid_validity: Some(1),
            highest_modseq: None,
            last_synced_uid: None,
        },
    )
    .unwrap();

    let should_resync = check_uidvalidity(&conn, &account_id, mailbox, 2).unwrap();
    assert!(should_resync);

    upsert_mailbox(
        &conn,
        &account_id,
        &postail_project_lib::db::Mailbox {
            name: mailbox.to_string(),
            uid_validity: Some(2),
            highest_modseq: None,
            last_synced_uid: None,
        },
    )
    .unwrap();

    let should_resync2 = check_uidvalidity(&conn, &account_id, mailbox, 2).unwrap();
    assert!(!should_resync2);
}

#[test]
fn test_concurrent_access() {
    use postail_project_lib::db::{
        add_account, list_accounts, AccountInput, Credentials, ImapConfig, PasswordCredentials,
        SmtpConfig,
    };
    use std::sync::{Arc, Mutex};
    use std::thread;

    let conn: Arc<Mutex<Connection>> = Arc::new(Mutex::new(init_temp_db()));
    let security = Arc::new(Mutex::new(test_manager()));

    let mut handles = vec![];

    for i in 0..5 {
        let conn_clone = Arc::clone(&conn);
        let security_clone = Arc::clone(&security);

        handles.push(thread::spawn(move || {
            let input = AccountInput {
                name: format!("Concurrent Account {}", i),
                email: format!("user{}@example.com", i),
                auth_type: "password".to_string(),
                imap_config: ImapConfig {
                    host: format!("imap{}.com", i),
                    port: 993,
                    tls: true,
                },
                smtp_config: SmtpConfig {
                    host: format!("smtp{}.com", i),
                    port: 587,
                    tls: true,
                },
                credentials: Credentials::Password(PasswordCredentials {
                    username: format!("user{}@example.com", i),
                    password: format!("pass{}", i),
                }),
            };

            let conn_guard = conn_clone.lock().unwrap();
            let security_guard = security_clone.lock().unwrap();
            add_account(&*conn_guard, input, &*security_guard).unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let conn_guard = conn.lock().unwrap();
    let accounts = list_accounts(&*conn_guard).unwrap();
    assert_eq!(accounts.len(), 5);
}

#[test]
fn test_calculate_backoff() {
    use postail_project_lib::db::calculate_backoff;

    let backoff1 = calculate_backoff(0);
    assert!(backoff1 > 0);

    let backoff2 = calculate_backoff(1);
    assert!(backoff2 > backoff1);

    let backoff3 = calculate_backoff(10);
    assert!(backoff3 > 0);
}

#[test]
fn test_attachment_cache_lru() {
    use postail_project_lib::db::enforce_attachment_cache_limit;

    let cache_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("postail")
        .join("attachments");
    std::fs::create_dir_all(&cache_dir).unwrap();

    for i in 0..5 {
        let path = cache_dir.join(format!("test_file_{}.enc", i));
        let content = format!("attachment content {}", i).into_bytes();
        std::fs::write(&path, &content).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let total_size_before: u64 = cache_dir
        .read_dir()
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name().to_string_lossy().starts_with("test_file_"))
        .map(|e| e.metadata().unwrap().len())
        .sum();

    assert!(total_size_before > 0, "Files should exist in cache dir");

    let removed = enforce_attachment_cache_limit(10).unwrap();

    let total_size_after: u64 = cache_dir
        .read_dir()
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name().to_string_lossy().starts_with("test_file_"))
        .map(|e| e.metadata().unwrap().len())
        .sum();

    assert!(
        total_size_after <= total_size_before,
        "Size should not increase"
    );
    assert!(
        removed > 0,
        "Some files should have been removed, removed: {}",
        removed
    );

    for i in 0..5 {
        let path = cache_dir.join(format!("test_file_{}.enc", i));
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn test_migration_up_down() {
    use postail_project_lib::db::backup::{run_migrations, MIGRATIONS};
    use rusqlite::Connection;
    use std::fs;
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path();
    fs::remove_file(db_path).unwrap();
    let conn = Connection::open(db_path).unwrap();

    let initial_version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .ok()
        .unwrap_or(0);

    run_migrations(&conn).unwrap();

    let final_version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert!(final_version > initial_version);
    assert_eq!(final_version, MIGRATIONS.len() as i32);

    conn.execute(
        "DELETE FROM schema_migrations WHERE version < ?",
        [final_version],
    )
    .unwrap();

    let version_after_cleanup: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(version_after_cleanup, final_version);
}

#[test]
fn test_performance_large_datasets() {
    use postail_project_lib::db::tables::create_indexes;
    use postail_project_lib::db::{
        add_account, upsert_message, AccountInput, Credentials, ImapConfig, PasswordCredentials,
        SmtpConfig,
    };
    use std::time::Instant;

    let mut conn = init_temp_db();
    create_indexes(&conn).unwrap();
    let security = test_manager();

    let account = add_account(
        &conn,
        AccountInput {
            name: "Perf Test Account".to_string(),
            email: "perf@example.com".to_string(),
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
                username: "perf@example.com".to_string(),
                password: "secret".to_string(),
            }),
        },
        &security,
    )
    .unwrap();
    let account_id = account.id;
    let mailbox = "INBOX";
    let created_at = Utc::now();

    let start = Instant::now();
    let batch_size = 100;
    let num_batches = 100;
    let total_messages = batch_size * num_batches;

    for batch in 0..num_batches {
        let batch_start = batch * batch_size;
        let tx = conn.transaction().unwrap();
        for i in 0..batch_size {
            let uid = (batch_start + i + 1) as u32;
            upsert_message(
                &tx,
                &account_id,
                mailbox,
                uid,
                Some(&format!("msg{}@example.com", uid)),
                created_at,
                Some(&format!("sender{}@example.com", uid)),
                Some("recipient@example.com"),
                Some(&format!("Subject {}", uid)),
                Some(&format!("Body content for message {}", uid)),
                Some(r#"["\Seen"]"#),
                None,
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }
    let insert_time = start.elapsed();
    println!("Inserted {} messages in {:?}", total_messages, insert_time);
    assert!(
        insert_time.as_secs() < 30,
        "Insert took too long: {:?}",
        insert_time
    );

    let query_start = Instant::now();
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM messages WHERE account_id = ? AND mailbox = ?")
        .unwrap();
    let count: i64 = stmt
        .query_row(params![account_id, mailbox], |row| row.get(0))
        .unwrap();
    let query_time = query_start.elapsed();
    println!("Count query took: {:?}", query_time);
    assert_eq!(count, total_messages as i64);
    assert!(
        query_time.as_millis() < 100,
        "Query took too long: {:?}",
        query_time
    );

    let fetch_start = Instant::now();
    let mut stmt = conn
        .prepare("SELECT subject FROM messages WHERE account_id = ? AND mailbox = ? ORDER BY uid DESC LIMIT 100")
        .unwrap();
    let subjects: Vec<String> = stmt
        .query_map(params![account_id, mailbox], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    let fetch_time = fetch_start.elapsed();
    println!("Fetch 100 messages took: {:?}", fetch_time);
    assert_eq!(subjects.len(), 100);
    assert!(
        fetch_time.as_millis() < 200,
        "Fetch took too long: {:?}",
        fetch_time
    );
}
