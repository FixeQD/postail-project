use std::path::Path;

use crate::security::stores::tpm::get_tpm_store;
use crate::security::stores::StorageTier;

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
        use crate::security::stores::tpm::linux::LinuxTpmStore;
        use crate::security::stores::SecretStore;

        if Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists() {
            if let Ok(store) = LinuxTpmStore::new() {
                // First check if device exists
                if store.is_available() {
                    // Try to actually create context to check real access
                    if store.check_context_silent() {
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

    pub fn get_store(&self) -> Option<Box<dyn crate::security::stores::SecretStore + 'static>> {
        get_tpm_store()
    }

    pub fn get_storage_tier(&self) -> StorageTier {
        StorageTier::Tpm
    }
}
