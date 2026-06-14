use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::sync::LazyLock;
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

pub static LOCK_STATE: LazyLock<Arc<Mutex<LockState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(LockState::default())));

pub fn lock() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.is_locked = true;
}

pub fn unlock(password: &str) -> Result<(), String> {
    let mut state = LOCK_STATE.lock().unwrap();

    if state.use_encryption_password {
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

// ── State-setting helpers for main app wrappers ────────────────────

pub fn apply_settings_state(timeout_secs: u64, pin_hash: Option<String>, use_encryption_password: bool) {
    let mut state = LOCK_STATE.lock().unwrap();
    state.timeout = Duration::from_secs(timeout_secs);
    state.pin_hash = pin_hash;
    state.use_encryption_password = use_encryption_password;
}

pub fn apply_set_timeout_state(minutes: u32) {
    let mut state = LOCK_STATE.lock().unwrap();
    state.timeout = Duration::from_secs(minutes as u64 * 60);
    state.last_activity = Instant::now();
}

pub fn hash_pin(pin: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash PIN: {e}"))?
        .to_string();
    Ok(hash)
}

pub fn apply_set_pin_state(hash: String) {
    let mut state = LOCK_STATE.lock().unwrap();
    state.pin_hash = Some(hash);
    state.use_encryption_password = false;
}

pub fn apply_use_encryption_password_state() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.use_encryption_password = true;
    state.pin_hash = None;
}
