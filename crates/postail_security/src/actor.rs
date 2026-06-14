use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;

use crate::error::{Result, SecurityError};
use crate::crypto::Crypto;

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
                            crate::crypto::helpers::encrypt_with_passphrase(
                                &data, &passphrase,
                            )
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
                            crate::crypto::helpers::decrypt_with_passphrase(
                                &data, &passphrase,
                            )
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
