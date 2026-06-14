mod proto;
mod seal;
mod tbs;

use std::fs;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};
use crate::master_key::MasterKey;
use crate::storage::SecretStore;
use crate::tpm::store::paths;

use tbs::TbsContext;

pub struct WindowsTpmStore {
    storage_path: PathBuf,
}

impl WindowsTpmStore {
    pub fn new() -> Result<Self> {
        Self::with_storage_path(paths::default_storage_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
        Ok(Self { storage_path })
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join(paths::SEALED_FILE_NAME)
    }
}

impl SecretStore for WindowsTpmStore {
    fn store(&self, key: &MasterKey) -> Result<()> {
        let tbs = TbsContext::new()?;
        let primary = seal::create_primary_key(&tbs)?;

        let sealed = seal::seal_data(&tbs, primary, key.as_bytes());
        let _ = seal::flush_context(&tbs, primary);
        let sealed = sealed?;

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(self.get_sealed_path(), sealed)?;

        Ok(())
    }

    fn retrieve(&self) -> Result<MasterKey> {
        let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecurityError::MasterKeyNotFound
            } else {
                SecurityError::Io(e)
            }
        })?;

        let tbs = TbsContext::new()?;
        let primary = seal::create_primary_key(&tbs)?;

        let unsealed = seal::unseal_data(&tbs, primary, &sealed);
        let _ = seal::flush_context(&tbs, primary);

        MasterKey::from_bytes(&unsealed?)
    }

    fn delete(&self) -> Result<()> {
        let path = self.get_sealed_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn exists(&self) -> bool {
        self.get_sealed_path().exists()
    }

    fn is_available(&self) -> bool {
        TbsContext::new().is_ok()
    }

    fn name(&self) -> &'static str {
        "TPM2 (Windows)"
    }
}
