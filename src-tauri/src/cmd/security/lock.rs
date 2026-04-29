use tauri::command;
use crate::globals::SECURITY;
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
    if is_using_encryption_password() {
        // Re-derive the master key from the provided passphrase and compare against the one currently held in memory
        let storage_path = crate::utils::config::get_data_dir().join("security");
        let store =
            crate::security::storage::argon2::Argon2Store::new(storage_path, password.clone());

        use crate::security::storage::SecretStore;
        let retrieved = store
            .retrieve()
            .map_err(|_| "Invalid password".to_string())?;

        let security = SECURITY.lock().await;
        let current_key = security.get_master_key_raw();
        if retrieved.as_bytes() != current_key.as_slice() {
            return Err("Invalid password".to_string());
        }
        drop(security);

        crate::security::lock::force_unlock();
        Ok(())
    } else {
        unlock(&password)
    }
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
pub async fn set_auto_lock_pin(pin: String) -> Result<(), String> {
    set_pin(&pin).await
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
