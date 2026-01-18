use std::path::Path;

use crate::security::stores::StorageTier;
use crate::security::stores::tpm::get_tpm_store;

pub enum TpmAvailability {
    Available,
    RequiresElevation,
    NotAvailable,
}

pub struct TpmInitializer;

impl TpmInitializer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn check_availability(&self) -> TpmAvailability {
        let store = get_tpm_store();
        
        if store.is_some() && store.as_ref().unwrap().is_available() {
            return TpmAvailability::Available;
        }
        
        if Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists() {
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
