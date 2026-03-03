use crate::globals::{DB_CONN, SECURITY, SMTP_MANAGER};
use crate::security::manager::PassphraseSecurityBuilder;
use crate::security::recovery::RecoveryStore;
use crate::security::stores::keyring::KeyringStore;
use crate::security::stores::tpm::get_tpm_store;
use crate::security::stores::{SecretStore, StorageTier};
use crate::security::{DbEncryption, SecurityManager};
use crate::utils::config::{load_config, save_config, AppConfig};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::command;
use tokio::task::spawn_blocking;
use tokio::time::timeout;

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TpmErrorType {
    Cancelled,
    AccessDenied,
    HelperFailed,
    StartFailed,
    SocketTimeout,
    Other,
}

#[derive(Serialize, Debug, Clone)]
pub struct TpmInitError {
    pub error_type: TpmErrorType,
    pub message: String,
}

impl std::fmt::Display for TpmInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl From<TpmInitError> for String {
    fn from(err: TpmInitError) -> Self {
        serde_json::to_string(&err).unwrap_or(err.message)
    }
}

#[derive(Serialize)]
pub struct SecurityOptions {
    pub tpm_available: bool,
    pub tpm_requires_elevation: bool,
    pub keyring_available: bool,
    pub argon2_available: bool,
}

#[command]
pub async fn check_security_options() -> Result<SecurityOptions, String> {
    let (tpm_available, tpm_requires_elevation) = timeout(
        Duration::from_secs(3),
        spawn_blocking(|| {
            use crate::security::TpmInitializer;
            let initializer = TpmInitializer::new();
            let availability = initializer.check_availability();
            match availability {
                crate::security::TpmAvailability::Available => (true, false),
                crate::security::TpmAvailability::RequiresElevation => (true, true),
                crate::security::TpmAvailability::NotAvailable => (false, false),
            }
        }),
    )
    .await
    .unwrap_or(Ok((false, false)))
    .unwrap_or((false, false));

    let keyring_available = KeyringStore::new()
        .map(|k| k.is_available())
        .unwrap_or(false);

    Ok(SecurityOptions {
        tpm_available,
        tpm_requires_elevation,
        keyring_available,
        argon2_available: true,
    })
}

#[derive(Serialize)]
pub enum TpmStatus {
    Available,
    RequiresElevation,
    NotAvailable,
}

#[command]
pub async fn check_tpm_availability() -> Result<TpmStatus, String> {
    use crate::security::TpmAvailability;

    let result = timeout(
        Duration::from_secs(3),
        spawn_blocking(|| {
            let initializer = crate::security::TpmInitializer::new();
            initializer.check_availability()
        }),
    )
    .await
    .unwrap_or(Ok(TpmAvailability::NotAvailable))
    .unwrap_or(TpmAvailability::NotAvailable);

    match result {
        TpmAvailability::Available => Ok(TpmStatus::Available),
        TpmAvailability::RequiresElevation => Ok(TpmStatus::RequiresElevation),
        TpmAvailability::NotAvailable => Ok(TpmStatus::NotAvailable),
    }
}

#[derive(Serialize)]
pub struct InitStatus {
    pub status: String,
    pub method: Option<String>,
}

#[command]
pub fn get_app_initialization_status() -> InitStatus {
    let data_dir = crate::utils::config::get_data_dir();
    let db_path = data_dir.join("postail.db");

    let status = if db_path.exists() {
        "Locked".to_string()
    } else {
        "SetupRequired".to_string()
    };

    let method = load_config().map(|c| c.security_method);

    InitStatus { status, method }
}

fn get_executable_path() -> std::io::Result<std::path::PathBuf> {
    if let Ok(appimage) = std::env::var("APPIMAGE") {
        return Ok(std::path::PathBuf::from(appimage));
    }
    std::env::current_exe()
}

pub async fn initialize_security_and_database(
    method: &str,
    passphrase: Option<String>,
    recovery_phrase: Option<String>,
) -> Result<(), String> {
    tracing::info!(target: "postail", "Initializing security with method: {}", method);

    let mut security = match method {
        "tpm" => {
            if let Some(tpm_store) = get_tpm_store() {
                SecurityManager::with_store(tpm_store.into(), StorageTier::Tpm)
            } else {
                return Err("TPM not available or not supported".to_string());
            }
        }
        "keyring" => match KeyringStore::new() {
            Ok(store) => SecurityManager::with_store(Arc::new(store), StorageTier::Keyring),
            Err(e) => {
                tracing::warn!(target: "postail", "Keyring initialization failed: {}", e);
                return Err(format!("Keyring not available: {}", e));
            }
        },
        "argon2" => match passphrase {
            Some(pass) => {
                let storage_path = crate::utils::config::get_data_dir().join("security");
                let builder = PassphraseSecurityBuilder::new(storage_path, pass);
                builder.build()
            }
            None => return Err("Passphrase required for Argon2".to_string()),
        },
        _ => return Err("Invalid security method".to_string()),
    };

    let is_unlocking = security.is_initialized();
    if is_unlocking {
        security.unlock().map_err(|e| e.to_string())?;
    } else {
        security.initialize().map_err(|e| e.to_string())?;
    }

    {
        let mut security_guard = SECURITY.lock().await;
        *security_guard = security;
    }

    // create recovery store if phrase was provided (argon2 setup only)
    let phrase_to_use = recovery_phrase.or_else(crate::security::recovery::get_pending_phrase);

    if let Some(phrase) = phrase_to_use {
        let storage_path = crate::utils::config::get_data_dir().join("security");
        let recovery_store = RecoveryStore::new(storage_path);
        let security = SECURITY.lock().await;
        let master_key = security.export_master_key().map_err(|e| e.to_string())?;
        // release lock before doing filesystem operations
        drop(security);
        recovery_store
            .create(&master_key, &phrase)
            .map_err(|e| e.to_string())?;
        crate::security::recovery::clear_pending_phrase(); // Clear checking phrase after use
        tracing::info!(target: "postail", "Recovery store created");
    }

    let master_key_raw = {
        let security = SECURITY.lock().await;

        security.get_master_key_raw()
    };
    let encryption =
        DbEncryption::derive_from_master_key(&master_key_raw).map_err(|e| e.to_string())?;
    let hex_key = encryption.hex_key();

    // Guard: if DB is already initialized, skip re-opening.
    {
        let already_init = DB_CONN.lock().await.is_some();
        if already_init {
            tracing::warn!(
                target: "postail",
                "initialize_security_and_database called but DB_CONN already initialized — skipping re-init to prevent corruption"
            );
            return Ok(());
        }
    }

    let data_dir = crate::utils::config::get_data_dir();
    let db_path = data_dir.join("postail.db");

    let db = if db_path.exists() {
        crate::db::connect_db_with_key(&hex_key).map_err(|e| e.to_string())?
    } else {
        crate::db::init_db_with_key(&hex_key).map_err(|e| e.to_string())?
    };

    {
        let mut db_guard = DB_CONN.lock().await;
        *db_guard = Some(db);
    }

    crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
    SMTP_MANAGER.lock().await.start_outbox_worker();

    let existing_theme = load_config().and_then(|c| c.theme);
    save_config(&AppConfig {
        security_method: method.to_string(),
        theme: existing_theme,
    })?;

    // Load lock settings from db
    crate::security::load_lock_settings().await;

    Ok(())
}

/// Initialize TPM with elevated privileges if needed (Linux only)
#[cfg(target_os = "linux")]
async fn initialize_tpm_elevated() -> Result<(), TpmInitError> {
    use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::time::Duration;
    use tokio::io::unix::AsyncFd;
    use tokio::process::Command;

    // Check if we're already running as helper mode to avoid infinite loops
    if std::env::var("POSTAIL_TPM_HELPER").is_ok() {
        return Err(TpmInitError {
            error_type: TpmErrorType::Other,
            message: "Already in helper mode".to_string(),
        });
    }

    // Define socket path
    let uid = unsafe { nix::libc::getuid() };
    let socket_path = std::path::PathBuf::from(format!("/run/user/{}/postail-tpm.sock", uid));

    // If socket already exists, try to PING it to check if it's really alive and has TPM access
    if socket_path.exists() {
        use crate::security::stores::tpm::linux::LinuxTpmStore;
        if let Ok(store) = LinuxTpmStore::new() {
            if store.verify_proxy() {
                tracing::info!(target: "postail", "TPM helper already running and healthy.");
                return Ok(());
            }
        }
        // If it exists but is stale/unhealthy, try to restart it
        let _ = std::fs::remove_file(&socket_path);
    }

    let exe_path = get_executable_path()
        .map_err(|e| TpmInitError {
            error_type: TpmErrorType::Other,
            message: format!("Failed to get executable path: {}", e),
        })?
        .to_string_lossy()
        .to_string();

    tracing::info!(target: "postail", "Requesting TPM elevation via pkexec (persistent helper)...");

    let socket_dir = socket_path.parent().ok_or_else(|| TpmInitError {
        error_type: TpmErrorType::Other,
        message: "Invalid socket path".to_string(),
    })?;

    #[derive(Debug)]
    struct InotifyWrapper(Inotify);
    impl AsRawFd for InotifyWrapper {
        fn as_raw_fd(&self) -> RawFd {
            use std::os::fd::AsFd;
            self.0.as_fd().as_raw_fd()
        }
    }

    let inotify = Inotify::init(InitFlags::IN_NONBLOCK | InitFlags::IN_CLOEXEC).map_err(
        |e: nix::Error| TpmInitError {
            error_type: TpmErrorType::Other,
            message: format!("inotify init failed: {}", e),
        },
    )?;
    inotify
        .add_watch(
            socket_dir,
            AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO,
        )
        .map_err(|e: nix::Error| TpmInitError {
            error_type: TpmErrorType::Other,
            message: format!("inotify add_watch failed: {}", e),
        })?;
    let async_inotify =
        AsyncFd::new(InotifyWrapper(inotify)).map_err(|e: std::io::Error| TpmInitError {
            error_type: TpmErrorType::Other,
            message: format!("AsyncFd wrapper failed: {}", e),
        })?;

    let (tx, mut rx) = tokio::sync::oneshot::channel::<TpmInitError>();

    tokio::spawn(async move {
        let data_dir = crate::utils::config::get_data_dir()
            .to_string_lossy()
            .to_string();

        let passthrough_vars = ["XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "APPIMAGE"];

        let mut cmd = Command::new("pkexec");
        cmd.arg("env")
            .arg("POSTAIL_TPM_HELPER=1")
            .arg(format!("POSTAIL_PARENT_PID={}", std::process::id()))
            .arg(format!("POSTAIL_DATA_DIR={}", data_dir));
        for var in passthrough_vars {
            if let Ok(val) = std::env::var(var) {
                cmd.arg(format!("{}={}", var, val));
            }
        }
        cmd.arg(&exe_path);

        let status = cmd.status().await;

        match status {
            Ok(s) if !s.success() => {
                let code = s.code();
                let (error_type, err_msg) = if code == Some(126) || code == Some(127) {
                    // pkexec returns 126 or 127 when auth is cancelled or failed
                    (
                        TpmErrorType::Cancelled,
                        "TPM elevation was cancelled by user".to_string(),
                    )
                } else {
                    (
                        TpmErrorType::HelperFailed,
                        format!("Persistent TPM helper failed with status: {}", s),
                    )
                };
                tracing::error!(target: "postail", "{}", err_msg);
                let _ = tx.send(TpmInitError {
                    error_type,
                    message: err_msg,
                });
            }
            Err(e) => {
                let err_msg = format!("Failed to start persistent TPM helper: {}", e);
                tracing::error!(target: "postail", "{}", err_msg);
                let _ = tx.send(TpmInitError {
                    error_type: TpmErrorType::StartFailed,
                    message: err_msg,
                });
            }
            _ => {
                tracing::info!(target: "postail", "Persistent TPM helper exited.");
            }
        }
    });

    tracing::info!(target: "postail", "Waiting for TPM helper socket to appear...");

    // Check if socket already appeared between add_watch and the await
    let wait_res: Result<(), TpmInitError> = async {
        if socket_path.exists() {
            return Ok::<(), TpmInitError>(());
        }

        while !socket_path.exists() {
            tokio::select! {
                res = async_inotify.readable() => {
                    let mut guard = res.map_err(|e: std::io::Error| TpmInitError {
                        error_type: TpmErrorType::Other,
                        message: e.to_string()
                    })?;
                    let mut events = guard.get_inner().0.read_events()
                        .map_err(|e: nix::Error| TpmInitError {
                            error_type: TpmErrorType::Other,
                            message: e.to_string()
                        })?;
                    let mut found = false;
                    for event in &mut events {
                        if let Some(name) = &event.name {
                            if name.to_string_lossy() == "postail-tpm.sock" {
                                found = true;
                                break;
                            }
                        }
                    }
                    if found { break; }
                    guard.clear_ready();
                }
                err = &mut rx => {
                    return Err(err.unwrap_or_else(|_| TpmInitError {
                        error_type: TpmErrorType::HelperFailed,
                        message: "TPM helper failed to start".to_string()
                    }));
                }
            }
        }
        Ok::<(), TpmInitError>(())
    }
    .await;

    wait_res?;

    // Final verification: try to connect to ensure the server finished binding
    let mut retries = 5;
    while retries > 0 {
        if std::os::unix::net::UnixStream::connect(&socket_path).is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        retries -= 1;
    }

    if retries == 0 {
        return Err(TpmInitError {
            error_type: TpmErrorType::SocketTimeout,
            message: "TPM helper socket appeared but is not responding".to_string(),
        });
    }

    tracing::info!(target: "postail", "TPM elevation successful (proxy ready).");
    Ok(())
}

#[command]
pub async fn initialize_security(
    method: String,
    passphrase: Option<String>,
    recovery_phrase: Option<String>,
) -> Result<(), String> {
    if method == "argon2" && passphrase.is_none() {
        return Err("Passphrase required for Argon2".to_string());
    }

    // Check if TPM requires elevation (Linux only)
    #[cfg(target_os = "linux")]
    if method == "tpm" {
        let availability =
            spawn_blocking(|| crate::security::TpmInitializer::new().check_availability())
                .await
                .map_err(|e| e.to_string())?;

        if matches!(
            availability,
            crate::security::TpmAvailability::RequiresElevation
        ) {
            initialize_tpm_elevated().await.map_err(|e| e.to_string())?;
        }
    }

    initialize_security_and_database(&method, passphrase, recovery_phrase)
        .await
        .map_err(|e| e)
}

/// Re-encrypts the existing (already-unlocked) master key with a new storage method.
#[command]
pub async fn change_security_method(
    method: String,
    passphrase: Option<String>,
) -> Result<(), String> {
    // Extract current master key — security must already be unlocked
    let master_key = {
        let security = SECURITY.lock().await;
        security
            .export_master_key()
            .map_err(|e| format!("No unlocked master key: {}", e))?
    };

    // Build new SecurityManager for the chosen method
    let mut new_security = match method.as_str() {
        "tpm" => {
            // TPM may need elevation
            #[cfg(target_os = "linux")]
            {
                let availability =
                    spawn_blocking(|| crate::security::TpmInitializer::new().check_availability())
                        .await
                        .map_err(|e| e.to_string())?;
                if matches!(
                    availability,
                    crate::security::TpmAvailability::RequiresElevation
                ) {
                    initialize_tpm_elevated().await.map_err(|e| e.to_string())?;
                }
            }
            if let Some(tpm_store) = get_tpm_store() {
                SecurityManager::with_store(tpm_store.into(), StorageTier::Tpm)
            } else {
                return Err("TPM not available or not supported".to_string());
            }
        }
        "keyring" => match KeyringStore::new() {
            Ok(store) => SecurityManager::with_store(Arc::new(store), StorageTier::Keyring),
            Err(e) => return Err(format!("Keyring not available: {}", e)),
        },
        "argon2" => {
            let pass = passphrase.ok_or("Passphrase required for Argon2")?;
            let storage_path = crate::utils::config::get_data_dir().join("security");
            PassphraseSecurityBuilder::new(storage_path, pass).build()
        }
        _ => return Err("Invalid security method".to_string()),
    };

    // Store the existing master key with the new method
    new_security
        .initialize_with_key(master_key)
        .map_err(|e| e.to_string())?;

    // Replace global security manager
    {
        let mut guard = SECURITY.lock().await;
        *guard = new_security;
    }

    // Save pending recovery phrase (generated by RecoveryStep before verification)
    let phrase_to_use = crate::security::recovery::get_pending_phrase();
    if let Some(phrase) = phrase_to_use {
        let storage_path = crate::utils::config::get_data_dir().join("security");
        let recovery_store = RecoveryStore::new(storage_path);
        let key_raw = {
            let security = SECURITY.lock().await;
            security.get_master_key_raw()
        };
        let master_key =
            crate::security::MasterKey::from_bytes(&key_raw).map_err(|e| e.to_string())?;
        recovery_store
            .create(&master_key, &phrase)
            .map_err(|e| e.to_string())?;
        crate::security::recovery::clear_pending_phrase();
        tracing::info!(target: "postail", "Recovery store re-created for new security method");
    }

    // Save new method to config
    let existing_theme = load_config().and_then(|c| c.theme);
    save_config(&AppConfig {
        security_method: method.clone(),
        theme: existing_theme,
    })?;

    tracing::info!(target: "postail", "Security method changed to: {}", method);
    Ok(())
}

#[command]
pub async fn reset_security_setup() -> Result<(), String> {
    use std::fs;

    // Safety: refuse if accounts already exist - this is only for initial setup rollback
    {
        let conn_guard = DB_CONN.lock().await;
        if let Some(conn) = conn_guard.as_ref() {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
                .unwrap_or(0);
            if count > 0 {
                return Err("Cannot reset: accounts already exist.".to_string());
            }
        }
    }

    // 1. Drop the DB connection
    {
        let mut db_guard = DB_CONN.lock().await;
        *db_guard = None;
    }

    // 2. Reset the security manager to an uninitialized state
    {
        let mut security_guard = SECURITY.lock().await;
        if let Some(tpm_store) = crate::security::stores::tpm::get_tpm_store() {
            *security_guard = SecurityManager::with_store(tpm_store.into(), StorageTier::Tpm);
        } else if let Ok(keyring) = KeyringStore::new() {
            *security_guard = SecurityManager::with_store(Arc::new(keyring), StorageTier::Keyring);
        }
        // master_key is None, so the manager is effectively locked/empty
    }

    let data_dir = crate::utils::config::get_data_dir();

    // 3. Delete the database file
    let db_path = data_dir.join("postail.db");
    if db_path.exists() {
        fs::remove_file(&db_path).map_err(|e| format!("Failed to delete database: {}", e))?;
    }

    // 4. Delete the security directory (argon2 sealed key, recovery.sealed, etc.)
    let security_path = data_dir.join("security");
    if security_path.exists() {
        fs::remove_dir_all(&security_path)
            .map_err(|e| format!("Failed to delete security data: {}", e))?;
    }

    // 5. Delete any encrypted credentials (none should exist, but clean up just in case)
    let creds_path = data_dir.join("creds");
    if creds_path.exists() {
        let _ = fs::remove_dir_all(&creds_path);
    }

    // 6. Remove security_method from config, keep theme
    let existing_theme = crate::utils::config::load_config().and_then(|c| c.theme);
    save_config(&AppConfig {
        security_method: String::new(),
        theme: existing_theme,
    })?;

    // 7. Clear any pending recovery phrase
    crate::security::recovery::clear_pending_phrase();

    tracing::info!(target: "postail", "[Security] Setup rolled back - clean slate for re-setup");
    Ok(())
}

#[command]
pub fn generate_recovery_phrase() -> String {
    let phrase = crate::security::recovery::generate_phrase();
    crate::security::recovery::store_pending_phrase(phrase.clone());
    phrase
}

#[command]
pub async fn unlock_with_recovery_phrase(phrase: String) -> Result<(), String> {
    let storage_path = crate::utils::config::get_data_dir().join("security");
    let recovery_store = crate::security::recovery::RecoveryStore::new(storage_path.clone());

    if !recovery_store.exists() {
        return Err("No recovery store found".to_string());
    }

    // decrypt master key from recovery phrase
    let master_key = recovery_store.unlock(&phrase).map_err(|e| e.to_string())?;

    let holder = crate::security::recovery::RecoveryKeyHolder::with_key(master_key);
    let security = SecurityManager::with_store(Arc::new(holder), StorageTier::Passphrase);

    {
        let mut security_guard = SECURITY.lock().await;
        *security_guard = security;
        security_guard.unlock().map_err(|e| e.to_string())?;
    }

    let encryption = {
        let security = SECURITY.lock().await;
        let master_key_raw = security.get_master_key_raw();
        DbEncryption::derive_from_master_key(&master_key_raw).map_err(|e| e.to_string())?
    };
    let hex_key = encryption.hex_key();

    let data_dir = crate::utils::config::get_data_dir();
    let db_path = data_dir.join("postail.db");

    let db = if db_path.exists() {
        crate::db::connect_db_with_key(&hex_key).map_err(|e| e.to_string())?
    } else {
        return Err("No database found to unlock".to_string());
    };

    {
        let mut db_guard = DB_CONN.lock().await;
        *db_guard = Some(db);
    }

    crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
    SMTP_MANAGER.lock().await.start_outbox_worker();
    crate::security::load_lock_settings().await;

    tracing::info!(target: "postail", "Unlocked via recovery phrase");
    Ok(())
}

#[command]
pub fn verify_recovery_words(indices: Vec<usize>, words: Vec<String>) -> Result<bool, String> {
    crate::security::recovery::verify_pending_phrase(&indices, &words).map_err(|e| e.to_string())
}

use crate::security::lock::{
    get_timeout_minutes, is_locked, is_using_encryption_password, record_activity, set_pin,
    set_timeout, unlock, use_encryption_password,
};

#[command]
pub fn record_lock_activity() {
    record_activity();
}

#[command]
pub fn is_app_locked() -> bool {
    is_locked()
}

#[command]
pub async fn unlock_app(password: String) -> Result<(), String> {
    let security = SECURITY.lock().await;
    let db_password = if security.is_initialized() {
        Some(hex::encode(security.get_master_key_raw()))
    } else {
        None
    };
    unlock(&password, db_password.as_deref())
}

#[command]
pub async fn set_auto_lock_timeout(minutes: u32) {
    set_timeout(minutes).await;
}

#[command]
pub fn get_auto_lock_timeout() -> u32 {
    get_timeout_minutes()
}

#[command]
pub async fn set_auto_lock_pin(pin: String) {
    set_pin(&pin).await;
}

#[command]
pub async fn use_encryption_password_for_lock() {
    use_encryption_password().await;
}

#[command]
pub fn is_lock_using_encryption_password() -> bool {
    is_using_encryption_password()
}

#[command]
pub fn is_lock_configured() -> bool {
    crate::security::is_lock_configured()
}

#[command]
pub fn get_security_method() -> Option<String> {
    load_config().map(|c| c.security_method)
}
