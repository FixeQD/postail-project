pub mod error;
pub use error::{Result, SecurityError};

pub mod crypto;
pub use crypto::helpers::{ZeroizingBytes, secure_zeroize, secure_zeroize_vec};
pub use crypto::{Crypto, decrypt_with_key, encrypt_with_key};

pub mod lock;
pub use lock::{
    force_unlock, get_timeout_minutes, is_lock_configured, is_locked, is_using_encryption_password,
    lock, record_activity, should_lock, unlock,
};

pub mod manager;
pub use manager::{PassphraseSecurityBuilder, SecurityManager};

pub mod master_key;
pub use master_key::{MASTER_KEY_LENGTH, MasterKey};

pub mod actor;
pub use actor::CryptoHandle;

pub mod recovery;

pub mod storage;
pub use storage::db::{DbEncryption, DbEncryptionError};
pub use storage::{SecretStore, StorageTier};

pub mod tpm;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use tpm::helper::tpm_helper_init;
#[cfg(feature = "tpm")]
pub use tpm::init::{TpmAvailability, TpmInitializer};
