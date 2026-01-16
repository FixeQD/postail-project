use std::sync::Mutex;

static TEST_SALT_MUTEX: Mutex<()> = Mutex::new(());

const TEST_DETERMINISTIC_SALT: &[u8] = b"test-salt-for-hkdf-32bytes!";

use hkdf::Hkdf;
use keyring::Entry;
use sha2::Sha256;
use std::sync::Arc;
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
        let entry = Entry::new(DB_ENC_SALT_SERVICE, DB_ENC_SALT_KEY)
            .map_err(|e| DbEncryptionError::Keyring(e.to_string()))?;

        if let Ok(salt_hex) = entry.get_password() {
            hex::decode(salt_hex).map_err(DbEncryptionError::Hex)
        } else {
            let salt: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
            let salt_hex = hex::encode(&salt);

            entry
                .set_password(&salt_hex)
                .map_err(|e| DbEncryptionError::Keyring(e.to_string()))?;

            Ok(salt)
        }
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

lazy_static::lazy_static! {
    pub static ref DB_ENCRYPTION: Arc<Result<DbEncryption, DbEncryptionError>> = {
        Arc::new(DbEncryption::initialize())
    };
}

impl DbEncryption {
    fn initialize() -> Result<Self, DbEncryptionError> {
        use crate::SECURITY;

        let master_key = {
            let security = SECURITY.lock().unwrap();
            security.get_master_key_raw()
        };

        Self::derive_from_master_key(&master_key)
    }

    pub fn global() -> &'static Arc<Result<DbEncryption, DbEncryptionError>> {
        &DB_ENCRYPTION
    }

    pub fn get_hex_key() -> String {
        DB_ENCRYPTION
            .as_ref()
            .as_ref()
            .map(|e| e.hex_key())
            .unwrap_or_else(|e| {
                eprintln!("[DB] Failed to get encryption key: {}", e);
                String::new()
            })
    }
}
