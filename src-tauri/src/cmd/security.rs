use crate::globals::{DB_CONN, SECURITY, SMTP_MANAGER};
use crate::security::manager::PassphraseSecurityBuilder;
use crate::security::stores::keyring::KeyringStore;
use crate::security::stores::tpm::get_tpm_store;
use crate::security::stores::{SecretStore, StorageTier};
use crate::security::{DbEncryption, SecurityManager};
use crate::security::recovery::RecoveryStore;
use crate::utils::config::{load_config, save_config, AppConfig};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::command;
use tokio::task::spawn_blocking;
use tokio::time::timeout;

#[derive(Serialize)]
pub struct SecurityOptions {
    pub tpm_available: bool,
    pub keyring_available: bool,
    pub argon2_available: bool,
}

#[command]
pub async fn check_security_options() -> Result<SecurityOptions, String> {
    let tpm_available = timeout(
        Duration::from_millis(500),
        spawn_blocking(|| get_tpm_store().is_some()),
    )
    .await
    .unwrap_or(Ok(false))
    .unwrap_or(false);

    let keyring_available = KeyringStore::new()
        .map(|k| k.is_available())
        .unwrap_or(false);

    Ok(SecurityOptions {
        tpm_available,
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
    let initializer = crate::security::TpmInitializer::new();
    match initializer.check_availability() {
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

pub fn initialize_security_and_database(
    method: &str,
    passphrase: Option<String>,
    recovery_phrase: Option<String>,
) -> Result<(), String> {
    tracing::info!(target: "postail", "Initializing security with method: {}", method);

    let security = match method {
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
    {
        let mut security_guard = SECURITY.lock().unwrap();
        *security_guard = security;

        if is_unlocking {
            security_guard.unlock().map_err(|e| e.to_string())?;
        } else {
            security_guard.initialize().map_err(|e| e.to_string())?;
        }
    }

    // create recovery store if phrase was provided (argon2 setup only)
    if let Some(phrase) = recovery_phrase {
        let storage_path = crate::utils::config::get_data_dir().join("security");
        let recovery_store = RecoveryStore::new(storage_path);
        let security = SECURITY.lock().unwrap();
        let master_key = security.export_master_key().map_err(|e| e.to_string())?;
        recovery_store.create(&master_key, &phrase).map_err(|e| e.to_string())?;
        tracing::info!(target: "postail", "Recovery store created");
    }

    let encryption = {
        let security = SECURITY.lock().unwrap();
        let master_key_raw = security.get_master_key_raw();
        DbEncryption::derive_from_master_key(&master_key_raw).map_err(|e| e.to_string())?
    };
    let hex_key = encryption.hex_key();

    let data_dir = crate::utils::config::get_data_dir();
    let db_path = data_dir.join("postail.db");

    let db = if db_path.exists() {
        crate::db::connect_db_with_key(&hex_key).map_err(|e| e.to_string())?
    } else {
        crate::db::init_db_with_key(&hex_key).map_err(|e| e.to_string())?
    };

    {
        let mut db_guard = DB_CONN.lock().unwrap();
        *db_guard = Some(db);
    }

    crate::maintenance::start_maintenance_scheduler(Arc::clone(&DB_CONN));
    SMTP_MANAGER.lock().unwrap().start_outbox_worker();

    let existing_theme = load_config().and_then(|c| c.theme);
    save_config(&AppConfig {
        security_method: method.to_string(),
        theme: existing_theme,
    })?;

    // Load lock settings from db
    crate::security::load_lock_settings();

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
    initialize_security_and_database(&method, passphrase, recovery_phrase)
}

#[command]
pub fn generate_recovery_phrase() -> String {
    crate::security::recovery::generate_phrase()
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
pub fn unlock_app(password: String) -> Result<(), String> {
    let security = SECURITY.lock().unwrap();
    let db_password = if security.is_initialized() {
        Some(hex::encode(security.get_master_key_raw()))
    } else {
        None
    };
    unlock(&password, db_password.as_deref())
}

#[command]
pub fn set_auto_lock_timeout(minutes: u32) {
    set_timeout(minutes);
}

#[command]
pub fn get_auto_lock_timeout() -> u32 {
    get_timeout_minutes()
}

#[command]
pub fn set_auto_lock_pin(pin: String) {
    set_pin(&pin);
}

#[command]
pub fn use_encryption_password_for_lock() {
    use_encryption_password();
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
