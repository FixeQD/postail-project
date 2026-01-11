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

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("OAuth not implemented for {provider}")]
    NotImplemented { provider: String },

    #[error("Invalid or expired state")]
    InvalidState,

    #[error("Token exchange failed: {status}")]
    TokenExchangeFailed { status: String },

    #[error("Provider did not return a refresh_token. Cannot create persistent account.")]
    NoRefreshToken,

    #[error("Token refresh failed: {status}")]
    TokenRefreshFailed { status: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SecurityError>;
