use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed: {0}")]
    Decryption(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("metadata serialization failed: {0}")]
    Metadata(String),
}

impl From<CacheError> for String {
    fn from(err: CacheError) -> Self {
        err.to_string()
    }
}
