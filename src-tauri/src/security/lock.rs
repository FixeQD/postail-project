use crate::db::settings::{get_setting, set_setting};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LockState {
    pub is_locked: bool,
    pub last_activity: Instant,
    pub timeout: Duration,
    pub pin_hash: Option<String>,
    pub use_encryption_password: bool,
}

impl Default for LockState {
    fn default() -> Self {
        Self {
            is_locked: false,
            last_activity: Instant::now(),
            timeout: Duration::from_secs(0),
            pin_hash: None,
            use_encryption_password: false,
        }
    }
}

pub static LOCK_STATE: Lazy<Arc<Mutex<LockState>>> =
    Lazy::new(|| Arc::new(Mutex::new(LockState::default())));

pub async fn load_settings() {
    let timeout_str = get_setting("lock_timeout_minutes").await.ok().flatten();
    let pin_hash = get_setting("lock_pin_hash").await.ok().flatten();
    let use_pass = get_setting("lock_use_encryption_password")
        .await
        .ok()
        .flatten();

    let mut state = LOCK_STATE.lock().unwrap();

    if let Some(s) = timeout_str {
        if let Ok(minutes) = s.parse::<u64>() {
            state.timeout = Duration::from_secs(minutes * 60);
        }
    }

    if let Some(hash) = pin_hash {
        if !hash.is_empty() {
            state.pin_hash = Some(hash);
        }
    }

    if let Some(flag) = use_pass {
        state.use_encryption_password = flag == "true";
    }
}

pub fn lock() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.is_locked = true;
}

pub fn unlock(password: &str) -> Result<(), String> {
    let mut state = LOCK_STATE.lock().unwrap();

    if state.use_encryption_password {
        // Encryption password path is verified externally via force_unlock_verified.
        return Err("Use unlock_with_encryption_password instead".to_string());
    }

    match &state.pin_hash.clone() {
        Some(stored_hash) => {
            let parsed = PasswordHash::new(stored_hash)
                .map_err(|_| "Invalid PIN hash stored".to_string())?;
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .map_err(|_| "Invalid PIN".to_string())?;
            state.is_locked = false;
            state.last_activity = Instant::now();
            Ok(())
        }
        None => Err("No PIN configured".to_string()),
    }
}

// Called after external verification (e.g. argon2 passphrase re-derivation).
pub fn force_unlock() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.is_locked = false;
    state.last_activity = Instant::now();
}

pub fn record_activity() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.last_activity = Instant::now();
}

pub fn is_locked() -> bool {
    let state = LOCK_STATE.lock().unwrap();
    state.is_locked
}

pub fn should_lock() -> bool {
    let state = LOCK_STATE.lock().unwrap();
    if state.is_locked {
        return false;
    }
    if state.timeout.as_secs() == 0 {
        return false;
    }
    if !state.use_encryption_password && state.pin_hash.is_none() {
        return false;
    }
    state.last_activity.elapsed() >= state.timeout
}

pub async fn set_timeout(minutes: u32) {
    {
        let mut state = LOCK_STATE.lock().unwrap();
        state.timeout = Duration::from_secs(minutes as u64 * 60);
        // Reset the countdown so the new timeout starts from now.
        state.last_activity = Instant::now();
    }
    let _ = set_setting("lock_timeout_minutes", &minutes.to_string()).await;
}

pub async fn set_pin(pin: &str) -> Result<(), String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash PIN: {e}"))?
        .to_string();

    {
        let mut state = LOCK_STATE.lock().unwrap();
        state.pin_hash = Some(hash.clone());
        state.use_encryption_password = false;
    }

    let _ = set_setting("lock_pin_hash", &hash).await;
    let _ = set_setting("lock_use_encryption_password", "false").await;
    Ok(())
}

pub async fn use_encryption_password() {
    {
        let mut state = LOCK_STATE.lock().unwrap();
        state.use_encryption_password = true;
        state.pin_hash = None;
    }
    let _ = set_setting("lock_use_encryption_password", "true").await;
    let _ = set_setting("lock_pin_hash", "").await;
}

pub fn get_timeout_minutes() -> u32 {
    let state = LOCK_STATE.lock().unwrap();
    (state.timeout.as_secs() / 60) as u32
}

pub fn is_using_encryption_password() -> bool {
    let state = LOCK_STATE.lock().unwrap();
    state.use_encryption_password
}

pub fn is_lock_configured() -> bool {
    let state = LOCK_STATE.lock().unwrap();
    state.use_encryption_password || state.pin_hash.is_some()
}
