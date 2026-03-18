#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(all(target_os = "windows", feature = "tpm"))]
pub mod windows;

#[cfg(feature = "tpm")]
pub mod common;

#[cfg(feature = "tpm")]
pub mod pcr;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::storage::SecretStore;

#[cfg(all(target_os = "linux", feature = "tpm"))]
pub fn get_tpm_store() -> Option<Box<dyn SecretStore>> {
    use crate::security::storage::SecretStore;
    if std::path::Path::new("/dev/tpmrm0").exists() || std::path::Path::new("/dev/tpm0").exists() {
        linux::LinuxTpmStore::new()
            .ok()
            .map(|store| Box::new(store) as Box<dyn SecretStore>)
    } else {
        None
    }
}

#[cfg(not(all(target_os = "linux", feature = "tpm")))]
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

    fn exists(&self) -> bool {
        false
    }

    fn is_available(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "TPM (unavailable)"
    }
}
