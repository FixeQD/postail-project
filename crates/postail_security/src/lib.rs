pub mod error;
pub use error::{Result, SecurityError};

pub mod crypto;
pub use crypto::{decrypt_with_key, encrypt_with_key, Crypto};
pub use crypto::helpers::{secure_zeroize, secure_zeroize_vec, ZeroizingBytes};

pub mod lock;
pub use lock::{
    get_timeout_minutes, is_lock_configured, is_locked, is_using_encryption_password,
    lock, record_activity, unlock,
    force_unlock, should_lock,
};

pub mod manager;
pub use manager::{PassphraseSecurityBuilder, SecurityManager};

pub mod master_key;
pub use master_key::{MasterKey, MASTER_KEY_LENGTH};

pub mod actor;
pub use actor::CryptoHandle;

pub mod recovery;

pub mod storage;
pub use storage::db::{DbEncryption, DbEncryptionError};
pub use storage::{SecretStore, StorageTier};

pub mod tpm;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use tpm::helper::tpm_helper_init;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use tpm::init::{TpmAvailability, TpmInitializer};
