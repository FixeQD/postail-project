use std::fs;
use std::path::PathBuf;
use tracing;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, Params, Version,
};
use zeroize::Zeroize;

use crate::error::{Result, SecurityError};
use crate::security::crypto::{decrypt_with_key, encrypt_with_key};
use crate::security::master_key::{MasterKey, MASTER_KEY_LENGTH};
use crate::security::stores::SecretStore;

const ARGON2_TIME_COST: u32 = 3;
const ARGON2_MEMORY_COST: u32 = 65536;
const ARGON2_PARALLELISM: u32 = 4;

pub struct Argon2Store {
    storage_path: PathBuf,
    passphrase: String,
}

impl Argon2Store {
    pub fn new(storage_path: PathBuf, passphrase: String) -> Self {
        Self {
            storage_path,
            passphrase: passphrase.trim().to_string(),
        }
    }

    fn derive_key_from_passphrase(&self, salt: &[u8]) -> Result<MasterKey> {
        let passphrase_bytes = self.passphrase.trim().as_bytes();

        let params = Params::new(
            ARGON2_MEMORY_COST,
            ARGON2_TIME_COST,
            ARGON2_PARALLELISM,
            Some(MASTER_KEY_LENGTH),
        )
        .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        let mut output = [0u8; MASTER_KEY_LENGTH];
        argon2
            .hash_password_into(passphrase_bytes, salt, &mut output)
            .map_err(|e| {
                tracing::error!(target: "postail", "[Security] Argon2 key derivation failed: {}", e);
                SecurityError::KeyDerivation(e.to_string())
            })?;

        MasterKey::from_bytes(&output)
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join("master_key.sealed")
    }
}

impl Drop for Argon2Store {
    fn drop(&mut self) {
        self.passphrase.zeroize();
    }
}

impl SecretStore for Argon2Store {
    fn store(&self, key: &MasterKey) -> Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let salt_bytes = salt.as_str().as_bytes();

        let derived_key = self.derive_key_from_passphrase(salt_bytes)?;
        let encrypted = encrypt_with_key(&derived_key, key.as_bytes())?;

        // format: salt_len(1) + salt + encrypted
        let mut data = Vec::with_capacity(1 + salt_bytes.len() + encrypted.len());
        data.push(salt_bytes.len() as u8);
        data.extend_from_slice(salt_bytes);
        data.extend_from_slice(&encrypted);

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(self.get_sealed_path(), &data)?;
        Ok(())
    }

    fn retrieve(&self) -> Result<MasterKey> {
        let data = fs::read(self.get_sealed_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecurityError::MasterKeyNotFound
            } else {
                SecurityError::Io(e)
            }
        })?;

        if data.is_empty() {
            return Err(SecurityError::MasterKeyNotFound);
        }

        let salt_len = data[0] as usize;
        if data.len() < 1 + salt_len {
            return Err(SecurityError::Decryption("corrupted sealed file".into()));
        }

        let salt = &data[1..1 + salt_len];
        let encrypted = &data[1 + salt_len..];

        let derived_key = self.derive_key_from_passphrase(salt)?;
        let decrypted = decrypt_with_key(&derived_key, encrypted).map_err(|e| {
            tracing::error!(target: "postail", "[Security] Failed to decrypt master key: {}. Possible incorrect passphrase or corrupted file.", e);
            SecurityError::InvalidPassphrase
        })?;

        MasterKey::from_bytes(&decrypted)
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
        !self.passphrase.is_empty()
    }

    fn name(&self) -> &'static str {
        "Passphrase (Argon2id)"
    }
}
