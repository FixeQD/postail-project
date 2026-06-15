//! TBS (TPM Base Services) context - sends raw TPM2 commands to the chip
//! Ported from tpm-rs

use crate::error::{Result, SecurityError};

use std::ffi::c_void;
use windows::Win32::System::TpmBaseServices::{
    TBS_COMMAND_LOCALITY, TBS_COMMAND_PRIORITY, TBS_CONTEXT_PARAMS2, TBS_CONTEXT_PARAMS2_0,
    TBS_CONTEXT_PARAMS2_0_0, Tbsi_Context_Create, Tbsi_GetDeviceInfo, Tbsip_Context_Close,
    Tbsip_Submit_Command,
};

const TBS_SUCCESS: u32 = 0;

/// TPM version returned by `Tbsi_GetDeviceInfo`.
#[allow(dead_code)]
pub const TPM_VERSION_12: u32 = 1;
pub const TPM_VERSION_20: u32 = 2;

/// TPM chip info returned by Windows TBS
/// Maps to the WinAPI `TPM_DEVICE_INFO` struct
#[derive(Debug, Clone)]
pub struct TpmDeviceInfo {
    /// Chip version: 1 = TPM 1.2, 2 = TPM 2.0
    pub tpm_version: u32,
    /// Interface type (TIS, CRB, etc.)
    pub tpm_interface_type: u32,
    /// Firmware implementation revision
    pub tpm_imp_revision: u32,
}

/// Binary layout of the WinAPI `TPM_DEVICE_INFO` struct
#[repr(C)]
struct RawDeviceInfo {
    struct_version: u32,
    tpm_version: u32,
    tpm_interface_type: u32,
    tpm_imp_revision: u32,
}

pub struct TbsContext {
    handle: *mut c_void,
}

impl TbsContext {
    /// Create a new TBS context for TPM 2.0
    pub fn new() -> Result<Self> {
        // bitfield: bit 2 = includeTpm20
        let params = TBS_CONTEXT_PARAMS2 {
            version: 2,
            Anonymous: TBS_CONTEXT_PARAMS2_0 {
                Anonymous: TBS_CONTEXT_PARAMS2_0_0 { _bitfield: 0b100 },
            },
        };

        let mut handle: *mut c_void = std::ptr::null_mut();

        let result = unsafe {
            Tbsi_Context_Create(
                &params as *const TBS_CONTEXT_PARAMS2 as *const _,
                &mut handle,
            )
        };

        if result != TBS_SUCCESS {
            return Err(SecurityError::Tpm(format!(
                "Tbsi_Context_Create failed: 0x{result:08X}"
            )));
        }

        Ok(Self { handle })
    }

    /// Queries Windows TBS for chip info
    pub fn get_device_info() -> Option<TpmDeviceInfo> {
        let mut info = RawDeviceInfo {
            struct_version: 1,
            tpm_version: 0,
            tpm_interface_type: 0,
            tpm_imp_revision: 0,
        };

        let result = unsafe {
            Tbsi_GetDeviceInfo(
                std::mem::size_of::<RawDeviceInfo>() as u32,
                &mut info as *mut RawDeviceInfo as *mut _,
            )
        };

        if result != TBS_SUCCESS {
            tracing::debug!(
                "Tbsi_GetDeviceInfo returned 0x{:08X} — brak TPM lub TBS niedostępny",
                result
            );
            return None;
        }

        tracing::debug!(
            "TPM device info: version={}, interface_type={}, imp_revision={}",
            info.tpm_version,
            info.tpm_interface_type,
            info.tpm_imp_revision,
        );

        Some(TpmDeviceInfo {
            tpm_version: info.tpm_version,
            tpm_interface_type: info.tpm_interface_type,
            tpm_imp_revision: info.tpm_imp_revision,
        })
    }

    /// Sends `TPM2_GetRandom(1)` as a lightweight liveness check
    pub fn probe(&self) -> bool {
        use super::proto;

        let cmd = proto::cmd_get_random(1);
        match self.submit(&cmd) {
            Ok(resp) => match proto::parse_get_random(&resp) {
                Ok(random_bytes) => {
                    if random_bytes.len() == 1 {
                        tracing::debug!("TPM2_GetRandom probe OK");
                        true
                    } else {
                        tracing::warn!(
                            "TPM2_GetRandom zwrócił {} bajtów zamiast 1",
                            random_bytes.len()
                        );
                        false
                    }
                }
                Err(e) => {
                    tracing::warn!("TPM2_GetRandom parse error: {e}");
                    false
                }
            },
            Err(e) => {
                tracing::warn!("TPM2_GetRandom submit error: {e}");
                false
            }
        }
    }

    /// Submit a raw TPM2 command and return the response bytes
    pub fn submit(&self, command: &[u8]) -> Result<Vec<u8>> {
        let mut output_size: u32 = 4096;
        let mut output = vec![0u8; output_size as usize];

        let result = unsafe {
            Tbsip_Submit_Command(
                self.handle,
                TBS_COMMAND_LOCALITY(0),
                TBS_COMMAND_PRIORITY(200),
                command,
                output.as_mut_ptr(),
                &mut output_size,
            )
        };

        if result != TBS_SUCCESS {
            return Err(SecurityError::Tpm(format!(
                "TPM2 command failed: 0x{result:08X}"
            )));
        }

        output.truncate(output_size as usize);
        Ok(output)
    }
}

impl Drop for TbsContext {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                let _ = Tbsip_Context_Close(self.handle);
            }
        }
    }
}

// Safety: TBS handles are thread-safe
unsafe impl Send for TbsContext {}
