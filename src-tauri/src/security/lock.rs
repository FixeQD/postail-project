use crate::db::settings::{get_setting, set_setting};
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
            timeout: Duration::from_secs(0), // Disabled by default (0 = no auto-lock)
            pin_hash: None,
            use_encryption_password: false,
        }
    }
}

static LOCK_STATE: Lazy<Arc<Mutex<LockState>>> =
    Lazy::new(|| Arc::new(Mutex::new(LockState::default())));

pub fn load_settings() {
    let mut state = LOCK_STATE.lock().unwrap();

    // Load timeout (default 0/disabled if not set)
    if let Ok(Some(timeout_str)) = get_setting("lock_timeout_minutes") {
        if let Ok(minutes) = timeout_str.parse::<u64>() {
            state.timeout = Duration::from_secs(minutes * 60);
        }
    }

    // Load PIN hash
    if let Ok(Some(pin_hash)) = get_setting("lock_pin_hash") {
        if !pin_hash.is_empty() {
            state.pin_hash = Some(pin_hash);
        }
    }

    // Load encryption password flag
    if let Ok(Some(use_pass)) = get_setting("lock_use_encryption_password") {
        state.use_encryption_password = use_pass == "true";
    }
}

pub fn lock() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.is_locked = true;
}

pub fn unlock(password: &str, db_password: Option<&str>) -> Result<(), String> {
    let mut state = LOCK_STATE.lock().unwrap();

    if state.use_encryption_password {
        match db_password {
            Some(db_pass) if db_pass == password => {
                state.is_locked = false;
                state.last_activity = Instant::now();
                Ok(())
            }
            _ => Err("Invalid password".to_string()),
        }
    } else {
        match &state.pin_hash {
            Some(hash) => {
                if hash == password {
                    state.is_locked = false;
                    state.last_activity = Instant::now();
                    Ok(())
                } else {
                    Err("Invalid PIN".to_string())
                }
            }
            None => Err("No PIN set".to_string()),
        }
    }
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
    // Don't lock if timeout is 0 (disabled)
    if state.timeout.as_secs() == 0 {
        return false;
    }
    // Don't lock if no PIN is configured
    if !state.use_encryption_password && state.pin_hash.is_none() {
        return false;
    }
    state.last_activity.elapsed() >= state.timeout
}

pub fn set_timeout(minutes: u32) {
    let mut state = LOCK_STATE.lock().unwrap();
    state.timeout = Duration::from_secs(minutes as u64 * 60);
    // Persist to database
    let _ = set_setting("lock_timeout_minutes", &minutes.to_string());
}

pub fn set_pin(pin: &str) {
    let mut state = LOCK_STATE.lock().unwrap();
    state.pin_hash = Some(pin.to_string());
    state.use_encryption_password = false;
    // Persist to database
    let _ = set_setting("lock_pin_hash", pin);
    let _ = set_setting("lock_use_encryption_password", "false");
}

pub fn use_encryption_password() {
    let mut state = LOCK_STATE.lock().unwrap();
    state.use_encryption_password = true;
    state.pin_hash = None;
    // Persist to database
    let _ = set_setting("lock_use_encryption_password", "true");
    let _ = set_setting("lock_pin_hash", ""); // Clear PIN
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
