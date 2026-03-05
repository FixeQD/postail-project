#[cfg(feature = "tpm")]
use tss_esapi::{tcti_ldr::TctiNameConf, Context};

#[cfg(feature = "tpm")]
use std::str::FromStr;

use std::fs;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::stores::SecretStore;

use super::common;

// ── Helpers ────────────────────────────────────────────────────────

fn tpm_dev_exists() -> bool {
    std::path::Path::new("/dev/tpmrm0").exists() || std::path::Path::new("/dev/tpm0").exists()
}

// ── LinuxTpmStore ──────────────────────────────────────────────────

pub struct LinuxTpmStore {
    storage_path: PathBuf,
    #[cfg(feature = "tpm")]
    tcti: TctiNameConf,
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
mod proxy {
    pub use crate::security::tpm_protocol::{
        receive_message, send_message, TpmRequest, TpmResponse,
    };
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;

    pub fn get_socket_path() -> PathBuf {
        let uid = unsafe { nix::libc::getuid() };
        PathBuf::from(format!("/run/user/{}/postail-tpm.sock", uid))
    }

    pub fn is_socket_alive() -> bool {
        let path = get_socket_path();
        if !path.exists() {
            return false;
        }
        std::os::unix::net::UnixStream::connect(&path).is_ok()
    }

    pub fn call_proxy(req: TpmRequest) -> Result<Option<Vec<u8>>, String> {
        let path = get_socket_path();
        let mut stream = UnixStream::connect(&path)
            .map_err(|e| format!("Failed to connect to TPM helper: {}", e))?;

        send_message(&mut stream, &req)?;
        let res: TpmResponse = receive_message(&mut stream)?;

        match res {
            TpmResponse::Ok { key } => Ok(key),
            TpmResponse::Err(e) => Err(e),
        }
    }
}

impl LinuxTpmStore {
    pub fn new() -> Result<Self> {
        Self::with_storage_path(common::default_storage_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
        #[cfg(feature = "tpm")]
        {
            let tcti = if std::path::Path::new("/dev/tpmrm0").exists() {
                TctiNameConf::from_str("device:/dev/tpmrm0").map_err(common::tpm_err)?
            } else if std::path::Path::new("/dev/tpm0").exists() {
                TctiNameConf::from_str("device:/dev/tpm0").map_err(common::tpm_err)?
            } else {
                TctiNameConf::Tabrmd(Default::default())
            };

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

    // ── Context & availability ─────────────────────────────────────

    #[cfg(feature = "tpm")]
    fn create_context(&self) -> Result<Context> {
        Context::new(self.tcti.clone()).map_err(common::tpm_err)
    }

    #[cfg(feature = "tpm")]
    pub fn check_context_silent(&self) -> bool {
        self.check_direct_access() || {
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

    /// Verifies if the current process has direct access to the TPM device by attempting a real (but lightweight) operation.
    #[cfg(feature = "tpm")]
    pub fn check_direct_access(&self) -> bool {
        if !tpm_dev_exists() {
            return false;
        }

        match self.create_context() {
            Ok(mut ctx) => ctx.get_random(8).is_ok(),
            Err(_) => false,
        }
    }

    /// Returns true if TPM is present but direct access fails AND proxy is not running.
    #[cfg(feature = "tpm")]
    pub fn check_needs_elevation(&self) -> bool {
        if self.check_direct_access() {
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

    /// Verifies if the proxy helper is running and responsive.
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    pub fn verify_proxy(&self) -> bool {
        proxy::call_proxy(proxy::TpmRequest::Ping).is_ok()
    }
}

// ── SecretStore impl ───────────────────────────────────────────────

impl SecretStore for LinuxTpmStore {
    #[cfg(feature = "tpm")]
    fn store(&self, key: &MasterKey) -> Result<()> {
        match self.create_context() {
            Ok(mut ctx) => {
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
            Err(_) => {
                #[cfg(target_os = "linux")]
                if std::env::var("POSTAIL_TPM_HELPER").is_err() && proxy::is_socket_alive() {
                    return proxy::call_proxy(proxy::TpmRequest::Store {
                        key: key.as_bytes().to_vec(),
                    })
                    .map(|_| ())
                    .map_err(SecurityError::Tpm);
                }

                Err(SecurityError::Tpm(
                    "TPM context unavailable and no helper running".into(),
                ))
            }
        }
    }

    #[cfg(not(feature = "tpm"))]
    fn store(&self, _key: &MasterKey) -> Result<()> {
        Err(SecurityError::Tpm("TPM support not compiled in".into()))
    }

    #[cfg(feature = "tpm")]
    fn retrieve(&self) -> Result<MasterKey> {
        match self.create_context() {
            Ok(mut ctx) => {
                let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        SecurityError::MasterKeyNotFound
                    } else {
                        SecurityError::Io(e)
                    }
                })?;

                let primary = common::create_primary_key(&mut ctx)?;
                let unsealed = common::unseal_data(&mut ctx, primary.key_handle, &sealed)?;

                ctx.flush_context(primary.key_handle.into())
                    .map_err(common::tpm_err)?;

                MasterKey::from_bytes(&unsealed)
            }
            Err(_) => {
                #[cfg(target_os = "linux")]
                if std::env::var("POSTAIL_TPM_HELPER").is_err() && proxy::is_socket_alive() {
                    let key_bytes = proxy::call_proxy(proxy::TpmRequest::Retrieve)
                        .map_err(SecurityError::Tpm)?
                        .ok_or_else(|| SecurityError::Tpm("No key returned from proxy".into()))?;
                    return MasterKey::from_bytes(&key_bytes);
                }

                Err(SecurityError::Tpm(
                    "TPM context unavailable and no helper running".into(),
                ))
            }
        }
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

        #[cfg(target_os = "linux")]
        if self.create_context().is_err()
            && std::env::var("POSTAIL_TPM_HELPER").is_err()
            && proxy::is_socket_alive()
        {
            let _ = proxy::call_proxy(proxy::TpmRequest::Delete);
        }

        Ok(())
    }

    fn exists(&self) -> bool {
        self.get_sealed_path().exists()
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "tpm")]
        {
            tpm_dev_exists()
        }

        #[cfg(not(feature = "tpm"))]
        {
            false
        }
    }

    fn name(&self) -> &'static str {
        "TPM2 (Linux)"
    }
}
