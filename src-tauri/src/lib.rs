pub mod db;
pub mod error;
pub mod globals;
pub mod oauth;
pub mod protocol;
pub mod security;
pub mod utils;

use crate::db::{
    add_account as db_add_account, init_db, list_accounts as db_list_accounts,
    remove_account as db_remove_account, AccountInput, AccountMeta, Credentials, ImapConfig,
    OAuthCredentials, SmtpConfig,
};
use crate::security::stores::SecretStore;
use crate::security::SecurityManager;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref DB_CONN: Arc<Mutex<Connection>> = Arc::new(Mutex::new(init_db().unwrap()));
    static ref SECURITY: Arc<Mutex<SecurityManager>> =
        Arc::new(Mutex::new(SecurityManager::new().unwrap()));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
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

    // Fetch user info to get email
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

    // Create account input
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
        argon2_available: true, // Always available
    })
}

#[tauri::command]
async fn initialize_security(method: String, passphrase: Option<String>) -> Result<(), String> {
    match method.as_str() {
        "tpm" => {
            println!("Initializing TPM security");

            if let Some(tpm_store) = crate::security::stores::tpm::get_tpm_store() {
                // Create SecurityManager with TPM store
                let new_security = crate::security::SecurityManager::with_store(
                    tpm_store.into(),
                    crate::security::stores::StorageTier::Tpm,
                );

                // Initialize it
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

            // Try to create keyring store
            match crate::security::stores::keyring::KeyringStore::new() {
                Ok(store) if store.is_available() => {
                    // Create SecurityManager with KeyringStore
                    let new_security = crate::security::SecurityManager::with_store(
                        std::sync::Arc::new(store),
                        crate::security::stores::StorageTier::Keyring,
                    );

                    // Initialize it
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

            // Create storage path
            let storage_path = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("postail")
                .join("security");

            // Create new SecurityManager with Argon2Store
            let builder =
                crate::security::manager::PassphraseSecurityBuilder::new(storage_path, pass);
            let new_security = builder.build();

            // Initialize it
            let mut security_guard = SECURITY.lock().unwrap();
            *security_guard = new_security;
            security_guard.initialize().map_err(|e| e.to_string())?;

            println!("Argon2 security initialized successfully");
            Ok(())
        }
        _ => return Err("Invalid security method".to_string()),
    }
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
            initialize_security
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
