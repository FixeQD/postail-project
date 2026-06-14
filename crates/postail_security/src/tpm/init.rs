use std::path::Path;

use crate::tpm::store::get_tpm_store;
use crate::storage::StorageTier;

pub enum TpmAvailability {
    Available,
    RequiresElevation,
    NotAvailable,
}

pub struct TpmInitializer;

impl Default for TpmInitializer {
    fn default() -> Self {
        Self::new()
    }
}

impl TpmInitializer {
    pub fn new() -> Self {
        Self
    }

    pub fn check_availability(&self) -> TpmAvailability {
        use crate::tpm::store::linux::LinuxTpmStore;
        use crate::storage::SecretStore;

        if Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists() {
            if let Ok(store) = LinuxTpmStore::new() {
                if store.is_available() {
                    if store.check_direct_access() {
                        return TpmAvailability::Available;
                    } else {
                        return TpmAvailability::RequiresElevation;
                    }
                }
            }
            return TpmAvailability::RequiresElevation;
        }

        TpmAvailability::NotAvailable
    }

    pub fn get_store(&self) -> Option<Box<dyn crate::storage::SecretStore + 'static>> {
        get_tpm_store()
    }

    pub fn get_storage_tier(&self) -> StorageTier {
        StorageTier::Tpm
    }
}
