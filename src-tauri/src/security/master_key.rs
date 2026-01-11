use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Result, SecurityError};

pub const MASTER_KEY_LENGTH: usize = 32;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; MASTER_KEY_LENGTH]);

impl MasterKey {
    pub fn generate() -> Self {
        let mut key = [0u8; MASTER_KEY_LENGTH];
        rand::thread_rng().fill_bytes(&mut key);
        Self(key)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != MASTER_KEY_LENGTH {
            return Err(SecurityError::InvalidKeyLength {
                expected: MASTER_KEY_LENGTH,
                got: bytes.len(),
            });
        }

        let mut key = [0u8; MASTER_KEY_LENGTH];
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }

    pub fn as_bytes(&self) -> &[u8; MASTER_KEY_LENGTH] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasterKey")
            .field("key", &"[REDACTED]")
            .finish()
    }
}
