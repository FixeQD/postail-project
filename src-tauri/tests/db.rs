use chrono::Utc;

use postail_project_lib::db::{
    add_account, init_db, list_accounts, remove_account, AccountInput, Credentials, ImapConfig,
    OAuthCredentials, PasswordCredentials, SmtpConfig,
};
use postail_project_lib::security::SecurityManager;

#[test]
fn test_init_db() {
    let conn = init_db().unwrap();
    // Check if table exists
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='accounts'")
        .unwrap();
    let exists: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
    assert_eq!(exists, Some("accounts".to_string()));
}

#[test]
fn test_add_account_password() {
    let conn = init_db().unwrap();
    let security = test_manager();

    let input = AccountInput {
        name: "Test Account".to_string(),
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
    let conn = init_db().unwrap();
    let security = test_manager();

    let input = AccountInput {
        name: "OAuth Account".to_string(),
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
    let conn = init_db().unwrap();
    let security = test_manager();

    // Add two accounts
    let input1 = AccountInput {
        name: "Account 1".to_string(),
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
    let conn = init_db().unwrap();
    let security = test_manager();

    let input = AccountInput {
        name: "To Remove".to_string(),
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
