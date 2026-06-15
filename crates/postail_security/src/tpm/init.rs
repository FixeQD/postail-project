use crate::storage::StorageTier;
use crate::tpm::store::get_tpm_store;

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
        // ── Linux ────────────────────────────────────────────────────────────
        #[cfg(target_os = "linux")]
        {
            use crate::storage::SecretStore;
            use crate::tpm::store::linux::LinuxTpmStore;
            use std::path::Path;

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
        }

        // ── Windows ──────────────────────────────────────────────────────────
        #[cfg(target_os = "windows")]
        {
            use crate::storage::SecretStore;
            use crate::tpm::store::windows::WindowsTpmStore;

            if let Ok(store) = WindowsTpmStore::new() {
                if store.is_available() {
                    return TpmAvailability::Available;
                }
            }
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
