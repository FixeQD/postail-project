use std::path::Path;

use crate::security::tpm::store::get_tpm_store;
use crate::security::storage::StorageTier;

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
        use crate::security::tpm::store::linux::LinuxTpmStore;
        use crate::security::storage::SecretStore;

        if Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists() {
            if let Ok(store) = LinuxTpmStore::new() {
                // First check if device exists
                if store.is_available() {
                    // Try to actually check direct access to the hardware.
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

    pub fn get_store(&self) -> Option<Box<dyn crate::security::storage::SecretStore + 'static>> {
        get_tpm_store()
    }

    pub fn get_storage_tier(&self) -> StorageTier {
        StorageTier::Tpm
    }
}
