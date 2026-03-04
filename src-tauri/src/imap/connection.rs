// =================================
// I spent my fu**in' like 10h on this so
// Sincerely, f*ck you async-imap
// =================================

use crate::error::ImapError;
use crate::oauth;
use crate::oauth::ProviderKind;
use async_imap::{Authenticator, Client, Session};
use native_tls::TlsConnector as NativeTlsConnector;
use serde_json;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, TlsStream};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

pub type ImapSession = Session<Compat<TlsStream<TcpStream>>>;
pub type ImapSessionPlain = Session<Compat<TcpStream>>;

struct Xoauth2Authenticator {
    email: String,
    access_token: String,
}

impl Xoauth2Authenticator {
    fn new(username: String, access_token: String) -> Self {
        Self {
            email: username,
            access_token,
        }
    }
}

impl Authenticator for Xoauth2Authenticator {
    type Response = String;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.email, self.access_token
        )
    }
}

impl super::ImapManager {
    async fn get_credentials(&self, account_id: &str) -> Result<String, ImapError> {
        let creds_path: String = {
            let conn_guard = self.conn.lock().await;
            let conn = conn_guard.as_ref().ok_or(ImapError::CredentialsFetch(
                "Database not initialized".to_string(),
            ))?;
            let mut stmt = conn
                .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
                .map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
            let path: String = stmt
                .query_row([account_id], |row| row.get(0))
                .map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
            path
        };
        let creds_path = crate::db::resolve_creds_path(&creds_path);

        let security = self.security.lock().await;
        let encrypted =
            std::fs::read(&creds_path).map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
        let decrypted = security
            .decrypt(&encrypted)
            .map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
        let creds_json =
            String::from_utf8(decrypted).map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
        Ok(creds_json)
    }

    async fn refresh_oauth_if_needed(
        &self,
        account_id: &str,
        auth_type: &str,
        host: &str,
        creds: &mut serde_json::Value,
    ) -> Result<(), ImapError> {
        use chrono::Utc;

        if auth_type != "oauth2" {
            return Ok(());
        }

        let expires_at = creds
            .get("expires_at")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let refresh_token = creds.get("refresh_token").and_then(|v| v.as_str());

        tracing::debug!(target: "postail", "OAuth check for {}: expires_at={}, has_refresh={}",
            account_id, expires_at, refresh_token.is_some());

        let now = Utc::now().timestamp();
        let seconds_until_expiry = expires_at.saturating_sub(now);

        tracing::debug!(target: "postail", "Token expires in {} seconds (now={})", seconds_until_expiry, now);

        let provider_kind =
            ProviderKind::from_imap_host(host).ok_or(ImapError::UnknownOAuthProvider)?;

        if seconds_until_expiry < 300 {
            if let Some(refresh_token) = refresh_token {
                tracing::info!(target: "postail", "Token expiring soon ({}s), refreshing for provider: {}", seconds_until_expiry, provider_kind.as_str());
                let provider = oauth::Provider::from_kind(provider_kind);

                match oauth::refresh_access_token(provider, refresh_token.to_string()).await {
                    Ok(new_tokens) => {
                        tracing::debug!(target: "postail", "Token refresh successful, new expires_in={}", new_tokens.expires_in);
                        creds["access_token"] = serde_json::Value::String(new_tokens.access_token);
                        if let Some(rt) = new_tokens.refresh_token {
                            creds["refresh_token"] = serde_json::Value::String(rt);
                        }
                        creds["expires_at"] = serde_json::Value::Number(serde_json::Number::from(
                            Utc::now().timestamp() + new_tokens.expires_in as i64,
                        ));

                        let security = self.security.lock().await;
                        let creds_path: String = {
                            let conn_guard = self.conn.lock().await;
                            let conn = conn_guard.as_ref().ok_or(ImapError::CredentialsFetch(
                                "Database not initialized".to_string(),
                            ))?;
                            let mut stmt = conn
                                .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
                                .map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
                            stmt.query_row([account_id], |row| row.get::<_, String>(0))
                                .map_err(|e| ImapError::CredentialsFetch(e.to_string()))
                        }?;
                        let creds_path = crate::db::resolve_creds_path(&creds_path);

                        let creds_json = creds.to_string();
                        let encrypted = security
                            .encrypt(creds_json.as_bytes())
                            .map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;
                        std::fs::write(&creds_path, encrypted)
                            .map_err(|e| ImapError::CredentialsFetch(e.to_string()))?;

                        tracing::info!(target: "postail", "Token refreshed successfully!");
                    }
                    Err(e) => {
                        tracing::error!(target: "postail", "Token refresh failed: {}", e);
                        return Err(ImapError::OAuthRefresh(e.to_string()));
                    }
                }
            } else {
                tracing::warn!(target: "postail", "Token expiring but no refresh token available");
            }
        }

        Ok(())
    }

    pub async fn connect_imap(&self, account_id: &str) -> Result<ImapSession, ImapError> {
        tracing::debug!(target: "postail", "[IMAP] connect_imap: starting for {}", account_id);

        let (host, port, use_tls, auth_type, email) = {
            let conn_guard = self.conn.lock().await;
            let conn = conn_guard.as_ref().ok_or(ImapError::Connection(
                "Database not initialized".to_string(),
            ))?;
            let mut stmt = conn
                .prepare("SELECT imap_host, imap_port, imap_tls, auth_type, email FROM accounts WHERE id = ?")
                .map_err(|e| ImapError::Connection(e.to_string()))?;
            stmt.query_row([account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? as u16,
                    row.get::<_, bool>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| ImapError::Connection(e.to_string()))?
        };

        tracing::debug!(target: "postail", "[IMAP] connect_imap: host={}, port={}, auth_type={}, use_tls={}", host, port, auth_type, use_tls);

        let creds_json = self.get_credentials(account_id).await?;
        tracing::debug!(target: "postail", "[IMAP] connect_imap: credentials retrieved");

        let mut creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| ImapError::Connection(e.to_string()))?;

        self.refresh_oauth_if_needed(account_id, &auth_type, &host, &mut creds)
            .await?;

        if use_tls {
            self.connect_imap_tls(account_id, &host, port, &auth_type, &email, &creds)
                .await
        } else {
            Err(ImapError::Connection(
                "Non-TLS IMAP connections are not currently supported. Please enable TLS."
                    .to_string(), // I'm f**kin done with this 😭
            ))
        }
    }

    async fn connect_imap_tls(
        &self,
        _account_id: &str,
        host: &str,
        port: u16,
        auth_type: &str,
        email: &str,
        creds: &serde_json::Value,
    ) -> Result<ImapSession, ImapError> {
        tracing::debug!(target: "postail", "[IMAP] connect_imap_tls: connecting TCP to {}:{}", host, port);
        let tcp_stream = TcpStream::connect((host, port))
            .await
            .map_err(|e: std::io::Error| ImapError::Connection(e.to_string()))?;
        tracing::debug!(target: "postail", "[IMAP] connect_imap_tls: TCP connected, starting TLS");

        let native_tls_connector = NativeTlsConnector::new()
            .map_err(|e: native_tls::Error| ImapError::Connection(e.to_string()))?;
        let tls_connector = TlsConnector::from(native_tls_connector);
        let tls_stream = tls_connector
            .connect(host, tcp_stream)
            .await
            .map_err(|e: native_tls::Error| ImapError::Connection(e.to_string()))?;
        tracing::debug!(target: "postail", "[IMAP] connect_imap_tls: TLS connected");

        let mut client = Client::new(tls_stream.compat());

        // Initial greeting
        client
            .read_response()
            .await
            .map_err(|e| ImapError::Connection(e.to_string()))?;

        self.authenticate_imap(client, auth_type, email, creds)
            .await
    }

    async fn authenticate_imap(
        &self,
        client: Client<Compat<TlsStream<TcpStream>>>,
        auth_type: &str,
        email: &str,
        creds: &serde_json::Value,
    ) -> Result<ImapSession, ImapError> {
        if auth_type == "oauth2" {
            let access_token = creds["access_token"]
                .as_str()
                .ok_or_else(|| ImapError::Login("No access_token".to_string()))?;

            tracing::debug!(target: "postail", "[IMAP] authenticate_imap: XOAUTH2 via authenticate for '{}'", email);

            let authenticator =
                Xoauth2Authenticator::new(email.to_string(), access_token.to_string());

            match client.authenticate("XOAUTH2", authenticator).await {
                Ok(session) => {
                    tracing::info!(target: "postail", "[IMAP] authenticate_imap: XOAUTH2 successful");
                    Ok(session)
                }
                Err((e, client)) => {
                    tracing::error!(target: "postail", "[IMAP] authenticate_imap: XOAUTH2 failed, trying OAUTHBEARER: {}", e);
                    let authenticator =
                        Xoauth2Authenticator::new(email.to_string(), access_token.to_string());
                    match client.authenticate("OAUTHBEARER", authenticator).await {
                        Ok(session) => {
                            tracing::info!(target: "postail", "[IMAP] authenticate_imap: OAUTHBEARER successful");
                            Ok(session)
                        }
                        Err((e2, _)) => {
                            tracing::error!(target: "postail", "[IMAP] authenticate_imap: Both OAUTH methods failed: {}, {}", e, e2);
                            Err(ImapError::Login(format!(
                                "XOAUTH2: {}; OAUTHBEARER: {}",
                                e, e2
                            )))
                        }
                    }
                }
            }
        } else {
            let username = creds["username"]
                .as_str()
                .ok_or_else(|| ImapError::Login("No username".to_string()))?;
            let password = creds["password"]
                .as_str()
                .ok_or_else(|| ImapError::Login("No password".to_string()))?;
            match client.login(username, password).await {
                Ok(session) => {
                    tracing::info!(target: "postail", "[IMAP] authenticate_imap: login successful");
                    Ok(session)
                }
                Err((e, _)) => {
                    tracing::error!(target: "postail", "[IMAP] authenticate_imap: login failed: {}", e);
                    Err(ImapError::Login(e.to_string()))
                }
            }
        }
    }

    #[allow(dead_code)]
    async fn authenticate_imap_plain(
        &self,
        client: Client<Compat<TcpStream>>,
        auth_type: &str,
        email: &str,
        creds: &serde_json::Value,
    ) -> Result<ImapSessionPlain, ImapError> {
        if auth_type == "oauth2" {
            let access_token = creds["access_token"]
                .as_str()
                .ok_or_else(|| ImapError::Login("No access_token".to_string()))?;

            tracing::debug!(target: "postail", "[IMAP] authenticate_imap_plain: XOAUTH2 via authenticate for '{}'", email);

            let authenticator =
                Xoauth2Authenticator::new(email.to_string(), access_token.to_string());

            match client.authenticate("XOAUTH2", authenticator).await {
                Ok(session) => {
                    tracing::info!(target: "postail", "[IMAP] authenticate_imap_plain: XOAUTH2 successful");
                    Ok(session)
                }
                Err((e, client)) => {
                    tracing::error!(target: "postail", "[IMAP] authenticate_imap_plain: XOAUTH2 failed, trying OAUTHBEARER: {}", e);
                    let authenticator =
                        Xoauth2Authenticator::new(email.to_string(), access_token.to_string());
                    match client.authenticate("OAUTHBEARER", authenticator).await {
                        Ok(session) => {
                            tracing::info!(target: "postail", "[IMAP] authenticate_imap_plain: OAUTHBEARER successful");
                            Ok(session)
                        }
                        Err((e2, _)) => {
                            tracing::error!(target: "postail", "[IMAP] authenticate_imap_plain: Both OAUTH methods failed: {}, {}", e, e2);
                            Err(ImapError::Login(format!(
                                "XOAUTH2: {}; OAUTHBEARER: {}",
                                e, e2
                            )))
                        }
                    }
                }
            }
        } else {
            let username = creds["username"]
                .as_str()
                .ok_or_else(|| ImapError::Login("No username".to_string()))?;
            let password = creds["password"]
                .as_str()
                .ok_or_else(|| ImapError::Login("No password".to_string()))?;
            match client.login(username, password).await {
                Ok(session) => {
                    tracing::info!(target: "postail", "[IMAP] authenticate_imap_plain: login successful");
                    Ok(session)
                }
                Err((e, _)) => {
                    tracing::error!(target: "postail", "[IMAP] authenticate_imap_plain: login failed: {}", e);
                    Err(ImapError::Login(e.to_string()))
                }
            }
        }
    }
}
