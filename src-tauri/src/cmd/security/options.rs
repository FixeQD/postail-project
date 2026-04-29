use std::time::Duration;
use tauri::command;
use tokio::task::spawn_blocking;
use tokio::time::timeout;
use serde::Serialize;
use crate::security::storage::{keyring::KeyringStore, SecretStore};

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

#[derive(Serialize)]
pub struct SecurityOptions {
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    pub tpm_available: bool,
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    pub tpm_requires_elevation: bool,
    pub keyring_available: bool,
    pub argon2_available: bool,
}

#[command]
pub async fn check_security_options() -> Result<SecurityOptions, String> {
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    let (tpm_available, tpm_requires_elevation) = timeout(
        Duration::from_secs(3),
        spawn_blocking(|| {
            use crate::security::TpmInitializer;
            let initializer = TpmInitializer::new();
            let availability = initializer.check_availability();
            match availability {
                crate::security::TpmAvailability::Available => (true, false),
                crate::security::TpmAvailability::RequiresElevation => (true, true),
                crate::security::TpmAvailability::NotAvailable => (false, false),
            }
        }),
    )
    .await
    .unwrap_or(Ok((false, false)))
    .unwrap_or((false, false));

    let keyring_available = KeyringStore::new()
        .map(|k| k.is_available())
        .unwrap_or(false);

    Ok(SecurityOptions {
        #[cfg(all(target_os = "linux", feature = "tpm"))]
        tpm_available,
        #[cfg(all(target_os = "linux", feature = "tpm"))]
        tpm_requires_elevation,
        keyring_available,
        argon2_available: true,
    })
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
#[derive(Serialize)]
pub enum TpmStatus {
    Available,
    RequiresElevation,
    NotAvailable,
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
#[command]
pub async fn check_tpm_availability() -> Result<TpmStatus, String> {
    use crate::security::TpmAvailability;

    let result = timeout(
        Duration::from_secs(3),
        spawn_blocking(|| {
            let initializer = crate::security::TpmInitializer::new();
            initializer.check_availability()
        }),
    )
    .await
    .unwrap_or(Ok(TpmAvailability::NotAvailable))
    .unwrap_or(TpmAvailability::NotAvailable);

    match result {
        TpmAvailability::Available => Ok(TpmStatus::Available),
        TpmAvailability::RequiresElevation => Ok(TpmStatus::RequiresElevation),
        TpmAvailability::NotAvailable => Ok(TpmStatus::NotAvailable),
    }
}
