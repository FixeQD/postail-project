pub mod db;
pub mod error;
pub mod globals;
pub mod imap;
pub mod oauth;
pub mod protocol;
pub mod security;
pub mod smtp;
pub mod utils;

use crate::db::accounts::{
    add_account as db_add_account, list_accounts as db_list_accounts,
    remove_account as db_remove_account,
};
use crate::db::{
    enqueue_message as db_enqueue_message, export_backup as db_export_backup,
    fetch_headers as db_fetch_headers, fetch_mailboxes as db_fetch_mailboxes,
    fetch_message_full as db_fetch_message_full, import_backup as db_import_backup, init_db,
    list_outbox as db_list_outbox, AccountInput, AccountMeta, Credentials, ImapConfig, MailHeader,
    Mailbox, MessageFull, OAuthCredentials, OutboxItem, SearchResult, SmtpConfig, SyncStatusEnum,
};
use crate::imap::ImapManager;
use crate::security::stores::SecretStore;
use crate::security::SecurityManager;
use crate::smtp::SmtpManager;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref DB_CONN: Arc<Mutex<Connection>> = Arc::new(Mutex::new(init_db().unwrap()));
    static ref SECURITY: Arc<Mutex<SecurityManager>> =
        Arc::new(Mutex::new(SecurityManager::new().unwrap()));
    static ref IMAP_MANAGER: Arc<Mutex<ImapManager>> = Arc::new(Mutex::new(ImapManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )));
    static ref SMTP_MANAGER: Arc<Mutex<SmtpManager>> = Arc::new(Mutex::new(SmtpManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )));
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn add_account(input: AccountInput) -> Result<AccountMeta, String> {
    let conn = DB_CONN.lock().unwrap();
    let security = SECURITY.lock().unwrap();
    db_add_account(&*conn, input, &*security).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_accounts() -> Result<Vec<AccountMeta>, String> {
    let conn = DB_CONN.lock().unwrap();
    db_list_accounts(&*conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_account(id: String) -> Result<(), String> {
    let conn = DB_CONN.lock().unwrap();
    db_remove_account(&*conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_oauth_flow(provider: String) -> Result<String, String> {
    let provider = match provider.as_str() {
        "gmail" => oauth::Provider::Gmail,
        "outlook" => oauth::Provider::Outlook,
        _ => return Err("Unknown provider".to_string()),
    };
    match oauth::start_oauth_flow(provider) {
        Ok(url) => Ok(url),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn complete_oauth_flow(code: String, state: String) -> Result<AccountMeta, String> {
    let (provider, tokens) = match oauth::complete_oauth_flow(code, state).await {
        Ok(result) => result,
        Err(e) => return Err(e.to_string()),
    };

    let email = match provider {
        oauth::Provider::Gmail => {
            let client = reqwest::Client::new();
            let response = client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(&tokens.access_token)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                let status = response.status().to_string();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                println!(
                    "Failed to fetch Gmail user info. Status: {}, Body: {}",
                    status, body
                );
                return Err("Failed to fetch Gmail user info".to_string());
            }
            let user_info: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            user_info["email"]
                .as_str()
                .ok_or("No email in Gmail response")?
                .to_string()
        }
        oauth::Provider::Outlook => {
            let client = reqwest::Client::new();
            let response = client
                .get("https://graph.microsoft.com/v1.0/me")
                .bearer_auth(&tokens.access_token)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                return Err("Failed to fetch Outlook user info".to_string());
            }
            let user_info: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            user_info["mail"]
                .as_str()
                .or_else(|| user_info["userPrincipalName"].as_str())
                .ok_or("No email in Outlook response")?
                .to_string()
        }
    };

    let account_input = AccountInput {
        name: format!(
            "{} Account",
            match provider {
                oauth::Provider::Gmail => "Gmail",
                oauth::Provider::Outlook => "Outlook",
            }
        ),
        email,
        auth_type: "oauth2".to_string(),
        credentials: Credentials::OAuth(OAuthCredentials {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        }),
        imap_config: match provider {
            oauth::Provider::Gmail => ImapConfig {
                host: "imap.gmail.com".to_string(),
                port: 993,
                tls: true,
            },
            oauth::Provider::Outlook => ImapConfig {
                host: "outlook.office365.com".to_string(),
                port: 993,
                tls: true,
            },
        },
        smtp_config: match provider {
            oauth::Provider::Gmail => SmtpConfig {
                host: "smtp.gmail.com".to_string(),
                port: 587,
                tls: true,
            },
            oauth::Provider::Outlook => SmtpConfig {
                host: "smtp-mail.outlook.com".to_string(),
                port: 587,
                tls: true,
            },
        },
    };

    let conn = DB_CONN.lock().unwrap();
    let security = SECURITY.lock().unwrap();
    let account = db_add_account(&*conn, account_input, &*security).map_err(|e| e.to_string())?;
    Ok(account)
}

#[derive(serde::Serialize)]
pub struct SecurityOptions {
    tpm_available: bool,
    keyring_available: bool,
    argon2_available: bool,
}

#[tauri::command]
async fn check_security_options() -> Result<SecurityOptions, String> {
    let tpm_available = crate::security::stores::tpm::get_tpm_store().is_some();

    let keyring_available = crate::security::stores::keyring::KeyringStore::new()
        .map(|k| k.is_available())
        .unwrap_or(false);

    Ok(SecurityOptions {
        tpm_available,
        keyring_available,
        argon2_available: true,
    })
}

#[tauri::command]
async fn initialize_security(method: String, passphrase: Option<String>) -> Result<(), String> {
    match method.as_str() {
        "tpm" => {
            println!("Initializing TPM security");

            if let Some(tpm_store) = crate::security::stores::tpm::get_tpm_store() {
                let new_security = crate::security::SecurityManager::with_store(
                    tpm_store.into(),
                    crate::security::stores::StorageTier::Tpm,
                );

                let mut security_guard = SECURITY.lock().unwrap();
                *security_guard = new_security;
                security_guard.initialize().map_err(|e| e.to_string())?;

                println!("TPM security initialized successfully");
                Ok(())
            } else {
                Err("TPM not available or not supported".to_string())
            }
        }
        "keyring" => {
            println!("Initializing keyring security");

            match crate::security::stores::keyring::KeyringStore::new() {
                Ok(store) if store.is_available() => {
                    let new_security = crate::security::SecurityManager::with_store(
                        std::sync::Arc::new(store),
                        crate::security::stores::StorageTier::Keyring,
                    );

                    let mut security_guard = SECURITY.lock().unwrap();
                    *security_guard = new_security;
                    security_guard.initialize().map_err(|e| e.to_string())?;

                    println!("Keyring security initialized successfully");
                    Ok(())
                }
                _ => Err("Keyring not available".to_string()),
            }
        }
        "argon2" => {
            let pass = passphrase.ok_or("Passphrase required for Argon2")?;
            println!("Initializing Argon2 security with passphrase");

            let storage_path = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("postail")
                .join("security");

            let builder =
                crate::security::manager::PassphraseSecurityBuilder::new(storage_path, pass);
            let new_security = builder.build();

            let mut security_guard = SECURITY.lock().unwrap();
            *security_guard = new_security;
            security_guard.initialize().map_err(|e| e.to_string())?;

            println!("Argon2 security initialized successfully");
            Ok(())
        }
        _ => return Err("Invalid security method".to_string()),
    }
}

#[tauri::command]
fn fetch_mailboxes(account_id: String) -> Result<Vec<Mailbox>, String> {
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.fetch_mailboxes_sync(&account_id)
}

#[tauri::command]
fn fetch_headers(
    account_id: String,
    mailbox: String,
    anchor: Option<u64>,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.fetch_headers_sync(&account_id, &mailbox, anchor.map(|a| a as u32), limit)
}

#[tauri::command]
fn fetch_message_full(
    account_id: String,
    mailbox: String,
    uid: u64,
) -> Result<Option<MessageFull>, String> {
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.fetch_message_full_sync(&account_id, &mailbox, uid as u32)
}

#[tauri::command]
fn start_sync(account_id: String) -> Result<(), String> {
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.start_sync(&account_id)
}

#[tauri::command]
fn stop_sync(_account_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn get_sync_status(_account_id: String) -> Result<SyncStatusEnum, String> {
    Ok(SyncStatusEnum::Idle)
}

#[tauri::command]
fn enqueue_message(account_id: String, raw_eml: String) -> Result<String, String> {
    let smtp = SMTP_MANAGER.lock().unwrap();
    smtp.enqueue_message(&account_id, raw_eml.as_bytes())
}

#[tauri::command]
fn list_outbox(account_id: String) -> Result<Vec<OutboxItem>, String> {
    let smtp = SMTP_MANAGER.lock().unwrap();
    smtp.list_outbox(&account_id)
}

#[tauri::command]
fn retry_sending(outbox_id: String) -> Result<(), String> {
    let smtp = SMTP_MANAGER.lock().unwrap();
    smtp.retry_sending(&outbox_id)
}

#[tauri::command]
fn cancel_sending(outbox_id: String) -> Result<(), String> {
    let smtp = SMTP_MANAGER.lock().unwrap();
    smtp.cancel_sending(&outbox_id)
}

#[tauri::command]
fn export_backup(passphrase: Option<String>) -> Result<String, String> {
    let conn = DB_CONN.lock().unwrap();
    let security = SECURITY.lock().unwrap();
    db_export_backup(&*conn, &*security, passphrase)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn import_backup(backup_path: String, passphrase: Option<String>) -> Result<(), String> {
    let mut conn = DB_CONN.lock().unwrap();
    let security = SECURITY.lock().unwrap();
    let path = std::path::PathBuf::from(backup_path);
    db_import_backup(&mut *conn, &*security, &path, passphrase).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port = portpicker::pick_unused_port().expect("failed to find a free port");
    globals::set_oauth_port(port);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            utils::oauth_server::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_oauth_flow,
            complete_oauth_flow,
            add_account,
            list_accounts,
            remove_account,
            check_security_options,
            initialize_security,
            fetch_mailboxes,
            fetch_headers,
            fetch_message_full,
            start_sync,
            stop_sync,
            get_sync_status,
            enqueue_message,
            list_outbox,
            retry_sending,
            cancel_sending,
            export_backup,
            import_backup
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
