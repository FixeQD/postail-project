use std::ops::Deref;
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ZeroizingBytes(pub Vec<u8>);

impl Deref for ZeroizingBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ZeroizingBytes {
    fn from(data: Vec<u8>) -> Self {
        ZeroizingBytes(data)
    }
}

impl From<&[u8]> for ZeroizingBytes {
    fn from(data: &[u8]) -> Self {
        ZeroizingBytes(data.to_vec())
    }
}

pub fn secure_zeroize(data: &mut [u8]) {
    data.zeroize();
}

pub fn secure_zeroize_vec(data: &mut Vec<u8>) {
    data.zeroize();
}

/// Encrypt data with a passphrase using Argon2 key derivation + AES-GCM.
/// Format: salt_len(1 byte) + salt + AES-ciphertext
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &str,
) -> crate::error::Result<Vec<u8>> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::Argon2;

    let salt = SaltString::generate(&mut OsRng);
    let salt_bytes = salt.as_str().as_bytes();

    let mut derived_key = [0u8; 32];
    let argon2 = Argon2::default();
    let passphrase_trimmed = passphrase.trim();

    argon2
        .hash_password_into(passphrase_trimmed.as_bytes(), salt_bytes, &mut derived_key)
        .map_err(|e| crate::error::SecurityError::KeyDerivation(e.to_string()))?;

    let key = crate::master_key::MasterKey::from_bytes(&derived_key)?;
    derived_key.zeroize();

    let encrypted = super::encrypt_with_key(&key, plaintext)?;

    let mut data = Vec::with_capacity(1 + salt_bytes.len() + encrypted.len());
    data.push(salt_bytes.len() as u8);
    data.extend_from_slice(salt_bytes);
    data.extend_from_slice(&encrypted);

    Ok(data)
}

/// Decrypt data encrypted with `encrypt_with_passphrase`.
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    passphrase: &str,
) -> crate::error::Result<Vec<u8>> {
    use argon2::Argon2;

    if ciphertext.is_empty() {
        return Err(crate::error::SecurityError::Decryption(
            "empty ciphertext".into(),
        ));
    }

    let salt_len = ciphertext[0] as usize;
    if ciphertext.len() < 1 + salt_len {
        return Err(crate::error::SecurityError::Decryption(
            "corrupted ciphertext".into(),
        ));
    }

    let salt = &ciphertext[1..1 + salt_len];
    let encrypted = &ciphertext[1 + salt_len..];

    let mut derived_key = [0u8; 32];
    let argon2 = Argon2::default();
    let passphrase_trimmed = passphrase.trim();

    argon2
        .hash_password_into(passphrase_trimmed.as_bytes(), salt, &mut derived_key)
        .map_err(|e| crate::error::SecurityError::KeyDerivation(e.to_string()))?;

    let key = crate::master_key::MasterKey::from_bytes(&derived_key)?;
    derived_key.zeroize();

    super::decrypt_with_key(&key, encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroizing_bytes_drops() {
        let bytes = ZeroizingBytes(vec![1, 2, 3, 4, 5]);
        {
            let slice: &[u8] = &bytes;
            assert_eq!(slice, &[1, 2, 3, 4, 5]);
        }
        let _ = bytes;
    }

    #[test]
    fn test_from_slice() {
        let bytes = ZeroizingBytes::from(&[1, 2, 3][..]);
        assert_eq!(&*bytes, &[1, 2, 3]);
    }
}
