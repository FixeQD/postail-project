// Crypto
pub mod crypto;
pub use crypto::{decrypt_with_key, encrypt_with_key, Crypto};
pub use crypto::helpers::{secure_zeroize, secure_zeroize_vec, ZeroizingBytes};

// Lock
pub mod lock;
pub use lock::{
    get_timeout_minutes, is_lock_configured, is_locked, is_using_encryption_password,
    load_settings as load_lock_settings, lock, record_activity, set_pin, set_timeout, unlock,
    use_encryption_password,
};
pub use lock::timer::{start_lock_timer, stop_lock_timer};

// Manager & Master Key
pub mod manager;
pub use manager::{PassphraseSecurityBuilder, SecurityManager};

pub mod master_key;
pub use master_key::{MasterKey, MASTER_KEY_LENGTH};

// Actor
pub mod actor;
pub use actor::CryptoHandle;

// Recovery
pub mod recovery;

// Storage
pub mod storage;
pub use storage::db::{DbEncryption, DbEncryptionError};
pub use storage::{SecretStore, StorageTier};

// TPM
pub mod tpm;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use tpm::helper::tpm_helper_init;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use tpm::init::{TpmAvailability, TpmInitializer};
