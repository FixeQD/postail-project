pub mod lock;
pub mod options;
pub mod recovery;

pub use lock::*;
pub use options::*;
pub use recovery::*;

use crate::globals::{CRYPTO_ACTOR, DB_CONN, SECURITY, SMTP_MANAGER, get_db_pool};
use crate::security::recovery::RecoveryStore;
use crate::security::storage::keyring::KeyringStore;
use crate::security::tpm::store::get_tpm_store;
use crate::security::{DbEncryption, PassphraseSecurityBuilder, SecurityManager, StorageTier};
use crate::utils::config::{AppConfig, load_config, save_config};
use serde::Serialize;
use std::sync::Arc;
use tauri::command;
#[cfg(all(target_os = "linux", feature = "tpm"))]
use tokio::task::spawn_blocking;

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

    // Initialize crypto actor for non-blocking crypto operations
    {
        let crypto = {
            let guard = SECURITY.lock().await;
            let sm: &SecurityManager = &*guard;
            sm.crypto()
                .map_err(|e| format!("Failed to create crypto: {}", e))?
        };
        let handle = crate::security::actor::CryptoHandle::new(crypto);
        let mut actor_guard = CRYPTO_ACTOR.write().await;
        *actor_guard = Some(handle);
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

    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    crate::maintenance::start_maintenance_scheduler(pool);
    SMTP_MANAGER.lock().await.start_outbox_worker();

    let existing_theme = load_config().and_then(|c| c.theme);
    save_config(&AppConfig {
        security_method: method.to_string(),
        theme: existing_theme,
    })?;

    // Load lock settings from db
    crate::security::load_lock_settings().await;

    // Load minimize-to-tray setting into global flag
    crate::cmd::settings::load_minimize_to_tray_setting().await;

    Ok(())
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
async fn initialize_tpm_elevated() -> Result<(), options::TpmInitError> {
    use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::time::Duration;
    use tokio::io::unix::AsyncFd;
    use tokio::process::Command;

    // Check if we're already running as helper mode to avoid infinite loops
    if std::env::var("POSTAIL_TPM_HELPER").is_ok() {
        return Err(options::TpmInitError {
            error_type: options::TpmErrorType::Other,
            message: "Already in helper mode".to_string(),
        });
    }

    // Define socket path
    let uid = unsafe { nix::libc::getuid() };
    let socket_path = std::path::PathBuf::from(format!("/run/user/{}/postail-tpm.sock", uid));

    // If socket already exists, try to PING it to check if it's really alive and has TPM access
    if socket_path.exists() {
        use crate::security::tpm::store::linux::LinuxTpmStore;
        if let Ok(store) = LinuxTpmStore::new() {
            if store.verify_proxy() {
                tracing::info!(target: "postail", "TPM helper already running and healthy.");
                return Ok(());
            }
        }
        // If it exists but is stale/unhealthy, try to restart it
        let _ = std::fs::remove_file(&socket_path);
    }

    let security_dir = crate::utils::config::get_data_dir().join("security");
    std::fs::create_dir_all(&security_dir).map_err(|e| options::TpmInitError {
        error_type: options::TpmErrorType::Other,
        message: format!("Failed to create security directory: {}", e),
    })?;

    let exe_path = if let Ok(appimage) = std::env::var("APPIMAGE") {
        std::path::PathBuf::from(appimage)
    } else {
        std::env::current_exe().map_err(|e| options::TpmInitError {
            error_type: options::TpmErrorType::Other,
            message: format!("Failed to get executable path: {}", e),
        })?
    }
    .to_string_lossy()
    .to_string();

    tracing::info!(target: "postail", "Requesting TPM elevation via pkexec (persistent helper)...");

    let socket_dir = socket_path.parent().ok_or_else(|| options::TpmInitError {
        error_type: options::TpmErrorType::Other,
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
        |e: nix::Error| options::TpmInitError {
            error_type: options::TpmErrorType::Other,
            message: format!("inotify init failed: {}", e),
        },
    )?;
    inotify
        .add_watch(
            socket_dir,
            AddWatchFlags::IN_CREATE | AddWatchFlags::IN_MOVED_TO,
        )
        .map_err(|e: nix::Error| options::TpmInitError {
            error_type: options::TpmErrorType::Other,
            message: format!("inotify add_watch failed: {}", e),
        })?;
    let async_inotify = AsyncFd::new(InotifyWrapper(inotify)).map_err(|e: std::io::Error| {
        options::TpmInitError {
            error_type: options::TpmErrorType::Other,
            message: format!("AsyncFd wrapper failed: {}", e),
        }
    })?;

    let (tx, mut rx) = tokio::sync::oneshot::channel::<options::TpmInitError>();

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
                        options::TpmErrorType::Cancelled,
                        "TPM elevation was cancelled by user".to_string(),
                    )
                } else {
                    (
                        options::TpmErrorType::HelperFailed,
                        format!("Persistent TPM helper failed with status: {}", s),
                    )
                };
                tracing::error!(target: "postail", "{}", err_msg);
                let _ = tx.send(options::TpmInitError {
                    error_type,
                    message: err_msg,
                });
            }
            Err(e) => {
                let err_msg = format!("Failed to start persistent TPM helper: {}", e);
                tracing::error!(target: "postail", "{}", err_msg);
                let _ = tx.send(options::TpmInitError {
                    error_type: options::TpmErrorType::StartFailed,
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
    let wait_res: Result<(), options::TpmInitError> = async {
        if socket_path.exists() {
            return Ok::<(), options::TpmInitError>(());
        }

        while !socket_path.exists() {
            tokio::select! {
                res = async_inotify.readable() => {
                    let mut guard = res.map_err(|e: std::io::Error| options::TpmInitError {
                        error_type: options::TpmErrorType::Other,
                        message: e.to_string()
                    })?;
                    let mut events = guard.get_inner().0.read_events()
                        .map_err(|e: nix::Error| options::TpmInitError {
                            error_type: options::TpmErrorType::Other,
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
                    return Err(err.unwrap_or_else(|_| options::TpmInitError {
                        error_type: options::TpmErrorType::HelperFailed,
                        message: "TPM helper failed to start".to_string()
                    }));
                }
            }
        }
        Ok::<(), options::TpmInitError>(())
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
        return Err(options::TpmInitError {
            error_type: options::TpmErrorType::SocketTimeout,
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
    #[cfg(all(target_os = "linux", feature = "tpm"))]
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
            #[cfg(all(target_os = "linux", feature = "tpm"))]
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
        if let Ok(pool) = get_db_pool().await {
            if let Ok(conn) = pool.get() {
                let count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
                    .unwrap_or(0);
                if count > 0 {
                    return Err("Cannot reset: accounts already exist.".to_string());
                }
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
        if let Some(tpm_store) = crate::security::tpm::store::get_tpm_store() {
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
pub fn get_security_method() -> Option<String> {
    load_config().map(|c| c.security_method)
}
