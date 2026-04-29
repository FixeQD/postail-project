use tauri::command;
use crate::globals::{SECURITY, DB_CONN, SMTP_MANAGER};
use crate::security::{DbEncryption, SecurityManager};
use crate::security::storage::StorageTier;
use std::sync::Arc;

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

    let pool = crate::globals::get_db_pool().await.map_err(|e| e.to_string())?;
    crate::maintenance::start_maintenance_scheduler(pool);
    SMTP_MANAGER.lock().await.start_outbox_worker();
    crate::security::load_lock_settings().await;

    tracing::info!(target: "postail", "Unlocked via recovery phrase");
    Ok(())
}

#[command]
pub fn verify_recovery_words(indices: Vec<usize>, words: Vec<String>) -> Result<bool, String> {
    crate::security::recovery::verify_pending_phrase(&indices, &words).map_err(|e| e.to_string())
}
