// Re-export all public items from the postail_security crate
pub use postail_security::*;

// Re-export modules so sub-paths like crate::security::storage::keyring::KeyringStore work
pub use postail_security::{actor, crypto, lock, manager, master_key, recovery, storage, tpm};

// Keep the Tauri-specific lock timer in the main app
pub mod lock_timer;
pub use lock_timer::{start_lock_timer, stop_lock_timer};

// Settings-persistence wrappers for lock module
pub mod lock_settings;
pub use lock_settings::{
    load_settings as load_lock_settings, set_pin, set_timeout, use_encryption_password,
};
