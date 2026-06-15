use crate::security::storage::{SecretStore, keyring::KeyringStore};
use serde::Serialize;
#[cfg(feature = "tpm")]
use std::time::Duration;
use tauri::command;
#[cfg(feature = "tpm")]
use tokio::task::spawn_blocking;
#[cfg(feature = "tpm")]
use tokio::time::timeout;

// ---------------------------------------------------------------------------
// TPM error types
// ---------------------------------------------------------------------------

#[cfg(all(target_os = "linux", feature = "tpm"))]
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TpmErrorType {
    Cancelled,
    AccessDenied,
    HelperFailed,
    StartFailed,
    SocketTimeout,
    Other,
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
#[derive(Serialize, Debug, Clone)]
pub struct TpmInitError {
    pub error_type: TpmErrorType,
    pub message: String,
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
impl std::fmt::Display for TpmInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
impl From<TpmInitError> for String {
    fn from(err: TpmInitError) -> Self {
        serde_json::to_string(&err).unwrap_or(err.message)
    }
}

#[cfg(feature = "tpm")]
#[derive(Serialize, Clone, Copy)]
pub enum TpmStatus {
    Available,
    RequiresElevation,
    NotAvailable,
}

#[cfg(feature = "tpm")]
impl From<crate::security::TpmAvailability> for TpmStatus {
    fn from(availability: crate::security::TpmAvailability) -> Self {
        match availability {
            crate::security::TpmAvailability::Available => TpmStatus::Available,
            crate::security::TpmAvailability::RequiresElevation => TpmStatus::RequiresElevation,
            crate::security::TpmAvailability::NotAvailable => TpmStatus::NotAvailable,
        }
    }
}

#[cfg(feature = "tpm")]
async fn tpm_check() -> TpmStatus {
    timeout(
        Duration::from_secs(3),
        spawn_blocking(|| crate::security::TpmInitializer::new().check_availability()),
    )
    .await
    .unwrap_or(Ok(crate::security::TpmAvailability::NotAvailable))
    .unwrap_or(crate::security::TpmAvailability::NotAvailable)
    .into()
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SecurityOptions {
    #[cfg(feature = "tpm")]
    pub tpm_available: bool,
    #[cfg(feature = "tpm")]
    pub tpm_requires_elevation: bool,
    pub keyring_available: bool,
    pub argon2_available: bool,
}

#[command]
pub async fn check_security_options() -> Result<SecurityOptions, String> {
    #[cfg(feature = "tpm")]
    let (tpm_available, tpm_requires_elevation) = match tpm_check().await {
        TpmStatus::Available => (true, false),
        TpmStatus::RequiresElevation => (true, true),
        TpmStatus::NotAvailable => (false, false),
    };

    let keyring_available = KeyringStore::new()
        .map(|k| k.is_available())
        .unwrap_or(false);

    Ok(SecurityOptions {
        #[cfg(feature = "tpm")]
        tpm_available,
        #[cfg(feature = "tpm")]
        tpm_requires_elevation,
        keyring_available,
        argon2_available: true,
    })
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
#[command]
pub async fn check_tpm_availability() -> Result<TpmStatus, String> {
    Ok(tpm_check().await)
}
