use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("keyring error: {0}")]
    Keyring(String),

    #[error("TPM error: {0}")]
    Tpm(String),

    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed: {0}")]
    Decryption(String),

    #[error("key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("master key not found")]
    MasterKeyNotFound,

    #[error("master key already exists")]
    MasterKeyAlreadyExists,

    #[error("invalid passphrase")]
    InvalidPassphrase,

    #[error("no secure storage available")]
    NoSecureStorageAvailable,

    #[error("invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("invalid nonce length")]
    InvalidNonceLength,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SecurityError>;