use super::provider::{ProviderInfo, ProviderKind};
use crate::error::OAuthError;
use crate::globals;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub provider: ProviderInfo,
    pub auth_url: String,
    pub token_url: String,
    pub scope: String,
    pub redirect_uri: String,
}

impl OAuthConfig {
    pub fn new(kind: ProviderKind) -> Result<Self, OAuthError> {
        let info = ProviderInfo::get(kind);

        let _client_id = info.client_id().ok_or(OAuthError::NotImplemented {
            provider: info.name.to_string(),
        })?;

        let port = globals::get_oauth_port();
        if port == 0 {
            panic!("OAuth port is not set");
        }

        Ok(OAuthConfig {
            provider: info.clone(),
            auth_url: info.auth_url.to_string(),
            token_url: info.token_url.to_string(),
            scope: info.scopes.to_string(),
            redirect_uri: format!("http://localhost:{}/oauth/callback", port),
        })
    }

    pub fn client_id(&self) -> Option<String> {
        self.provider.client_id()
    }

    pub fn client_secret(&self) -> Option<String> {
        self.provider.client_secret()
    }
}
