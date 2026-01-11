use std::fs;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::stores::SecretStore;

const SEALED_FILE_NAME: &str = "master_key.tpm";

pub struct WindowsTpmStore {
    storage_path: PathBuf,
}

impl WindowsTpmStore {
    pub fn new() -> Result<Self> {
        Ok(Self {
            storage_path: default_storage_path(),
        })
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join(SEALED_FILE_NAME)
    }

    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn seal_with_tpm(&self, data: &[u8]) -> Result<Vec<u8>> {
        use windows::core::PCWSTR;
        use windows::Win32::Security::Cryptography::{
            NCryptCreateProtectionDescriptor, NCryptProtectSecret, NCryptUnprotectSecret,
            NCRYPT_DESCRIPTOR_HANDLE, NCRYPT_MACHINE_KEY_FLAG, NCRYPT_PROTECT_TO_LOCAL_SYSTEM,
            NCRYPT_SILENT_FLAG,
        };

        let descriptor_string: Vec<u16> = "LOCAL=user\0".encode_utf16().collect();

        unsafe {
            let mut descriptor_handle: NCRYPT_DESCRIPTOR_HANDLE = std::mem::zeroed();

            let result = NCryptCreateProtectionDescriptor(
                PCWSTR(descriptor_string.as_ptr()),
                0,
                &mut descriptor_handle,
            );

            if result.is_err() {
                return Err(SecurityError::Tpm(format!(
                    "NCryptCreateProtectionDescriptor failed: {:?}",
                    result
                )));
            }

            let mut protected_blob: *mut u8 = std::ptr::null_mut();
            let mut protected_size: u32 = 0;

            let result = NCryptProtectSecret(
                descriptor_handle,
                NCRYPT_SILENT_FLAG,
                data.as_ptr(),
                data.len() as u32,
                std::ptr::null(),
                std::ptr::null(),
                &mut protected_blob,
                &mut protected_size,
            );

            if result.is_err() {
                return Err(SecurityError::Tpm(format!(
                    "NCryptProtectSecret failed: {:?}",
                    result
                )));
            }

            let sealed =
                std::slice::from_raw_parts(protected_blob, protected_size as usize).to_vec();

            windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(
                protected_blob as *mut _,
            ));

            Ok(sealed)
        }
    }

    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn unseal_with_tpm(&self, sealed: &[u8]) -> Result<Vec<u8>> {
        use windows::Win32::Security::Cryptography::{NCryptUnprotectSecret, NCRYPT_SILENT_FLAG};

        unsafe {
            let mut unprotected_blob: *mut u8 = std::ptr::null_mut();
            let mut unprotected_size: u32 = 0;

            let result = NCryptUnprotectSecret(
                std::ptr::null_mut(),
                NCRYPT_SILENT_FLAG,
                sealed.as_ptr(),
                sealed.len() as u32,
                std::ptr::null(),
                std::ptr::null(),
                &mut unprotected_blob,
                &mut unprotected_size,
            );

            if result.is_err() {
                return Err(SecurityError::Tpm(format!(
                    "NCryptUnprotectSecret failed: {:?}",
                    result
                )));
            }

            let unsealed =
                std::slice::from_raw_parts(unprotected_blob, unprotected_size as usize).to_vec();

            windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(
                unprotected_blob as *mut _,
            ));

            Ok(unsealed)
        }
    }

    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn check_tpm_available(&self) -> bool {
        use windows::Win32::Security::Tpm::Tbsi_GetDeviceInfo;
        use windows::Win32::Security::Tpm::TBS_DEVICE_INFO;

        unsafe {
            let mut device_info: TBS_DEVICE_INFO = std::mem::zeroed();
            let result = Tbsi_GetDeviceInfo(
                std::mem::size_of::<TBS_DEVICE_INFO>() as u32,
                &mut device_info as *mut _ as *mut _,
            );

            result.is_ok() && device_info.tpmVersion != 0
        }
    }
}

impl SecretStore for WindowsTpmStore {
    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn store(&self, key: &MasterKey) -> Result<()> {
        let sealed = self.seal_with_tpm(key.as_bytes())?;

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(self.get_sealed_path(), sealed)?;
        Ok(())
    }

    #[cfg(not(all(target_os = "windows", feature = "tpm")))]
    fn store(&self, _key: &MasterKey) -> Result<()> {
        Err(SecurityError::Tpm("TPM support not compiled in".into()))
    }

    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn retrieve(&self) -> Result<MasterKey> {
        let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecurityError::MasterKeyNotFound
            } else {
                SecurityError::Io(e)
            }
        })?;

        let unsealed = self.unseal_with_tpm(&sealed)?;
        MasterKey::from_bytes(&unsealed)
    }

    #[cfg(not(all(target_os = "windows", feature = "tpm")))]
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

    fn is_available(&self) -> bool {
        #[cfg(all(target_os = "windows", feature = "tpm"))]
        {
            self.check_tpm_available()
        }

        #[cfg(not(all(target_os = "windows", feature = "tpm")))]
        {
            false
        }
    }

    fn name(&self) -> &'static str {
        "TPM (Windows)"
    }
}

fn default_storage_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("security")
}
