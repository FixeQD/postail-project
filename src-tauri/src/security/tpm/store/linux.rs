use tss_esapi::{Context, tcti_ldr::TctiNameConf};

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::storage::SecretStore;

use super::common;

// ── Helpers ────────────────────────────────────────────────────────

fn tpm_dev_exists() -> bool {
    std::path::Path::new("/dev/tpmrm0").exists() || std::path::Path::new("/dev/tpm0").exists()
}

// ── LinuxTpmStore ──────────────────────────────────────────────────

pub struct LinuxTpmStore {
    storage_path: PathBuf,
    tcti: TctiNameConf,
}

mod proxy {
    pub use crate::security::tpm::protocol::{
        TpmRequest, TpmResponse, receive_message, send_message,
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
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join(common::SEALED_FILE_NAME)
    }

    // ── Context & availability ─────────────────────────────────────

    pub fn create_context(&self) -> Result<Context> {
        Context::new(self.tcti.clone()).map_err(common::tpm_err)
    }

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
    pub fn verify_proxy(&self) -> bool {
        proxy::call_proxy(proxy::TpmRequest::Ping).is_ok()
    }
}

// ── SecretStore impl ───────────────────────────────────────────────
impl SecretStore for LinuxTpmStore {
    fn store(&self, key: &MasterKey) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if std::env::var("POSTAIL_TPM_HELPER").is_ok() {
                let mut ctx = self.create_context()?;
                let primary = common::create_primary_key(&mut ctx)?;
                let sealed = common::seal_data(&mut ctx, primary.key_handle, key.as_bytes())?;

                fs::write(self.get_sealed_path(), sealed)?;

                ctx.flush_context(primary.key_handle.into())
                    .map_err(common::tpm_err)?;
                return Ok(());
            }
        }

        match self.create_context() {
            Ok(mut ctx) => {
                let primary = common::create_primary_key(&mut ctx)?;
                let sealed = common::seal_data(&mut ctx, primary.key_handle, key.as_bytes())?;

                fs::write(self.get_sealed_path(), sealed)?;

                ctx.flush_context(primary.key_handle.into())
                    .map_err(common::tpm_err)?;
                Ok(())
            }
            Err(_) => {
                #[cfg(target_os = "linux")]
                if proxy::is_socket_alive() {
                    let sealed = proxy::call_proxy(proxy::TpmRequest::Seal {
                        key: key.as_bytes().to_vec(),
                    })
                    .map_err(SecurityError::Tpm)?
                    .ok_or_else(|| {
                        SecurityError::Tpm("No sealed data returned from helper".into())
                    })?;

                    fs::write(self.get_sealed_path(), sealed)?;

                    return Ok(());
                }

                Err(SecurityError::Tpm(
                    "TPM context unavailable and no helper running".into(),
                ))
            }
        }
    }

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
                    let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            SecurityError::MasterKeyNotFound
                        } else {
                            SecurityError::Io(e)
                        }
                    })?;

                    let key_bytes = proxy::call_proxy(proxy::TpmRequest::Unseal { data: sealed })
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
            let _ = proxy::call_proxy(proxy::TpmRequest::DeleteFile { path: path.clone() });
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
        { tpm_dev_exists() }
    }

    fn name(&self) -> &'static str {
        "TPM2 (Linux)"
    }
}
