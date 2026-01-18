use crate::error::Result;
use crate::security::master_key::MasterKey;

pub mod argon2;
pub mod keyring;
pub mod tpm;

pub trait SecretStore: Send + Sync {
    fn store(&self, key: &MasterKey) -> Result<()>;
    fn retrieve(&self) -> Result<MasterKey>;
    fn delete(&self) -> Result<()>;
    fn is_available(&self) -> bool;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorageTier {
    Passphrase = 0,
    Argon2 = 1,
    Tpm = 2,
    Keyring = 3,
}
