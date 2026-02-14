pub mod crypto;
pub mod db_encryption;
pub mod lock;
pub mod lock_timer;
pub mod manager;
pub mod master_key;
pub mod recovery;
pub mod stores;
pub mod tpm_init;
pub mod zeroize_helpers;

pub use crypto::{decrypt_with_key, encrypt_with_key, Crypto};
pub use db_encryption::{DbEncryption, DbEncryptionError};
pub use lock::{
    get_timeout_minutes, is_lock_configured, is_locked, is_using_encryption_password,
    load_settings as load_lock_settings, lock, record_activity, set_pin, set_timeout, unlock,
    use_encryption_password,
};
pub use lock_timer::{start_lock_timer, stop_lock_timer};
pub use manager::{PassphraseSecurityBuilder, SecurityManager};
pub use master_key::{MasterKey, MASTER_KEY_LENGTH};
pub use stores::{SecretStore, StorageTier};
pub use tpm_init::{TpmAvailability, TpmInitializer};
pub use zeroize_helpers::{secure_zeroize, secure_zeroize_vec, ZeroizingBytes};
