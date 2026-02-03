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
use chrono::Utc;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::spawn_blocking;
use tokio::time::timeout;

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

#[derive(serde::Serialize)]
pub struct OAuthFlowResponse {
    pub url: String,
    pub port: u16,
}

#[tauri::command]
async fn start_oauth_flow(provider: String) -> Result<OAuthFlowResponse, String> {
    let provider_kind =
        oauth::ProviderKind::parse(&provider).ok_or_else(|| "Unknown provider".to_string())?;
    let provider = oauth::Provider::from_kind(provider_kind);
    match oauth::start_oauth_flow(provider) {
        Ok((url, port)) => Ok(OAuthFlowResponse { url, port }),
        Err(e) => Err(e.to_string()),
    }
}

/// Completes an OAuth authorization flow and creates a new account from the obtained credentials.
///
/// On success, inserts a new account into the database using provider-specific information and the
/// retrieved OAuth tokens, and returns the created account metadata.
///
/// # Errors
///
/// Returns an `Err(String)` when the OAuth flow cannot be completed, required user info (email)
/// cannot be fetched, the database is not initialized, or account creation fails.
///
/// # Examples
///
/// ```no_run
/// # use postail::*;
/// # async fn example() {
/// let res = complete_oauth_flow("auth_code".to_string(), "state".to_string()).await;
/// match res {
///     Ok(account_meta) => println!("Created account: {}", account_meta.id),
///     Err(err) => eprintln!("Failed to complete OAuth flow: {}", err),
/// }
/// # }
/// ```
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
                tracing::error!(target: "postail", "Failed to fetch Gmail user info. Status: {}, Body: {}", status, body);
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
            expires_at: Utc::now().timestamp() + tokens.expires_in as i64,
            auth_type: "oauth2".to_string(),
            provider_type: provider.kind.as_str().to_string(),
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
    let tpm_available = timeout(
        Duration::from_millis(500),
        spawn_blocking(|| crate::security::stores::tpm::get_tpm_store().is_some()),
    )
    .await
    .unwrap_or(Ok(false))
    .unwrap_or(false);

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

fn initialize_security_and_database(
    method: &str,
    passphrase: Option<String>,
) -> Result<(), String> {
    tracing::info!(target: "postail", "Initializing security with method: {}", method);

    // Create security manager based on method
    let security = match method {
        "tpm" => {
            if let Some(tpm_store) = crate::security::stores::tpm::get_tpm_store() {
                crate::security::SecurityManager::with_store(
                    tpm_store.into(),
                    crate::security::stores::StorageTier::Tpm,
                )
            } else {
                return Err("TPM not available or not supported".to_string());
            }
        }
        "keyring" => match crate::security::stores::keyring::KeyringStore::new() {
            Ok(store) => crate::security::SecurityManager::with_store(
                std::sync::Arc::new(store),
                crate::security::stores::StorageTier::Keyring,
            ),
            Err(e) => {
                tracing::warn!(target: "postail", "Keyring initialization failed: {}", e);
                return Err(format!("Keyring not available: {}", e));
            }
        },
        "argon2" => match passphrase {
            Some(pass) => {
                let storage_path = dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("postail")
                    .join("security");
                let builder = crate::security::manager::PassphraseSecurityBuilder::new(
                    storage_path.clone(),
                    pass,
                );
                builder.build()
            }
            None => return Err("Passphrase required for Argon2".to_string()),
        },
        _ => return Err("Invalid security method".to_string()),
    };

    let is_unlocking = security.is_initialized();
    tracing::info!(target: "postail", "Security {}...",
        if is_unlocking { "unlocking" } else { "initializing" });

    {
        let mut security_guard = SECURITY.lock().unwrap();
        *security_guard = security;

        if is_unlocking {
            security_guard.unlock().map_err(|e| e.to_string())?;
        } else {
            security_guard.initialize().map_err(|e| e.to_string())?;
        }
    }

    tracing::info!(target: "postail", "Security {} successfully",
        if is_unlocking { "unlocked" } else { "initialized" });

    let encryption = {
        let security = SECURITY.lock().unwrap();
        let master_key_raw = security.get_master_key_raw();
        crate::security::DbEncryption::derive_from_master_key(&master_key_raw)
            .map_err(|e| e.to_string())?
    };
    let hex_key = encryption.hex_key();

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("postail");
    let db_path = data_dir.join("postail.db");

    let db = if db_path.exists() {
        tracing::info!(target: "postail", "Connecting to existing database...");
        crate::db::connect_db_with_key(&hex_key).map_err(|e| {
            tracing::error!(target: "postail", "Failed to connect to existing database: {}", e);
            format!("Failed to connect to database: {}", e)
        })?
    } else {
        tracing::info!(target: "postail", "Initializing new database...");
        crate::db::init_db_with_key(&hex_key).map_err(|e| e.to_string())?
    };

    {
        let mut db_guard = DB_CONN.lock().unwrap();
        *db_guard = Some(db);
    }

    tracing::info!(target: "postail", "Starting background services...");
    crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
    SMTP_MANAGER.lock().unwrap().start_outbox_worker();
    tracing::info!(target: "postail", "Background services started");

    tracing::info!(target: "postail", "Database ready!");
    tracing::info!(target: "postail", "{} initialization complete!",
        if is_unlocking { "Unlock" } else { "Setup" });

    Ok(())
}

#[tauri::command]
async fn initialize_security(method: String, passphrase: Option<String>) -> Result<(), String> {
    if method == "argon2" && passphrase.is_none() {
        return Err("Passphrase required for Argon2".to_string());
    }

    initialize_security_and_database(&method, passphrase)
}

#[tauri::command]
fn fetch_mailboxes(account_id: String) -> Result<Vec<Mailbox>, String> {
    let imap = IMAP_MANAGER.lock().unwrap();
    let mut mailboxes = imap.fetch_mailboxes_sync(&account_id)?;

    let provider_kind = {
        let conn_guard = DB_CONN.lock().unwrap();
        if let Some(conn) = conn_guard.as_ref() {
            let mut stmt = conn
                .prepare("SELECT provider_type, imap_host FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            let mut rows = stmt.query([&account_id]).map_err(|e| e.to_string())?;

            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let provider_type: String = row.get(0).unwrap_or_default();
                let imap_host: String = row.get(1).unwrap_or_default();

                oauth::ProviderKind::parse(&provider_type)
                    .or_else(|| oauth::ProviderKind::from_imap_host(&imap_host))
            } else {
                None
            }
        } else {
            None
        }
    };

    for mailbox in &mut mailboxes {
        let decoded = utf7_imap::decode_utf7_imap(mailbox.name.clone());
        mailbox.display_name = decoded.clone();

        let lower = decoded.to_lowercase();

        mailbox.role = "other".to_string();

        if lower == "inbox" {
            mailbox.role = "inbox".to_string();
            mailbox.display_name = "Inbox".to_string(); // Force standard casing
        } else if lower.contains("draft") {
            mailbox.role = "drafts".to_string();
        } else if lower.contains("sent") {
            mailbox.role = "sent".to_string();
        } else if lower.contains("trash") || lower.contains("bin") || lower.contains("deleted") {
            mailbox.role = "trash".to_string();
        } else if lower.contains("junk") || lower.contains("spam") {
            mailbox.role = "junk".to_string();
        } else if lower.contains("archive") {
            mailbox.role = "archive".to_string();
        }

        if let Some(kind) = provider_kind {
            let info = oauth::ProviderInfo::get(kind);
            if mailbox.name == info.sent_folder {
                mailbox.role = "sent".to_string();
            }

            // Gmail specific: Remove [Gmail]/ prefix for display
            if kind == oauth::ProviderKind::Gmail && mailbox.display_name.starts_with("[Gmail]/") {
                mailbox.display_name = mailbox.display_name.replace("[Gmail]/", "");
            }
        }
    }

    mailboxes.sort_by(|a, b| {
        let role_score = |role: &str| match role {
            "inbox" => 0,
            "sent" => 1,
            "drafts" => 2,
            "trash" => 3,
            "archive" => 4,
            "junk" => 5,
            _ => 10,
        };

        let score_a = role_score(&a.role);
        let score_b = role_score(&b.role);

        if score_a != score_b {
            score_a.cmp(&score_b)
        } else {
            a.display_name.cmp(&b.display_name)
        }
    });

    Ok(mailboxes)
}

#[tauri::command]
async fn fetch_headers(
    account_id: String,
    mailbox: String,
    anchor: Option<u64>,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    let anchor: Option<u32> = anchor
        .map(|a| a.try_into().map_err(|_| "Anchor too large".to_string()))
        .transpose()?;
    let imap = IMAP_MANAGER.lock().unwrap().clone();
    imap.fetch_headers_hybrid(&account_id, &mailbox, anchor, limit)
        .await
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
    tracing::info!(target: "postail", "[UI] start_sync called for {}", account_id);
    let imap = IMAP_MANAGER.lock().unwrap();
    tracing::info!(target: "postail", "[UI] Calling imap.start_sync");
    match imap.start_sync(&account_id) {
        Ok(()) => {
            tracing::info!(target: "postail", "[UI] start_sync succeeded");
            Ok(())
        }
        Err(e) => {
            tracing::error!(target: "postail", "[UI] start_sync failed: {}", e);
            Err(e.to_string())
        }
    }
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

/// Moves the specified messages to the trash for an account's mailbox.
///
/// This validates UIDs fit into the internal 32-bit representation and delegates the move operation
/// to the database. It may fail if the database is not initialized, any UID is larger than what
/// fits in a `u32`, or the underlying database operation returns an error.
///
/// # Returns
///
/// `Ok(())` on success, `Err(String)` with a descriptive message on failure.
///
/// # Examples
///
/// ```
/// // Move messages with UIDs 1 and 2 to trash for the given account and mailbox.
/// let res = move_to_trash("account-id".into(), "INBOX".into(), vec![1, 2]);
/// assert!(res.is_ok());
/// ```
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

/// Enqueues a raw RFC-822 email message for delivery from the given account.
///
/// The `raw_eml` bytes must contain the complete MIME/RFC-822 message to send.
///
/// # Examples
///
/// ```no_run
/// let account = "account-id-123".to_string();
/// let eml = b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Hi\r\n\r\nHello".to_vec();
/// let result = enqueue_message(account, eml);
/// assert!(result.is_ok()); // returns an outbox item id on success
/// ```
#[tauri::command]
fn enqueue_message(account_id: String, raw_eml: Vec<u8>) -> Result<String, String> {
    let smtp = SMTP_MANAGER.lock().unwrap();
    smtp.enqueue_message(&account_id, &raw_eml)
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

/// Runs database maintenance tasks using the configured database connection.
///
/// This spawns a blocking task to execute maintenance via the database layer and waits for completion.
///
/// # Returns
///
/// `Ok(())` if maintenance completed successfully, `Err(String)` with an error message otherwise.
///
/// # Examples
///
/// ```
/// # async fn example() {
/// let result = run_maintenance().await;
/// assert!(result.is_ok());
/// # }
/// ```
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

/// Persists a draft message to the application's database.
///
/// Saves the provided `Draft` into persistent storage. Returns an `Err` with a string message if the database is not initialized or if saving fails.
///
/// # Arguments
///
/// * `draft` - The draft to persist; its identifiers, recipients, subject, body, and attachments are stored.
///
/// # Examples
///
/// ```no_run
/// # use tokio::runtime::Runtime;
/// // `draft` should be constructed according to `crate::db::Draft` fields.
/// let rt = Runtime::new().unwrap();
/// let draft = /* build a crate::db::Draft instance */;
/// rt.block_on(async {
///     // On success this completes without error.
///     let _ = crate::commands::save_draft(draft).await;
/// });
/// ```
#[tauri::command]
async fn save_draft(draft: crate::db::Draft) -> Result<(), String> {
    let body_len = draft.body.as_ref().map(|b| b.len()).unwrap_or(0);
    tracing::info!(target: "postail", "[save_draft] Received draft from frontend - id={}, subject={:?}, body_len={}, to_count={}, cc_count={}, bcc_count={}",
        draft.id, draft.subject, body_len, draft.to.len(), draft.cc.len(), draft.bcc.len());

    let db_conn = Arc::clone(&DB_CONN);
    let _ = tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::save_draft(conn, &draft).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    tracing::info!(target: "postail", "[save_draft] Draft saved successfully");
    Ok(())
}

/// Lists saved drafts for the specified account.
///
/// Fetches all drafts stored in the application's database that belong to the given account ID.
///
/// # Returns
///
/// `Ok(Vec<crate::db::Draft>)` with the account's drafts on success, `Err(String)` with an error message on failure.
///
/// # Examples
///
/// ```
/// # use tokio;
/// # async fn run_example() {
/// let account_id = "account-123".to_string();
/// let result = crate::commands::list_drafts(account_id).await;
/// match result {
///     Ok(drafts) => println!("Found {} drafts", drafts.len()),
///     Err(e) => eprintln!("Error listing drafts: {}", e),
/// }
/// # }
/// ```
#[tauri::command]
async fn list_drafts(account_id: String) -> Result<Vec<crate::db::Draft>, String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::list_drafts(conn, &account_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Deletes a saved draft by its ID.
///
/// # Parameters
///
/// - `id`: The identifier of the draft to remove.
///
/// # Returns
///
/// `Ok(())` on success, `Err` with a string message on failure.
///
/// # Examples
///
/// ```
/// let rt = tokio::runtime::Runtime::new().unwrap();
/// let res = rt.block_on(delete_draft("draft-id-123".into()));
/// // `res` will be `Ok(())` if deletion succeeded or `Err(_)` on failure.
/// ```
#[tauri::command]
async fn delete_draft(id: String) -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::delete_draft(conn, &id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Searches contacts that match the provided query and returns up to `limit` results.
///
/// The query is used to filter contacts (for example by name or email). Results are returned
/// in no particular order and may be an empty vector if there are no matches.
///
/// # Parameters
///
/// - `query`: The search string used to match contacts.
/// - `limit`: Maximum number of contacts to return.
///
/// # Returns
///
/// A `Vec<Contact>` containing matching contacts. Returns an empty vector when no contacts match.
///
/// # Examples
///
/// ```no_run
/// let results = search_contacts("alice".to_string(), 10).unwrap();
/// for c in results { println!("{}", c.email); }
/// ```
#[tauri::command]
fn search_contacts(query: String, limit: u32) -> Result<Vec<crate::db::Contact>, String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    crate::db::search_contacts(conn, &query, limit).map_err(|e| e.to_string())
}

/// Adds a file from `path` to the draft attachments store and returns the created attachment metadata.
///
/// `path` is the filesystem path to the file to add. The file is read and stored by the attachments subsystem.
///
/// # Returns
///
/// The stored `crate::db::DraftAttachment` on success.
///
/// # Examples
///
/// ```
/// use tokio::runtime::Runtime;
///
/// // Create a runtime to call the async command in tests/examples.
/// let rt = Runtime::new().unwrap();
/// // Replace with an actual file path available during the test.
/// let path = "tests/fixtures/example.png".to_string();
/// let attachment = rt.block_on(crate::commands::add_attachment(path)).unwrap();
/// assert!(!attachment.id.is_empty());
/// ```
#[tauri::command]
async fn add_attachment(path: String) -> Result<crate::db::DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_attachment(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Creates a draft attachment from raw bytes and stores it in the attachments storage.
///
/// # Parameters
///
/// - `bytes`: The raw file contents to store as the attachment.
/// - `filename`: The filename to associate with the stored attachment.
/// - `content_type`: The MIME content type of the attachment (e.g., `"image/png"`).
///
/// # Returns
///
/// A `DraftAttachment` representing the stored attachment on success, or an error message string on failure.
///
/// # Examples
///
/// ```
/// # use tokio;
/// # async fn _example() -> Result<(), String> {
/// let data = b"hello world".to_vec();
/// let filename = "hello.txt".to_string();
/// let content_type = "text/plain".to_string();
/// let attachment = crate::commands::add_attachment_bytes(data, filename, content_type).await?;
/// assert!(!attachment.id.is_empty());
/// # Ok(())
/// # }
/// ```
#[tauri::command]
async fn add_attachment_bytes(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<crate::db::DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_attachment_bytes(bytes, filename, content_type)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Removes an attachment record with the given id from the database.
///
/// # Examples
///
/// ```no_run
/// // call from an async context
/// let _ = remove_attachment("attachment-id-123".into()).await;
/// ```
///
/// # Returns
///
/// `Ok(())` if the attachment was removed successfully, `Err(String)` with an error message otherwise.
#[tauri::command]
async fn remove_attachment(id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::remove_attachment(&id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Adds an inline attachment from in-memory bytes and stores it in the draft attachments store.
///
/// The provided bytes are persisted as an inline attachment with the given filename and MIME
/// content type; on success the stored `DraftAttachment` record is returned.
///
/// # Examples
///
/// ```no_run
/// # async fn example() {
/// let bytes = b"hello".to_vec();
/// let res = add_inline_attachment(bytes, "file.txt".into(), "text/plain".into()).await;
/// assert!(res.is_ok());
/// # }
/// ```
#[tauri::command]
async fn add_inline_attachment(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<crate::db::DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_inline_attachment_bytes(bytes, filename, content_type)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
pub struct BuildEmailResult {
    pub eml_bytes: Vec<u8>,
    pub html_with_cids: String,
}

/// Builds a complete MIME email from a stored draft, producing the serialized EML bytes and the HTML body with inline asset CIDs.
///
/// This loads the draft and originating account email from the database, replaces asset URLs in the draft HTML with content IDs, assembles recipients and attachments, and constructs a multipart MIME message.
///
/// On success returns a `BuildEmailResult` whose `eml_bytes` field contains the serialized email and whose `html_with_cids` field contains the processed HTML with CID references. Returns `Err(String)` if the database is not initialized, the draft is not found, or any I/O/serialization/database operation fails.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// // `draft_id` should refer to an existing draft in the initialized test database.
/// let _res = build_email_from_draft("example-draft-id".into()).await;
/// # });
/// ```
#[tauri::command]
async fn build_email_from_draft(draft_id: String) -> Result<BuildEmailResult, String> {
    tracing::info!(target: "postail", "[build_email_from_draft] Starting for draft_id={}", draft_id);
    let db_conn = Arc::clone(&DB_CONN);

    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

        let draft = crate::db::load_draft(conn, &draft_id)
            .map_err(|e| e.to_string())?
            .ok_or("Draft not found")?;

        let from_email: String = {
            let mut stmt = conn
                .prepare("SELECT email FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            stmt.query_row([&draft.account_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
        };

        let html_body = draft.body.unwrap_or_default();
        let html_with_cids =
            crate::smtp::mime_builder::replace_asset_urls_with_cids(&html_body, &draft.attachments);

        let to: Vec<&str> = draft.to.iter().map(|s| s.as_str()).collect();
        let cc: Vec<&str> = draft.cc.iter().map(|s| s.as_str()).collect();
        let bcc: Vec<&str> = draft.bcc.iter().map(|s| s.as_str()).collect();
        let subject = draft.subject.unwrap_or_default();

        tracing::info!(target: "postail", "[build_email_from_draft] Building email with {} to, {} cc, {} bcc recipients, subject='{}'",
            to.len(), cc.len(), bcc.len(), subject);

        let eml_bytes = crate::smtp::mime_builder::build_multipart_email(
            &from_email,
            to,
            cc,
            bcc,
            &subject,
            &html_with_cids,
            &draft.attachments,
        )
        .map_err(|e| e.to_string())?;

        Ok(BuildEmailResult {
            eml_bytes,
            html_with_cids,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Sanitizes HTML email content and returns detailed results about the sanitization.
///
/// The returned `SanitizeResult` includes the sanitized HTML and metadata describing
/// any removals or modifications performed during sanitization.
///
/// # Examples
///
/// ```
/// let input = "<script>alert(1)</script><p>Hello</p>".to_string();
/// let res = process_email_content(input);
/// assert!(res.html.contains("<p>Hello</p>"));
/// ```
#[tauri::command]
fn process_email_content(html: String) -> crate::utils::sanitizer::SanitizeResult {
    crate::utils::sanitizer::sanitize_email_html_with_details(&html)
}

/// Apply automatic fixes to HTML email content.
///
/// This performs non-destructive, automated corrections (e.g., fixing common markup issues and sanitizing problematic constructs)
/// to produce HTML better suited for email rendering.
///
/// # Returns
///
/// The corrected HTML string.
///
/// # Examples
///
/// ```
/// let raw = "<div><p>Unclosed tag<p></div>";
/// let fixed = auto_fix_email_html(raw.into());
/// assert!(fixed.contains("</p>"));
/// ```
#[tauri::command]
fn auto_fix_email_html(html: String) -> String {
    crate::utils::sanitizer::auto_fix_email_html(&html)
}

/// Initializes logging and starts the Tauri application with plugins, command handlers, and app setup.
///
/// This configures the tracing subscriber, registers opener/os/dialog plugins, starts the local OAuth helper,
/// wires the global SMTP manager to the app handle, registers all IPC/tauri commands, registers the
/// "postail" URI scheme protocol handler, and runs the Tauri runtime.
///
/// # Examples
///
/// ```no_run
/// fn main() {
///     postail::run();
/// }
/// ```
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .without_time()
        .with_line_number(false)
        .with_file(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            utils::oauth_server::start(handle.clone());

            SMTP_MANAGER.lock().unwrap().set_app_handle(handle);

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
            run_maintenance,
            save_draft,
            list_drafts,
            delete_draft,
            search_contacts,
            add_attachment,
            add_attachment_bytes,
            add_inline_attachment,
            remove_attachment,
            build_email_from_draft,
            process_email_content,
            auto_fix_email_html
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}