pub mod context;
pub mod proxy;
pub mod seal;

use std::fs;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};
use crate::master_key::MasterKey;
use crate::storage::SecretStore;
use crate::tpm::store::paths;

use context::LinuxTpmContext;

pub struct LinuxTpmStore {
    storage_path: PathBuf,
    ctx: LinuxTpmContext,
}

impl LinuxTpmStore {
    pub fn new() -> Result<Self> {
        Self::with_storage_path(paths::default_storage_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
        Ok(Self {
            storage_path,
            ctx: LinuxTpmContext::new()?,
        })
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join(paths::SEALED_FILE_NAME)
    }

    pub fn create_context(&self) -> Result<tss_esapi::Context> {
        self.ctx.create_context()
    }

    pub fn check_context_silent(&self) -> bool {
        self.ctx.check_direct_access() || {
            #[cfg(target_os = "linux")]
            {
                std::env::var("POSTAIL_TPM_HELPER").is_err() && self.verify_proxy()
            }
            #[cfg(not(target_os = "linux"))]
            {
                false
            }
        }
    }

    pub fn check_direct_access(&self) -> bool {
        self.ctx.check_direct_access()
    }

    pub fn check_needs_elevation(&self) -> bool {
        if self.ctx.check_direct_access() {
            return false;
        }
        #[cfg(target_os = "linux")]
        {
            if proxy::is_socket_alive() {
                return false;
            }
        }
        true
    }

    pub fn verify_proxy(&self) -> bool {
        proxy::call_proxy(crate::tpm::protocol::TpmRequest::Ping).is_ok()
    }

    fn seal_and_write(&self, key: &MasterKey) -> Result<()> {
        let mut ctx = self.ctx.create_context()?;
        let primary = seal::create_primary_key(&mut ctx)?;
        let sealed = seal::seal_data(&mut ctx, primary.key_handle, key.as_bytes())?;

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(self.get_sealed_path(), sealed)?;

        ctx.flush_context(primary.key_handle.into())
            .map_err(paths::tpm_err)?;
        Ok(())
    }
}

impl SecretStore for LinuxTpmStore {
    fn store(&self, key: &MasterKey) -> Result<()> {
        if self.ctx.create_context().is_ok() {
            return self.seal_and_write(key);
        }

        #[cfg(target_os = "linux")]
        if proxy::is_socket_alive() {
            let sealed = proxy::call_proxy(crate::tpm::protocol::TpmRequest::Seal {
                key: key.as_bytes().to_vec(),
            })
            .map_err(SecurityError::Tpm)?
            .ok_or_else(|| SecurityError::Tpm("No sealed data returned from helper".into()))?;

            if let Some(parent) = self.get_sealed_path().parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(self.get_sealed_path(), sealed)?;
            return Ok(());
        }

        Err(SecurityError::Tpm(
            "TPM context unavailable and no helper running".into(),
        ))
    }

    fn retrieve(&self) -> Result<MasterKey> {
        match self.ctx.create_context() {
            Ok(mut ctx) => {
                let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        SecurityError::MasterKeyNotFound
                    } else {
                        SecurityError::Io(e)
                    }
                })?;

                let primary = seal::create_primary_key(&mut ctx)?;
                let unsealed = seal::unseal_data(&mut ctx, primary.key_handle, &sealed)?;

                ctx.flush_context(primary.key_handle.into())
                    .map_err(paths::tpm_err)?;

                MasterKey::from_bytes(&unsealed)
            }
            Err(_) => {
                #[cfg(target_os = "linux")]
                if std::env::var("POSTAIL_TPM_HELPER").is_err() && proxy::is_socket_alive() {
                    let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            SecurityError::MasterKeyNotFound
                        } else {
                            SecurityError::Io(e)
                        }
                    })?;

                    let key_bytes = proxy::call_proxy(crate::tpm::protocol::TpmRequest::Unseal {
                        data: sealed,
                    })
                    .map_err(SecurityError::Tpm)?
                    .ok_or_else(|| {
                        SecurityError::Tpm("No unsealed data returned from helper".into())
                    })?;
                    return MasterKey::from_bytes(&key_bytes);
                }

                Err(SecurityError::Tpm(
                    "TPM context unavailable and no helper running".into(),
                ))
            }
        }
    }

    fn delete(&self) -> Result<()> {
        let path = self.get_sealed_path();

        #[cfg(target_os = "linux")]
        if std::env::var("POSTAIL_TPM_HELPER").is_err() && proxy::is_socket_alive() {
            let _ = proxy::call_proxy(crate::tpm::protocol::TpmRequest::DeleteFile {
                path: path.clone(),
            });
            return Ok(());
        }

        if path.exists() {
            fs::remove_file(&path)?;
        }

        Ok(())
    }

    fn exists(&self) -> bool {
        self.get_sealed_path().exists()
    }

    fn is_available(&self) -> bool {
        context::tpm_dev_exists()
    }

    fn name(&self) -> &'static str {
        "TPM2 (Linux)"
    }
}
