use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use url::Url;

use crate::error::OAuthError;
use crate::globals;

fn get_client_secret(provider: &Provider) -> Option<String> {
    match provider {
        Provider::Gmail => option_env!("GMAIL_CLIENT_SECRET").map(|s| s.to_string()),
        Provider::Outlook => option_env!("OUTLOOK_CLIENT_SECRET").map(|s| s.to_string()),
    }
}

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
    pub fn config(&self) -> Result<OAuthConfig, OAuthError> {
        let provider_name = match self {
            Provider::Gmail => "Gmail",
            Provider::Outlook => "Outlook",
        };
        let client_id_env = match self {
            Provider::Gmail => option_env!("GMAIL_CLIENT_ID"),
            Provider::Outlook => option_env!("OUTLOOK_CLIENT_ID"),
        };
        let client_id = client_id_env.ok_or(OAuthError::NotImplemented {
            provider: provider_name.to_string(),
        })?;
        let (auth_url, token_url, scope) = match self {
            Provider::Gmail => (
                "https://accounts.google.com/o/oauth2/v2/auth",
                "https://oauth2.googleapis.com/token",
                "https://mail.google.com/ https://www.googleapis.com/auth/userinfo.email",
            ),
            Provider::Outlook => (
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                "https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send",
            ),
        };

        let port = globals::get_oauth_port();
        if port == 0 {
            panic!("OAuth port is not set");
        }

        Ok(OAuthConfig {
            client_id: client_id.to_string(),
            auth_url: auth_url.to_string(),
            token_url: token_url.to_string(),
            scope: scope.to_string(),
            redirect_uri: format!("http://localhost:{}/oauth/callback", port),
        })
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

pub fn start_oauth_flow(provider: Provider) -> Result<String, OAuthError> {
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
) -> Result<(Provider, OAuthTokens), OAuthError> {
    let (provider, pkce) = PENDING_FLOWS
        .lock()
        .unwrap()
        .remove(&state)
        .ok_or(OAuthError::InvalidState)?;

    let config = provider.config()?;
    let client = Client::new();

    let mut params = vec![
        ("client_id", config.client_id.clone()),
        ("code", code.clone()),
        ("grant_type", "authorization_code".to_string()),
        ("redirect_uri", config.redirect_uri.clone()),
        ("code_verifier", pkce.code_verifier.clone()),
    ];

    if let Some(client_secret) = get_client_secret(&provider) {
        params.push(("client_secret", client_secret));
    }

    let response = client.post(&config.token_url).form(&params).send().await?;

    if !response.status().is_success() {
        return Err(OAuthError::TokenExchangeFailed {
            status: response.status().to_string(),
        });
    }

    let tokens: OAuthTokens = response.json().await?;
    if tokens.refresh_token.is_none() {
        return Err(OAuthError::NoRefreshToken);
    }
    Ok((provider, tokens))
}

pub async fn refresh_access_token(
    provider: Provider,
    refresh_token: String,
) -> Result<OAuthTokens, OAuthError> {
    let config = provider.config()?;
    let client = Client::new();

    let params = [
        ("client_id", config.client_id.as_str()),
        ("refresh_token", &refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let response = client.post(&config.token_url).form(&params).send().await?;

    if !response.status().is_success() {
        return Err(OAuthError::TokenRefreshFailed {
            status: response.status().to_string(),
        });
    }

    let tokens: OAuthTokens = response.json().await?;
    Ok(tokens)
}
