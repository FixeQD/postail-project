use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{Result, SecurityError};
use crate::security::crypto::{decrypt_with_key, encrypt_with_key, Crypto};
use crate::security::master_key::MasterKey;
use crate::security::storage::keyring::KeyringStore;
use crate::security::tpm::store::get_tpm_store;
use crate::security::storage::{SecretStore, StorageTier};

struct NoSecureStorageStore;

impl SecretStore for NoSecureStorageStore {
    fn store(&self, _key: &MasterKey) -> Result<()> {
        Err(SecurityError::NoSecureStorageAvailable)
    }

    fn retrieve(&self) -> Result<MasterKey> {
        Err(SecurityError::NoSecureStorageAvailable)
    }

    fn delete(&self) -> Result<()> {
        Ok(())
    }

    fn exists(&self) -> bool {
        false
    }

    fn is_available(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "No Secure Storage"
    }
}

pub struct SecurityManager {
    store: Arc<dyn SecretStore>,
    master_key: Option<MasterKey>,
    storage_tier: StorageTier,
}

impl SecurityManager {
    pub fn new() -> Result<Self> {
        let (store, tier) = Self::select_best_store()?;
        Ok(Self {
            store,
            master_key: None,
            storage_tier: tier,
        })
    }

    pub fn with_store(store: Arc<dyn SecretStore>, tier: StorageTier) -> Self {
        Self {
            store,
            master_key: None,
            storage_tier: tier,
        }
    }

    fn select_best_store() -> Result<(Arc<dyn SecretStore>, StorageTier)> {
        if let Ok(keyring) = KeyringStore::new() {
            if keyring.is_available() {
                return Ok((Arc::new(keyring), StorageTier::Keyring));
            }
        }

        if let Some(tpm_store) = get_tpm_store() {
            return Ok((tpm_store.into(), StorageTier::Tpm));
        }

        Ok((Arc::new(NoSecureStorageStore), StorageTier::Passphrase))
    }

    pub fn storage_tier(&self) -> StorageTier {
        self.storage_tier
    }

    pub fn storage_name(&self) -> &'static str {
        self.store.name()
    }

    pub fn initialize(&mut self) -> Result<()> {
        if self.master_key.is_some() {
            return Err(SecurityError::MasterKeyAlreadyExists);
        }

        let key = MasterKey::generate();
        self.store.store(&key)?;
        self.master_key = Some(key);
        Ok(())
    }

    pub fn initialize_with_key(&mut self, key: MasterKey) -> Result<()> {
        if self.master_key.is_some() {
            return Err(SecurityError::MasterKeyAlreadyExists);
        }

        self.store.store(&key)?;
        self.master_key = Some(key);
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<()> {
        if self.master_key.is_some() {
            return Ok(());
        }

        let key = self.store.retrieve()?;
        self.master_key = Some(key);
        Ok(())
    }

    pub fn lock(&mut self) {
        self.master_key = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.master_key.is_some()
    }

    pub fn is_initialized(&self) -> bool {
        self.store.exists()
    }

    pub fn destroy(&mut self) -> Result<()> {
        self.store.delete()?;
        self.master_key = None;
        Ok(())
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_key()?;
        encrypt_with_key(key, plaintext)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_key()?;
        decrypt_with_key(key, ciphertext)
    }

    pub fn crypto(&self) -> Result<Crypto> {
        let key = self.get_key()?;
        Ok(Crypto::new(key))
    }

    pub fn export_master_key(&self) -> Result<MasterKey> {
        self.get_key().cloned()
    }

    fn get_key(&self) -> Result<&MasterKey> {
        self.master_key
            .as_ref()
            .ok_or(SecurityError::MasterKeyNotFound)
    }

    pub fn get_master_key_raw(&self) -> Vec<u8> {
        self.master_key
            .as_ref()
            .map(|mk| mk.as_bytes().to_vec())
            .unwrap_or_default()
    }

    pub fn encrypt_with_passphrase(&self, plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
        crate::security::crypto::helpers::encrypt_with_passphrase(plaintext, passphrase)
    }

    pub fn decrypt_with_passphrase(&self, ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
        crate::security::crypto::helpers::decrypt_with_passphrase(ciphertext, passphrase)
    }
}

pub struct PassphraseSecurityBuilder {
    storage_path: PathBuf,
    passphrase: String,
}

impl PassphraseSecurityBuilder {
    pub fn new(storage_path: PathBuf, passphrase: String) -> Self {
        Self {
            storage_path,
            passphrase: passphrase.trim().to_string(),
        }
    }

    pub fn build(self) -> SecurityManager {
        use crate::security::storage::argon2::Argon2Store;

        let store = Argon2Store::new(self.storage_path, self.passphrase);
        SecurityManager::with_store(Arc::new(store), StorageTier::Passphrase)
    }
}
