use tss_esapi::{Context, tcti_ldr::TctiNameConf};

use std::str::FromStr;

use crate::error::Result;
use crate::tpm::store::paths::tpm_err;

pub fn tpm_dev_exists() -> bool {
    std::path::Path::new("/dev/tpmrm0").exists() || std::path::Path::new("/dev/tpm0").exists()
}

pub struct LinuxTpmContext {
    tcti: TctiNameConf,
}

impl LinuxTpmContext {
    pub fn new() -> Result<Self> {
        let tcti = if std::path::Path::new("/dev/tpmrm0").exists() {
            TctiNameConf::from_str("device:/dev/tpmrm0").map_err(tpm_err)?
        } else if std::path::Path::new("/dev/tpm0").exists() {
            TctiNameConf::from_str("device:/dev/tpm0").map_err(tpm_err)?
        } else {
            TctiNameConf::Tabrmd(Default::default())
        };

        Ok(Self { tcti })
    }

    pub fn create_context(&self) -> Result<Context> {
        Context::new(self.tcti.clone()).map_err(tpm_err)
    }

    pub fn check_direct_access(&self) -> bool {
        if !tpm_dev_exists() {
            return false;
        }

        match self.create_context() {
            Ok(mut ctx) => ctx.get_random(8).is_ok(),
            Err(_) => false,
        }
    }
}
