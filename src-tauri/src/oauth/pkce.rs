use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceData {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

impl PkceData {
    pub fn generate() -> Self {
        let code_verifier: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(hash);

        let state: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        Self {
            code_verifier,
            code_challenge,
            state,
        }
    }
}
