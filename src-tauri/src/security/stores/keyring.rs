use keyring::Entry;

use crate::security::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::stores::SecretStore;

const SERVICE_NAME: &str = "postail";
const KEY_NAME: &str = "master_key";

pub struct KeyringStore {
    entry: Entry,
}

impl KeyringStore {
    pub fn new() -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME)
            .map_err(|e| SecurityError::Keyring(e.to_string()))?;
        Ok(Self { entry })
    }

    pub fn with_user(user: &str) -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, user)
            .map_err(|e| SecurityError::Keyring(e.to_string()))?;
        Ok(Self { entry })
    }
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new().expect("failed to create keyring entry")
    }
}

impl SecretStore for KeyringStore {
    fn store(&self, key: &MasterKey) -> Result<()> {
        let hex_key = hex_encode(key.as_bytes());
        self.entry
            .set_password(&hex_key)
            .map_err(|e| SecurityError::Keyring(e.to_string()))?;
        Ok(())
    }

    fn retrieve(&self) -> Result<MasterKey> {
        let hex_key = self
            .entry
            .get_password()
            .map_err(|e| match e {
                keyring::Error::NoEntry => SecurityError::MasterKeyNotFound,
                _ => SecurityError::Keyring(e.to_string()),
            })?;

        let bytes = hex_decode(&hex_key)?;
        MasterKey::from_bytes(&bytes)
    }

    fn delete(&self) -> Result<()> {
        self.entry
            .delete_credential()
            .map_err(|e| match e {
                keyring::Error::NoEntry => SecurityError::MasterKeyNotFound,
                _ => SecurityError::Keyring(e.to_string()),
            })?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        Entry::new(SERVICE_NAME, "availability_check").is_ok()
    }

    fn name(&self) -> &'static str {
        "OS Keyring"
    }
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub(crate) fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(SecurityError::Decryption("invalid hex length".into()));
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| SecurityError::Decryption("invalid hex character".into()))
        })
        .collect()
}