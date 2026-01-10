#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

use crate::security::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::stores::SecretStore;

#[cfg(all(target_os = "linux", feature = "tpm"))]
pub fn get_tpm_store() -> Option<Box<dyn SecretStore>> {
    match linux::LinuxTpmStore::new() {
        Ok(store) if store.is_available() => Some(Box::new(store)),
        _ => None,
    }
}

#[cfg(all(target_os = "windows", feature = "tpm"))]
pub fn get_tpm_store() -> Option<Box<dyn SecretStore>> {
    match windows::WindowsTpmStore::new() {
        Ok(store) if store.is_available() => Some(Box::new(store)),
        _ => None,
    }
}

#[cfg(not(feature = "tpm"))]
pub fn get_tpm_store() -> Option<Box<dyn SecretStore>> {
    None
}

pub struct NoTpmStore;

impl SecretStore for NoTpmStore {
    fn store(&self, _key: &MasterKey) -> Result<()> {
        Err(SecurityError::NoSecureStorageAvailable)
    }

    fn retrieve(&self) -> Result<MasterKey> {
        Err(SecurityError::NoSecureStorageAvailable)
    }

    fn delete(&self) -> Result<()> {
        Err(SecurityError::NoSecureStorageAvailable)
    }

    fn is_available(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "TPM (unavailable)"
    }
}