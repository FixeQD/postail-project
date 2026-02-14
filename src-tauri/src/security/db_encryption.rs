use std::fs;
use std::sync::Mutex;
use tracing;

static TEST_SALT_MUTEX: Mutex<()> = Mutex::new(());

const TEST_DETERMINISTIC_SALT: &[u8] = b"test-salt-for-hkdf-32bytes!";

use hkdf::Hkdf;
use keyring::Entry;
use sha2::Sha256;
use thiserror::Error;
use zeroize::Zeroize;

const DB_ENC_KEY_INFO: &[u8] = b"postail-db-encryption-v1";
const DB_ENC_SALT_SERVICE: &str = "postail";
const DB_ENC_SALT_KEY: &str = "db-encryption-salt";

#[derive(Debug, Error)]
pub enum DbEncryptionError {
    #[error("keyring error: {0}")]
    Keyring(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("HKDF error: {0}")]
    Hkdf(String),
}

pub struct DbEncryption {
    cipher_key: [u8; 32],
}

impl DbEncryption {
    pub fn derive_from_master_key(master_key: &[u8]) -> Result<Self, DbEncryptionError> {
        let salt = Self::get_or_create_salt()?;
        Self::derive_with_salt(master_key, &salt)
    }

    pub fn derive_with_test_salt(master_key: &[u8]) -> Result<Self, DbEncryptionError> {
        let _guard = TEST_SALT_MUTEX.lock().unwrap();
        Self::derive_with_salt(master_key, TEST_DETERMINISTIC_SALT)
    }

    fn derive_with_salt(master_key: &[u8], salt: &[u8]) -> Result<Self, DbEncryptionError> {
        let mut okm = [0u8; 32];
        let hkdf = Hkdf::<Sha256>::new(Some(salt), master_key);
        hkdf.expand(DB_ENC_KEY_INFO, &mut okm)
            .map_err(|e| DbEncryptionError::Hkdf(e.to_string()))?;

        Ok(Self { cipher_key: okm })
    }

    fn get_or_create_salt() -> Result<Vec<u8>, DbEncryptionError> {
        let salt_file = crate::utils::config::get_data_dir()
            .join("security")
            .join("db_salt");

        // 1. Try Keyring first
        let entry = Entry::new(DB_ENC_SALT_SERVICE, DB_ENC_SALT_KEY)
            .map_err(|e| DbEncryptionError::Keyring(e.to_string()))?;

        match entry.get_password() {
            Ok(salt_hex) => {
                if let Ok(salt) = hex::decode(&salt_hex) {
                    return Ok(salt);
                }
            }
            Err(_) => {}
        }

        // 2. Try File Fallback
        if salt_file.exists() {
            if let Ok(salt_hex) = fs::read_to_string(&salt_file) {
                if let Ok(salt) = hex::decode(salt_hex.trim()) {
                    // Try to restore to keyring if it was missing
                    let _ = entry.set_password(&hex::encode(&salt));

                    return Ok(salt);
                }
            }
        }

        // 3. Generate new salt
        tracing::info!(target: "postail", "[Security] No DB salt found in keyring or file. Generating new salt...");
        let salt: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let salt_hex = hex::encode(&salt);

        // Try to save to both
        if let Err(e) = entry.set_password(&salt_hex) {
            tracing::warn!(target: "postail", "[Security] Failed to save salt to keyring: {}", e);
        }

        if let Some(parent) = salt_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Err(e) = fs::write(&salt_file, &salt_hex) {
            tracing::error!(target: "postail", "[Security] Failed to save salt to file: {}", e);
        }

        Ok(salt)
    }

    pub fn hex_key(&self) -> String {
        hex::encode(self.cipher_key)
    }

    pub fn raw_key(&self) -> &[u8; 32] {
        &self.cipher_key
    }

    pub fn re_derive_from_new_master_key(
        &self,
        old_master_key: &[u8],
        new_master_key: &[u8],
    ) -> Result<Self, DbEncryptionError> {
        let salt = Self::get_or_create_salt()?;
        let mut okm = [0u8; 32];

        let hkdf_old = Hkdf::<Sha256>::new(Some(&salt), old_master_key);
        let mut old_derived = [0u8; 32];
        hkdf_old
            .expand(b"postail-db-migration-v1", &mut old_derived)
            .map_err(|e| DbEncryptionError::Hkdf(e.to_string()))?;

        let hkdf_new = Hkdf::<Sha256>::new(Some(&salt), new_master_key);
        hkdf_new
            .expand(DB_ENC_KEY_INFO, &mut okm)
            .map_err(|e| DbEncryptionError::Hkdf(e.to_string()))?;

        Ok(Self { cipher_key: okm })
    }
}

impl Drop for DbEncryption {
    fn drop(&mut self) {
        self.cipher_key.zeroize();
    }
}

impl DbEncryption {
    pub fn from_master_key(master_key: &[u8]) -> Result<Self, DbEncryptionError> {
        if master_key.is_empty() {
            return Err(DbEncryptionError::Keyring(
                "Master key not available during encryption initialization".to_string(),
            ));
        }
        Self::derive_from_master_key(master_key)
    }

    pub fn get_hex_key(master_key: &[u8]) -> String {
        Self::from_master_key(master_key)
            .map(|e| e.hex_key())
            .unwrap_or_else(|e| {
                tracing::error!(target: "postail", "[DB] Failed to get encryption key: {}", e);
                String::new()
            })
    }
}
