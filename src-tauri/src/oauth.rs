use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    Gmail,
    Outlook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub auth_url: String,
    pub token_url: String,
    pub scope: String,
    pub redirect_uri: String,
}

impl Provider {
    pub fn config(&self) -> Result<OAuthConfig, Box<dyn std::error::Error>> {
        match self {
            Provider::Gmail => {
                let client_id =
                    option_env!("GMAIL_CLIENT_ID").ok_or("OAuth not implemented for Gmail.")?;
                Ok(OAuthConfig {
                    client_id: client_id.to_string(),
                    auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                    token_url: "https://oauth2.googleapis.com/token".to_string(),
                    scope: "https://mail.google.com/".to_string(), // Full access for IMAP/SMTP
                    redirect_uri: "postail://oauth/callback".to_string(),
                })
            }
            Provider::Outlook => {
                let client_id =
                    option_env!("OUTLOOK_CLIENT_ID").ok_or("OAuth not implemented for Outlook.")?;
                Ok(OAuthConfig {
                    client_id: client_id.to_string(),
                    auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
                    token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                    scope: "https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send".to_string(),
                    redirect_uri: "postail://oauth/callback".to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PkceData {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

impl PkceData {
    pub fn generate() -> Self {
        let code_verifier: String = thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(128)
            .map(char::from)
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(hash);

        let state: String = thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
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

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
}

// Global store for pending OAuth flows (state -> (provider, pkce))
lazy_static::lazy_static! {
    static ref PENDING_FLOWS: Mutex<HashMap<String, (Provider, PkceData)>> = Mutex::new(HashMap::new());
}

pub fn start_oauth_flow(provider: Provider) -> Result<String, Box<dyn std::error::Error>> {
    let config = provider.config()?;
    let pkce = PkceData::generate();

    let mut url = Url::parse(&config.auth_url)?;
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &config.scope)
        .append_pair("state", &pkce.state)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256");

    let auth_url = url.to_string();

    // Store for later verification
    PENDING_FLOWS
        .lock()
        .unwrap()
        .insert(pkce.state.clone(), (provider, pkce));

    Ok(auth_url)
}

pub async fn complete_oauth_flow(
    code: String,
    state: String,
) -> Result<OAuthTokens, Box<dyn std::error::Error>> {
    let (provider, pkce) = PENDING_FLOWS
        .lock()
        .unwrap()
        .remove(&state)
        .ok_or("Invalid or expired state")?;

    let config = provider.config()?;
    let client = Client::new();

    let params = [
        ("client_id", config.client_id.as_str()),
        ("code", &code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", config.redirect_uri.as_str()),
        ("code_verifier", &pkce.code_verifier),
    ];

    let response = client.post(&config.token_url).form(&params).send().await?;

    if !response.status().is_success() {
        return Err(format!("Token exchange failed: {}", response.status()).into());
    }

    let tokens: OAuthTokens = response.json().await?;
    Ok(tokens)
}
