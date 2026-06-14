//! Bits shared by every TPM backend

use std::path::PathBuf;

use crate::error::SecurityError;

pub const SEALED_FILE_NAME: &str = "master_key.tpm";

pub fn default_storage_path() -> PathBuf {
    if let Ok(dir) = std::env::var("POSTAIL_DATA_DIR") {
        return PathBuf::from(dir).join("security");
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("security")
}

pub fn tpm_err(e: impl std::fmt::Display) -> SecurityError {
    SecurityError::Tpm(e.to_string())
}
