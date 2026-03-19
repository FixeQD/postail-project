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

impl From<OAuthError> for String {
    fn from(err: OAuthError) -> Self {
        err.to_string()
    }
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("EML cache error: {0}")]
    EmlCache(String),

    #[error("Body cache error: {0}")]
    BodyCache(String),
}

pub type Result<T> = std::result::Result<T, SecurityError>;

#[derive(Debug, Error)]
pub enum ImapError {
    #[error("IMAP connection failed: {0}")]
    Connection(String),

    #[error("IMAP login failed: {0}")]
    Login(String),

    #[error("OAuth refresh failed: {0}")]
    OAuthRefresh(String),

    #[error("Unknown OAuth provider")]
    UnknownOAuthProvider,

    #[error("Failed to fetch credentials: {0}")]
    CredentialsFetch(String),

    #[error("IDLE not supported for mailbox {mailbox}")]
    IdleNotSupported { mailbox: String },

    #[error("IDLE init failed for {mailbox}: {error}")]
    IdleInitFailed { mailbox: String, error: String },

    #[error("IDLE wait error for {mailbox}: {error}")]
    IdleWaitError { mailbox: String, error: String },

    #[error("IDLE timeout for {mailbox}")]
    IdleTimeout { mailbox: String },

    #[error("IDLE reinit failed for {mailbox}")]
    IdleReinitFailed { mailbox: String },

    #[error("Mailbox sync error for {mailbox}: {error}")]
    MailboxSyncError { mailbox: String, error: String },

    #[error("No sync running for account {account_id}")]
    NoSyncRunning { account_id: String },

    #[error("Sync thread join failed for {account_id}: {error}")]
    SyncThreadJoinFailed { account_id: String, error: String },

    #[error("Failed to join sync thread for {account_id}: {error}")]
    FailedToJoinThread { account_id: String, error: String },
}

impl From<ImapError> for String {
    fn from(err: ImapError) -> Self {
        err.to_string()
    }
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Sync loop failed for account {account_id}: {error}")]
    SyncLoopFailed { account_id: String, error: String },

    #[error("Sync thread panic for account {account_id}: {error}")]
    SyncThreadPanic { account_id: String, error: String },
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unknown OAuth provider")]
    UnknownOAuthProvider,

    #[error("Failed to fetch Gmail user info: {0}")]
    GmailUserInfoFailed(String),

    #[error("Failed to fetch Outlook user info: {0}")]
    OutlookUserInfoFailed(String),

    #[error("No email in OAuth response")]
    NoEmailInResponse,

    #[error("TPM not available or not supported")]
    TpmNotAvailable,

    #[error("Keyring not available")]
    KeyringNotAvailable,

    #[error("Invalid security method: {0}")]
    InvalidSecurityMethod(String),

    #[error("Failed to get database encryption key")]
    DbEncryptionKeyFailed,

    #[error("Database encryption verification failed")]
    DbEncryptionVerifyFailed,

    #[error("Database migration failed: {0}")]
    MigrationFailed(String),

    #[error("Encrypted database file was not created")]
    EncryptedDbNotCreated,

    #[error("Encrypted database integrity check failed")]
    EncryptedDbIntegrityFailed,

    #[error("Tables not found after migration")]
    TablesNotFound,

    #[error("Failed to derive key")]
    KeyDerivationFailed,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("IMAP error: {0}")]
    Imap(#[from] ImapError),

    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Imap(ImapError::Connection(err))
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Imap(ImapError::Connection(err.to_string()))
    }
}

impl From<OAuthError> for AppError {
    fn from(err: OAuthError) -> Self {
        match err {
            OAuthError::NoRefreshToken => AppError::NoEmailInResponse,
            OAuthError::NotImplemented { provider } => {
                AppError::GmailUserInfoFailed(format!("OAuth not implemented for {}", provider))
            }
            OAuthError::InvalidState => AppError::GmailUserInfoFailed("Invalid OAuth state".into()),
            OAuthError::TokenExchangeFailed { status } => {
                AppError::GmailUserInfoFailed(format!("Token exchange failed: {}", status))
            }
            OAuthError::TokenRefreshFailed { status } => {
                AppError::GmailUserInfoFailed(format!("Token refresh failed: {}", status))
            }
            OAuthError::Http(e) => AppError::GmailUserInfoFailed(format!("HTTP error: {}", e)),
            OAuthError::UrlParse(e) => {
                AppError::GmailUserInfoFailed(format!("URL parse error: {}", e))
            }
        }
    }
}

impl From<DBError> for AppError {
    fn from(err: DBError) -> Self {
        match err {
            DBError::Migration(e) => AppError::MigrationFailed(e),
            DBError::Sqlite(e) => AppError::DatabaseError(e.to_string()),
            DBError::Io(e) => AppError::IoError(e.to_string()),
            DBError::Security(e) => AppError::GmailUserInfoFailed(format!("Security error: {}", e)),
            DBError::Json(e) => AppError::DatabaseError(format!("JSON error: {}", e)),
            DBError::Cache(e) => AppError::IoError(format!("Cache error: {}", e)),
            DBError::EmlCache(e) => AppError::IoError(format!("EML cache error: {}", e)),
            DBError::BodyCache(e) => AppError::IoError(format!("Body cache error: {}", e)),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

impl From<async_imap::error::Error> for AppError {
    fn from(err: async_imap::error::Error) -> Self {
        let msg = err.to_string();
        if msg.contains("connection") || msg.contains("Connection") {
            AppError::Imap(ImapError::Connection(msg))
        } else if msg.contains("login") || msg.contains("Login") || msg.contains("authentication") {
            AppError::Imap(ImapError::Login(msg))
        } else {
            AppError::Imap(ImapError::Connection(msg))
        }
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("server returned {status} for {url}")]
    BadStatus { status: u16, url: String },

    #[error("resource at {url} exceeds size limit ({size} bytes)")]
    TooLarge { url: String, size: usize },
}

impl From<NetworkError> for String {
    fn from(err: NetworkError) -> Self {
        err.to_string()
    }
}
