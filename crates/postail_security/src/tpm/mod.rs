#[cfg(all(target_os = "linux", feature = "tpm"))]
pub mod helper;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub mod init;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub mod protocol;

pub mod store;

#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use helper::tpm_helper_init;
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use init::{TpmAvailability, TpmInitializer};
