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
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Security::Cryptography::NCryptCreateProtectionDescriptor;
        use windows::Win32::Security::Cryptography::NCryptFreeBuffer;
        use windows::Win32::Security::Cryptography::NCryptProtectSecret;

        let descriptor_string: Vec<u16> = "LOCAL=user\0".encode_utf16().collect();

        unsafe {
            let descriptor_handle = NCryptCreateProtectionDescriptor(
                PCWSTR(descriptor_string.as_ptr()),
                0,
            )
            .map_err(|e| {
                SecurityError::Tpm(format!("NCryptCreateProtectionDescriptor failed: {:?}", e))
            })?;

            let mut protected_blob: *mut u8 = std::ptr::null_mut();
            let mut protected_size: u32 = 0;

            NCryptProtectSecret(
                descriptor_handle,
                0,
                data,
                None,
                Some(HWND(std::ptr::null_mut())),
                &mut protected_blob,
                &mut protected_size,
            )
            .map_err(|e| SecurityError::Tpm(format!("NCryptProtectSecret failed: {:?}", e)))?;

            let sealed =
                std::slice::from_raw_parts(protected_blob, protected_size as usize).to_vec();

            let _ = NCryptFreeBuffer(protected_blob as *mut _);

            Ok(sealed)
        }
    }

    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn unseal_with_tpm(&self, sealed: &[u8]) -> Result<Vec<u8>> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Security::Cryptography::NCryptFreeBuffer;
        use windows::Win32::Security::Cryptography::NCryptUnprotectSecret;
        use windows::Win32::Security::Cryptography::NCRYPT_SILENT_FLAG;

        unsafe {
            let mut unprotected_blob: *mut u8 = std::ptr::null_mut();
            let mut unprotected_size: u32 = 0;

            NCryptUnprotectSecret(
                None,
                NCRYPT_SILENT_FLAG,
                sealed,
                None,
                Some(HWND(std::ptr::null_mut())),
                &mut unprotected_blob,
                &mut unprotected_size,
            )
            .map_err(|e| SecurityError::Tpm(format!("NCryptUnprotectSecret failed: {:?}", e)))?;

            let unsealed =
                std::slice::from_raw_parts(unprotected_blob, unprotected_size as usize).to_vec();

            let _ = NCryptFreeBuffer(unprotected_blob as *mut _);

            Ok(unsealed)
        }
    }

    #[cfg(all(target_os = "windows", feature = "tpm"))]
    fn check_tpm_available(&self) -> bool {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
        };

        unsafe {
            let tpm_path: Vec<u16> = r"\\.\TPM"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let handle = CreateFileW(
                PCWSTR(tpm_path.as_ptr()),
                0, // No access rights needed for existence check
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            );

            match handle {
                Ok(h) if h != INVALID_HANDLE_VALUE => {
                    let _ = CloseHandle(h);
                    true
                }
                _ => false,
            }
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

    fn exists(&self) -> bool {
        self.get_sealed_path().exists()
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
