#[cfg(feature = "tpm")]
use tss_esapi::{tcti_ldr::TctiNameConf, Context};

#[cfg(feature = "tpm")]
use std::str::FromStr;

use std::fs;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::storage::SecretStore;

use super::common;

// ── WindowsTpmStore ────────────────────────────────────────────────

pub struct WindowsTpmStore {
    storage_path: PathBuf,
    #[cfg(feature = "tpm")]
    tcti: TctiNameConf,
}

impl WindowsTpmStore {
    pub fn new() -> Result<Self> {
        Self::with_storage_path(common::default_storage_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
        #[cfg(feature = "tpm")]
        {
            let tcti = TctiNameConf::from_str("tbs").map_err(common::tpm_err)?;
            Ok(Self { storage_path, tcti })
        }

        #[cfg(not(feature = "tpm"))]
        {
            Ok(Self { storage_path })
        }
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join(common::SEALED_FILE_NAME)
    }

    #[cfg(feature = "tpm")]
    fn create_context(&self) -> Result<Context> {
        Context::new(self.tcti.clone()).map_err(common::tpm_err)
    }
}

// ── SecretStore impl ───────────────────────────────────────────────

impl SecretStore for WindowsTpmStore {
    #[cfg(feature = "tpm")]
    fn store(&self, key: &MasterKey) -> Result<()> {
        let mut ctx = self.create_context()?;
        let primary = common::create_primary_key(&mut ctx)?;
        let sealed = common::seal_data(&mut ctx, primary.key_handle, key.as_bytes())?;

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(self.get_sealed_path(), sealed)?;

        ctx.flush_context(primary.key_handle.into())
            .map_err(common::tpm_err)?;
        Ok(())
    }

    #[cfg(not(feature = "tpm"))]
    fn store(&self, _key: &MasterKey) -> Result<()> {
        Err(SecurityError::Tpm("TPM support not compiled in".into()))
    }

    #[cfg(feature = "tpm")]
    fn retrieve(&self) -> Result<MasterKey> {
        let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecurityError::MasterKeyNotFound
            } else {
                SecurityError::Io(e)
            }
        })?;

        let mut ctx = self.create_context()?;
        let primary = common::create_primary_key(&mut ctx)?;
        let unsealed = common::unseal_data(&mut ctx, primary.key_handle, &sealed)?;

        ctx.flush_context(primary.key_handle.into())
            .map_err(common::tpm_err)?;

        MasterKey::from_bytes(&unsealed)
    }

    #[cfg(not(feature = "tpm"))]
    fn retrieve(&self) -> Result<MasterKey> {
        Err(SecurityError::Tpm("TPM support not compiled in".into()))
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
        #[cfg(feature = "tpm")]
        {
            self.create_context()
                .map(|mut ctx| ctx.get_random(8).is_ok())
                .unwrap_or(false)
        }

        #[cfg(not(feature = "tpm"))]
        {
            false
        }
    }

    fn name(&self) -> &'static str {
        "TPM2 (Windows)"
    }
}
