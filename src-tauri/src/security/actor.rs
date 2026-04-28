use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;

use crate::error::{Result, SecurityError};
use crate::security::crypto::Crypto;

enum CryptoRequest {
    Encrypt {
        data: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<u8>>>,
    },
    Decrypt {
        data: Vec<u8>,
        reply: oneshot::Sender<Result<Vec<u8>>>,
    },
    EncryptWithPassphrase {
        data: Vec<u8>,
        passphrase: String,
        reply: oneshot::Sender<Result<Vec<u8>>>,
    },
    DecryptWithPassphrase {
        data: Vec<u8>,
        passphrase: String,
        reply: oneshot::Sender<Result<Vec<u8>>>,
    },
    GetCrypto {
        reply: oneshot::Sender<Result<Crypto>>,
    },
}

pub struct CryptoHandle {
    tx: Sender<CryptoRequest>,
}

impl CryptoHandle {
    pub fn new(crypto: Crypto) -> Self {
        let (tx, mut rx) = mpsc::channel::<CryptoRequest>(64);

        tokio::spawn(async move {
            let crypto = Some(crypto);

            while let Some(req) = rx.recv().await {
                match req {
                    CryptoRequest::Encrypt { data, reply } => {
                        let result = if let Some(c) = &crypto {
                            c.encrypt(&data)
                        } else {
                            Err(SecurityError::MasterKeyNotFound)
                        };
                        let _ = reply.send(result);
                    }
                    CryptoRequest::Decrypt { data, reply } => {
                        let result = if let Some(c) = &crypto {
                            c.decrypt(&data)
                        } else {
                            Err(SecurityError::MasterKeyNotFound)
                        };
                        let _ = reply.send(result);
                    }
                    CryptoRequest::EncryptWithPassphrase {
                        data,
                        passphrase,
                        reply,
                    } => {
                        let result = tokio::task::spawn_blocking(move || {
                            encrypt_with_passphrase_blocking(&data, &passphrase)
                        })
                        .await
                        .unwrap_or_else(|e| {
                            Err(SecurityError::Encryption(format!(
                                "blocking task failed: {}",
                                e
                            )))
                        });
                        let _ = reply.send(result);
                    }
                    CryptoRequest::DecryptWithPassphrase {
                        data,
                        passphrase,
                        reply,
                    } => {
                        let result = tokio::task::spawn_blocking(move || {
                            decrypt_with_passphrase_blocking(&data, &passphrase)
                        })
                        .await
                        .unwrap_or_else(|e| {
                            Err(SecurityError::Decryption(format!(
                                "blocking task failed: {}",
                                e
                            )))
                        });
                        let _ = reply.send(result);
                    }
                    CryptoRequest::GetCrypto { reply } => {
                        let result = if let Some(c) = &crypto {
                            Ok(c.clone())
                        } else {
                            Err(SecurityError::MasterKeyNotFound)
                        };
                        let _ = reply.send(result);
                    }
                }
            }
        });

        Self { tx }
    }

    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(CryptoRequest::Encrypt {
                data: data.to_vec(),
                reply: reply_tx,
            })
            .await
            .map_err(|_| SecurityError::ActorDead)?;
        reply_rx
            .await
            .map_err(|_| SecurityError::ActorDead)?
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(CryptoRequest::Decrypt {
                data: data.to_vec(),
                reply: reply_tx,
            })
            .await
            .map_err(|_| SecurityError::ActorDead)?;
        reply_rx
            .await
            .map_err(|_| SecurityError::ActorDead)?
    }

    pub async fn encrypt_with_passphrase(
        &self,
        data: &[u8],
        passphrase: &str,
    ) -> Result<Vec<u8>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(CryptoRequest::EncryptWithPassphrase {
                data: data.to_vec(),
                passphrase: passphrase.to_string(),
                reply: reply_tx,
            })
            .await
            .map_err(|_| SecurityError::ActorDead)?;
        reply_rx
            .await
            .map_err(|_| SecurityError::ActorDead)?
    }

    pub async fn decrypt_with_passphrase(
        &self,
        data: &[u8],
        passphrase: &str,
    ) -> Result<Vec<u8>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(CryptoRequest::DecryptWithPassphrase {
                data: data.to_vec(),
                passphrase: passphrase.to_string(),
                reply: reply_tx,
            })
            .await
            .map_err(|_| SecurityError::ActorDead)?;
        reply_rx
            .await
            .map_err(|_| SecurityError::ActorDead)?
    }

    pub async fn get_crypto(&self) -> Result<Crypto> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(CryptoRequest::GetCrypto { reply: reply_tx })
            .await
            .map_err(|_| SecurityError::ActorDead)?;
        reply_rx
            .await
            .map_err(|_| SecurityError::ActorDead)?
    }
}

// ── Blocking helpers for Argon2 ─────────────────────────────────────

fn encrypt_with_passphrase_blocking(
    plaintext: &[u8],
    passphrase: &str,
) -> Result<Vec<u8>> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::Argon2;
    use zeroize::Zeroize;

    let salt = SaltString::generate(&mut OsRng);
    let salt_bytes = salt.as_str().as_bytes();

    let mut derived_key = [0u8; 32];
    let argon2 = Argon2::default();
    let passphrase_trimmed = passphrase.trim();

    argon2
        .hash_password_into(passphrase_trimmed.as_bytes(), salt_bytes, &mut derived_key)
        .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;

    let key = crate::security::master_key::MasterKey::from_bytes(&derived_key)?;
    derived_key.zeroize();

    let encrypted = crate::security::crypto::encrypt_with_key(&key, plaintext)?;

    let mut data = Vec::with_capacity(1 + salt_bytes.len() + encrypted.len());
    data.push(salt_bytes.len() as u8);
    data.extend_from_slice(salt_bytes);
    data.extend_from_slice(&encrypted);

    Ok(data)
}

fn decrypt_with_passphrase_blocking(
    ciphertext: &[u8],
    passphrase: &str,
) -> Result<Vec<u8>> {
    use argon2::Argon2;
    use zeroize::Zeroize;

    if ciphertext.is_empty() {
        return Err(SecurityError::Decryption("empty ciphertext".into()));
    }

    let salt_len = ciphertext[0] as usize;
    if ciphertext.len() < 1 + salt_len {
        return Err(SecurityError::Decryption("corrupted ciphertext".into()));
    }

    let salt = &ciphertext[1..1 + salt_len];
    let encrypted = &ciphertext[1 + salt_len..];

    let mut derived_key = [0u8; 32];
    let argon2 = Argon2::default();
    let passphrase_trimmed = passphrase.trim();

    argon2
        .hash_password_into(passphrase_trimmed.as_bytes(), salt, &mut derived_key)
        .map_err(|e| SecurityError::KeyDerivation(e.to_string()))?;

    let key = crate::security::master_key::MasterKey::from_bytes(&derived_key)?;
    derived_key.zeroize();

    crate::security::crypto::decrypt_with_key(&key, encrypted)
}
