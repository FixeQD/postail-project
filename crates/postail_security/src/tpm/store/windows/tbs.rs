//! TBS (TPM Base Services) context - sends raw TPM2 commands to the chip
//! Ported from tpm-rs

use crate::error::{Result, SecurityError};

use std::ffi::c_void;
use windows::Win32::System::TpmBaseServices::{
    TBS_COMMAND_LOCALITY, TBS_COMMAND_PRIORITY, TBS_CONTEXT_PARAMS2, TBS_CONTEXT_PARAMS2_0,
    TBS_CONTEXT_PARAMS2_0_0, Tbsi_Context_Create, Tbsip_Context_Close, Tbsip_Submit_Command,
};

const TBS_SUCCESS: u32 = 0;

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
