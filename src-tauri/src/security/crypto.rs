use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use crate::security::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;

pub const NONCE_LENGTH: usize = 12;

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(key: &MasterKey) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
            .expect("key length is always 32 bytes"); // can't fail with valid MasterKey
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SecurityError::Encryption(e.to_string()))?;

        let mut result = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < NONCE_LENGTH {
            return Err(SecurityError::InvalidNonceLength);
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| SecurityError::Decryption(e.to_string()))?;

        Ok(plaintext)
    }
}

pub fn encrypt_with_key(key: &MasterKey, plaintext: &[u8]) -> Result<Vec<u8>> {
    Crypto::new(key).encrypt(plaintext)
}

pub fn decrypt_with_key(key: &MasterKey, ciphertext: &[u8]) -> Result<Vec<u8>> {
    Crypto::new(key).decrypt(ciphertext)
}