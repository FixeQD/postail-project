use std::fs;
use std::path::PathBuf;

use bip39::{Language, Mnemonic, MnemonicType};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::error::{Result, SecurityError};
use crate::security::crypto::{decrypt_with_key, encrypt_with_key};
use crate::security::master_key::{MasterKey, MASTER_KEY_LENGTH};
use crate::security::stores::SecretStore;

pub struct RecoveryKeyHolder {
    key: std::sync::Mutex<Option<MasterKey>>,
}

impl RecoveryKeyHolder {
    pub fn with_key(key: MasterKey) -> Self {
        Self {
            key: std::sync::Mutex::new(Some(key)),
        }
    }
}

impl SecretStore for RecoveryKeyHolder {
    fn store(&self, key: &MasterKey) -> Result<()> {
        *self.key.lock().unwrap() = Some(key.clone());
        Ok(())
    }

    fn retrieve(&self) -> Result<MasterKey> {
        self.key
            .lock()
            .unwrap()
            .clone()
            .ok_or(SecurityError::MasterKeyNotFound)
    }

    fn delete(&self) -> Result<()> {
        *self.key.lock().unwrap() = None;
        Ok(())
    }

    fn exists(&self) -> bool {
        self.key.lock().unwrap().is_some()
    }

    fn is_available(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Recovery Key Holder"
    }
}

const HKDF_INFO: &[u8] = b"postail-recovery-key-v1";

/// Generates a new 12-word BIP39 phrase.
pub fn generate_phrase() -> String {
    let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    mnemonic.phrase().to_string()
}

/// Derives a 32-byte MasterKey from a BIP39 recovery phrase.
pub fn derive_recovery_key(phrase: &str) -> Result<MasterKey> {
    let mnemonic = Mnemonic::from_phrase(phrase, Language::English)
        .map_err(|e| SecurityError::KeyDerivation(format!("invalid recovery phrase: {}", e)))?;

    let seed = bip39::Seed::new(&mnemonic, "");
    let seed_bytes = seed.as_bytes();

    let hk = Hkdf::<Sha256>::new(None, seed_bytes);
    let mut output = [0u8; MASTER_KEY_LENGTH];
    hk.expand(HKDF_INFO, &mut output)
        .map_err(|_| SecurityError::KeyDerivation("HKDF expand failed".to_string()))?;

    let key = MasterKey::from_bytes(&output)?;
    output.zeroize();
    Ok(key)
}

pub struct RecoveryStore {
    storage_path: PathBuf,
}

impl RecoveryStore {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    fn sealed_path(&self) -> PathBuf {
        self.storage_path.join("recovery.sealed")
    }

    /// Encrypts and saves the master key wit h the recovery phrase.
    pub fn create(&self, master_key: &MasterKey, phrase: &str) -> Result<()> {
        let recovery_key = derive_recovery_key(phrase)?;
        let encrypted = encrypt_with_key(&recovery_key, master_key.as_bytes())?;

        if let Some(parent) = self.sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(self.sealed_path(), &encrypted)?;
        Ok(())
    }

    /// Decrypts and returns the master key with the recovery phrase.
    pub fn unlock(&self, phrase: &str) -> Result<MasterKey> {
        let data = fs::read(self.sealed_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecurityError::MasterKeyNotFound
            } else {
                SecurityError::Io(e)
            }
        })?;

        if data.is_empty() {
            return Err(SecurityError::MasterKeyNotFound);
        }

        let recovery_key = derive_recovery_key(phrase)?;

        let decrypted = decrypt_with_key(&recovery_key, &data)
            .map_err(|_| SecurityError::InvalidPassphrase)?;

        MasterKey::from_bytes(&decrypted)
    }

    pub fn exists(&self) -> bool {
        self.sealed_path().exists()
    }
}
