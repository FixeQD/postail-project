pub mod db;
pub mod error;
pub mod globals;
pub mod imap;
pub mod maintenance;
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
    export_backup as db_export_backup, import_backup as db_import_backup, AccountInput,
    AccountMeta, Credentials, ImapConfig, MailHeader, Mailbox, MessageFull, OAuthCredentials,
    OutboxItem, SmtpConfig, SyncStatusEnum,
};
use crate::imap::ImapManager;
use crate::security::stores::SecretStore;
use crate::security::SecurityManager;
use crate::smtp::SmtpManager;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const HTTP_TIMEOUT_SECS: Duration = Duration::from_secs(30);

lazy_static! {
    pub static ref DB_CONN: Arc<Mutex<Option<Connection>>> = Arc::new(Mutex::new(None));
    pub static ref SECURITY: Arc<Mutex<SecurityManager>> = Arc::new(Mutex::new(
        SecurityManager::new().expect("Failed to initialize security")
    ));
    pub static ref IMAP_MANAGER: Arc<Mutex<ImapManager>> = Arc::new(Mutex::new(ImapManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )));
    pub static ref SMTP_MANAGER: Arc<Mutex<SmtpManager>> = Arc::new(Mutex::new(SmtpManager::new(
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
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let security = SECURITY.lock().unwrap();
    db_add_account(conn, input, &security).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_accounts() -> Result<Vec<AccountMeta>, String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db_list_accounts(conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_account(id: String) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db_remove_account(conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_oauth_flow(provider: String) -> Result<String, String> {
    let provider_kind =
        oauth::ProviderKind::parse(&provider).ok_or_else(|| "Unknown provider".to_string())?;
    let provider = oauth::Provider::from_kind(provider_kind);
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

    let email = match provider.kind {
        oauth::ProviderKind::Gmail => {
            let client = reqwest::Client::builder()
                .timeout(HTTP_TIMEOUT_SECS)
                .build()
                .map_err(|e| e.to_string())?;
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
        oauth::ProviderKind::Outlook => {
            let client = reqwest::Client::builder()
                .timeout(HTTP_TIMEOUT_SECS)
                .build()
                .map_err(|e| e.to_string())?;
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
        name: format!("{} Account", provider.kind.display_name()),
        email,
        auth_type: "oauth2".to_string(),
        credentials: Credentials::OAuth(OAuthCredentials {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        }),
        imap_config: ImapConfig {
            host: oauth::ProviderInfo::get(provider.kind)
                .imap_host
                .to_string(),
            port: 993,
            tls: true,
        },
        smtp_config: SmtpConfig {
            host: oauth::ProviderInfo::get(provider.kind)
                .smtp_host
                .to_string(),
            port: 587,
            tls: true,
        },
    };

    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let security = SECURITY.lock().unwrap();
    let account = db_add_account(conn, account_input, &security).map_err(|e| e.to_string())?;
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

#[derive(serde::Serialize)]
pub enum TpmStatus {
    Available,
    RequiresElevation,
    NotAvailable,
}

#[tauri::command]
async fn check_tpm_availability() -> Result<TpmStatus, String> {
    use crate::security::TpmAvailability;
    let initializer = crate::security::TpmInitializer::new();
    match initializer.check_availability() {
        TpmAvailability::Available => Ok(TpmStatus::Available),
        TpmAvailability::RequiresElevation => Ok(TpmStatus::RequiresElevation),
        TpmAvailability::NotAvailable => Ok(TpmStatus::NotAvailable),
    }
}

#[tauri::command]
fn get_app_initialization_status() -> String {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("postail");
    let db_path = data_dir.join("postail.db");

    if db_path.exists() {
        "Locked".to_string()
    } else {
        "SetupRequired".to_string()
    }
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

                // Init database after security
                let db = crate::db::init_db().map_err(|e| e.to_string())?;
                {
                    let mut db_guard = DB_CONN.lock().unwrap();
                    *db_guard = Some(db);
                }

                crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
                let smtp = SMTP_MANAGER.lock().unwrap();
                smtp.start_outbox_worker();

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

                    // Init database after security
                    let db = crate::db::init_db().map_err(|e| e.to_string())?;
                    {
                        let mut db_guard = DB_CONN.lock().unwrap();
                        *db_guard = Some(db);
                    }

                    crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
                    let smtp = SMTP_MANAGER.lock().unwrap();
                    smtp.start_outbox_worker();

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
                crate::security::manager::PassphraseSecurityBuilder::new(storage_path.clone(), pass);
            let new_security = builder.build();

            let mut security_guard = SECURITY.lock().unwrap();
            *security_guard = new_security;
            
            // Check if vault file exists - if yes, we MUST unlock, not initialize
            let vault_path = storage_path.join("master_key.sealed");
            let vault_exists = vault_path.exists();
            
            // Try to unlock if vault exists, otherwise initialize
            if vault_exists {
                match security_guard.unlock() {
                    Ok(()) => {
                        println!("Argon2: Unlocked existing vault");
                    }
                    Err(e) => {
                        eprintln!("Argon2: Failed to unlock vault (wrong password?): {}", e);
                        return Err("Wrong password - could not unlock vault".to_string());
                    }
                }
            } else {
                println!("Argon2: Creating new vault");
                security_guard.initialize().map_err(|e| e.to_string())?;
            }

            println!("Argon2 security initialized successfully");

            // Derive encryption key BEFORE releasing lock to avoid deadlock
            let master_key_raw = security_guard.get_master_key_raw();
            let encryption = crate::security::DbEncryption::derive_from_master_key(&master_key_raw)
                .map_err(|e| e.to_string())?;
            let hex_key = encryption.hex_key();
            
            drop(security_guard);

            // Check if database already exists
            let data_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("postail");
            let db_path = data_dir.join("postail.db");
            
            let db = if db_path.exists() {
                println!("Connecting to existing database...");
                crate::db::connect_db_with_key(&hex_key).map_err(|e| e.to_string())?
            } else {
                println!("Initializing new database...");
                let db = crate::db::init_db_with_key(&hex_key).map_err(|e| e.to_string())?;
                println!("Starting maintenance scheduler...");
                crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
                println!("Starting SMTP worker...");
                let smtp = SMTP_MANAGER.lock().unwrap();
                smtp.start_outbox_worker();
                println!("SMTP worker started");
                db
            };
            
            {
                let mut db_guard = DB_CONN.lock().unwrap();
                *db_guard = Some(db);
            }
            
            println!("Database ready!");
            println!("Initialization complete!");
            Ok(())
        }
        _ => Err("Invalid security method".to_string()),
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
    let anchor: Option<u32> = anchor
        .map(|a| a.try_into().map_err(|_| "Anchor too large".to_string()))
        .transpose()?;
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.fetch_headers_sync(&account_id, &mailbox, anchor, limit)
}

#[tauri::command]
fn fetch_message_full(
    account_id: String,
    mailbox: String,
    uid: u64,
) -> Result<Option<MessageFull>, String> {
    let uid_u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.fetch_message_full_sync(&account_id, &mailbox, uid_u32)
}

#[tauri::command]
fn start_sync(account_id: String) -> Result<(), String> {
    let imap = IMAP_MANAGER.lock().unwrap();
    imap.start_sync(&account_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_sync(_account_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn get_sync_status(account_id: String) -> Result<SyncStatusEnum, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(rt.block_on(async {
        use crate::imap::sync_status::SYNC_STATUS_MANAGER;
        SYNC_STATUS_MANAGER.get_status(&account_id).await
    }))
}

#[tauri::command]
fn search_messages(
    account_id: Option<String>,
    mailbox: Option<String>,
    query: String,
    limit: u32,
) -> Result<Vec<db::search::SearchResult>, String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db::search_messages(
        conn,
        account_id.as_deref(),
        mailbox.as_deref(),
        &query,
        limit,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn mark_read(
    account_id: String,
    mailbox: String,
    uids: Vec<u64>,
    read: bool,
) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;
    db::mark_read(conn, &account_id, &mailbox, &uids, read).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn move_to_trash(account_id: String, mailbox: String, uids: Vec<u64>) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;
    db::move_to_trash(conn, &account_id, &mailbox, &uids).map_err(|e| e.to_string())?;
    Ok(())
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
async fn export_backup(passphrase: Option<String>) -> Result<String, String> {
    let db_conn = Arc::clone(&DB_CONN);
    let security = Arc::clone(&SECURITY);
    let passphrase_clone = passphrase.clone();
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        let sec = security.lock().unwrap();
        db_export_backup(conn, &sec, passphrase_clone)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_backup(backup_path: String, passphrase: Option<String>) -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    let security = Arc::clone(&SECURITY);
    let path = std::path::PathBuf::from(backup_path);
    let passphrase_clone = passphrase;
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        let sec = security.lock().unwrap();
        db_import_backup(conn, &sec, &path, passphrase_clone).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn run_maintenance() -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::run_maintenance(conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            utils::oauth_server::start(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_initialization_status,
            greet,
            start_oauth_flow,
            complete_oauth_flow,
            add_account,
            list_accounts,
            remove_account,
            check_security_options,
            check_tpm_availability,
            initialize_security,
            fetch_mailboxes,
            fetch_headers,
            fetch_message_full,
            start_sync,
            stop_sync,
            get_sync_status,
            search_messages,
            mark_read,
            move_to_trash,
            enqueue_message,
            list_outbox,
            retry_sending,
            cancel_sending,
            export_backup,
            import_backup,
            run_maintenance
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
