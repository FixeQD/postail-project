pub mod crypto;
pub mod db_encryption;
pub mod manager;
pub mod master_key;
pub mod stores;
pub mod tpm_init;
pub mod zeroize_helpers;

pub use crypto::{decrypt_with_key, encrypt_with_key, Crypto};
pub use db_encryption::{DbEncryption, DbEncryptionError};
pub use manager::{PassphraseSecurityBuilder, SecurityManager};
pub use master_key::{MasterKey, MASTER_KEY_LENGTH};
pub use stores::{SecretStore, StorageTier};
pub use zeroize_helpers::{secure_zeroize, secure_zeroize_vec, ZeroizingBytes};
pub use tpm_init::{TpmAvailability, TpmInitializer};
