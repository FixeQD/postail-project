use serde::Serialize;
use thiserror::Error;

/// Named TPM2 "format-one" response codes (TCG spec §6.6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TpmErrorKind {
    /// TPM_RC_INTEGRITY (0x1F) - sealed blob doesn't match current primary key
    /// Usually means the TPM got cleared
    /// Data isn't recoverable via TPM anymore
    Integrity,
    /// TPM_RC_POLICY_FAIL (0x1D) - PCR7 changed since sealing (boot state changed)
    /// Data is still intact, just needs the right boot state again
    PolicyFail,
    /// TPM_RC_BAD_AUTH (0x22)
    BadAuth,
    /// Anything else
    Unknown,
}

impl TpmErrorKind {
    /// Classify a raw TPM2 format-one response code.
    pub fn from_rc(code: u32) -> Self {
        if code & 0x80 == 0 {
            return Self::Unknown;
        }
        match code & 0x3F {
            0x1F => Self::Integrity,
            0x1D => Self::PolicyFail,
            0x22 => Self::BadAuth,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("keyring error: {0}")]
    Keyring(String),

    /// Free-text TPM error
    #[error("{0}")]
    Tpm(String),

    /// TPM error classified to a specific TPM2 response code
    /// `message` already contains the raw code
    #[error("[{kind:?}] {message}")]
    TpmCode {
        kind: TpmErrorKind,
        code: u32,
        message: String,
    },

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

    #[error("crypto actor is not running")]
    ActorDead,

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

/// JSON shape for the frontend: `{ errorType, message }`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FrontendTpmError<'a> {
    error_type: TpmErrorKind,
    message: &'a str,
}

impl SecurityError {
    /// Serializes classified TPM errors to `{ errorType, message }` JSON;
    /// falls back to the plain message for everything else.
    pub fn to_frontend_json(&self) -> String {
        match self {
            SecurityError::TpmCode { kind, message, .. } => {
                serde_json::to_string(&FrontendTpmError {
                    error_type: *kind,
                    message,
                })
                .unwrap_or_else(|_| message.clone())
            }
            other => other.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, SecurityError>;
