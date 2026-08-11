//! Bits shared by every TPM backend

use std::path::PathBuf;

use crate::error::{SecurityError, TpmErrorKind};

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

/// Pulls the `0x........` TPM2 response code out of one of `proto.rs`'s `"TPM2 error: 0x........"` messages, if present
fn extract_rc(msg: &str) -> Option<u32> {
    let (_, hex) = msg.split_once("0x")?;
    let hex = hex.get(0..8)?;
    u32::from_str_radix(hex, 16).ok()
}

/// Single choke point every TPM backend (Linux and Windows) funnels its raw protocol errors through
/// Tries to recognize a TPM2 response code in the message and produce a classified `SecurityError::TpmCode`
pub fn tpm_err(e: impl std::fmt::Display) -> SecurityError {
    let message = e.to_string();

    match extract_rc(&message) {
        Some(code) => {
            let kind = TpmErrorKind::from_rc(code);
            if kind == TpmErrorKind::Unknown {
                SecurityError::Tpm(message)
            } else {
                SecurityError::TpmCode {
                    kind,
                    code,
                    message,
                }
            }
        }
        None => SecurityError::Tpm(message),
    }
}
