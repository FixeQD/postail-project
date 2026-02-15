use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use reqwest::Client;
use url::Url;

use crate::error::OAuthError;
use crate::globals;

use super::config::OAuthConfig;
use super::pkce::PkceData;
use super::provider::ProviderKind;
use super::tokens::OAuthTokens;

pub fn is_state_pending(state: &str) -> bool {
    PENDING_FLOWS.lock().unwrap().contains_key(state)
}

const HTTP_TIMEOUT_SECS: u64 = 30;

fn create_http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .unwrap_or_else(|_| Client::new())
}

#[derive(Debug, Clone)]
pub struct Provider {
    pub kind: ProviderKind,
}

impl Provider {
    pub fn gmail() -> Self {
        Self {
            kind: ProviderKind::Gmail,
        }
    }

    pub fn outlook() -> Self {
        Self {
            kind: ProviderKind::Outlook,
        }
    }

    pub fn from_kind(kind: ProviderKind) -> Self {
        Self { kind }
    }

    pub fn config(&self) -> Result<OAuthConfig, OAuthError> {
        OAuthConfig::new(self.kind)
    }
}

impl From<ProviderKind> for Provider {
    fn from(kind: ProviderKind) -> Self {
        Self::from_kind(kind)
    }
}

lazy_static::lazy_static! {
    static ref PENDING_FLOWS: Mutex<HashMap<String, (Provider, PkceData)>> = Mutex::new(HashMap::new());
}

pub fn start_oauth_flow(provider: Provider) -> Result<(String, u16), OAuthError> {
    let config = provider.config()?;
    let pkce = PkceData::generate();

    let mut url = Url::parse(&config.auth_url)?;
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id().unwrap_or_default())
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &config.scope)
        .append_pair("state", &pkce.state)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256");

    let auth_url = url.to_string();

    PENDING_FLOWS
        .lock()
        .unwrap()
        .insert(pkce.state.clone(), (provider, pkce));

    Ok((auth_url, globals::get_oauth_port()))
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
    let client = create_http_client();

    let client_id = config.client_id().unwrap_or_default();

    let mut params = vec![
        ("client_id", client_id.clone()),
        ("code", code.clone()),
        ("grant_type", "authorization_code".to_string()),
        ("redirect_uri", config.redirect_uri.clone()),
        ("code_verifier", pkce.code_verifier.clone()),
    ];

    if let Some(client_secret) = config.client_secret() {
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
    let client = create_http_client();

    let client_id = config.client_id().unwrap_or_default();

    let mut params = vec![
        ("client_id", client_id.clone()),
        ("refresh_token", refresh_token.clone()),
        ("grant_type", "refresh_token".to_string()),
    ];

    if let Some(client_secret) = config.client_secret() {
        params.push(("client_secret", client_secret));
    }

    let response = client.post(&config.token_url).form(&params).send().await?;

    if !response.status().is_success() {
        let status = response.status().to_string();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(target: "postail", "Token refresh failed: {} - {}", status, body);
        return Err(OAuthError::TokenRefreshFailed { status });
    }

    let tokens: OAuthTokens = response.json().await?;
    Ok(tokens)
}
