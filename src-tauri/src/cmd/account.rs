use crate::db::accounts::{
    add_account as db_add_account, list_accounts as db_list_accounts,
    remove_account as db_remove_account,
};
use crate::db::{
    AccountInput, AccountMeta, Credentials, ImapConfig, ManualServerConfig, OAuthCredentials,
    PasswordCredentials, SmtpConfig,
};
use crate::globals::{DB_CONN, IMAP_MANAGER, SECURITY};
use crate::oauth;
use crate::utils::oauth_server;
use async_imap::Client as ImapClient;
use async_smtp::authentication::{Credentials as SmtpCredentials, Mechanism};
use async_smtp::extension::ClientId;
use async_smtp::{SmtpClient, SmtpTransport};
use chrono::Utc;
use futures::TryStreamExt;
use native_tls::TlsConnector as NativeTlsConnector;
use serde::Serialize;
use std::time::Duration;
use tauri::{command, AppHandle};
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;

const HTTP_TIMEOUT_SECS: Duration = Duration::from_secs(30);
const CONNECTION_TIMEOUT_SECS: Duration = Duration::from_secs(10);

#[command]
pub async fn add_account(input: AccountInput) -> Result<AccountMeta, String> {
    let account = {
        let (conn_guard, security) = {
            let conn_guard = DB_CONN.lock().await;
            let security = SECURITY.lock().await;
            (conn_guard, security)
        };
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        db_add_account(conn, input, &security).map_err(|e| e.to_string())?
    };

    // Only sync folder list
    if let Err(e) = {
        let imap = IMAP_MANAGER.lock().await;
        imap.fetch_mailboxes(&account.id).await
    } {
        tracing::warn!(target: "postail", "[Account] Failed to sync mailbox list for new account {}: {}", account.id, e);
    } else {
        tracing::info!(target: "postail", "[Account] Synced mailbox list for new account {}", account.id);
    }

    Ok(account)
}

#[derive(Debug)]
enum ConnectionTestError {
    Imap(String),
    Smtp(String),
}

impl std::fmt::Display for ConnectionTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionTestError::Imap(msg) => write!(f, "IMAP: {}", msg),
            ConnectionTestError::Smtp(msg) => write!(f, "SMTP: {}", msg),
        }
    }
}

async fn test_imap_connection(config: &ManualServerConfig) -> Result<(), ConnectionTestError> {
    let username = config.get_username();
    let password = &config.password;

    tracing::info!(target: "postail", "[Test IMAP] Connecting to {}:{}", config.imap_host, config.imap_port);

    let tcp_stream = tokio::time::timeout(
        CONNECTION_TIMEOUT_SECS,
        TcpStream::connect((config.imap_host.as_str(), config.imap_port)),
    )
    .await
    .map_err(|_| ConnectionTestError::Imap("Connection timeout".to_string()))?
    .map_err(|e| ConnectionTestError::Imap(format!("TCP connection failed: {}", e)))?;

    if config.imap_tls {
        let native_tls_connector = NativeTlsConnector::new()
            .map_err(|e| ConnectionTestError::Imap(format!("TLS setup failed: {}", e)))?;
        let tls_connector = TlsConnector::from(native_tls_connector);
        let tls_stream = tls_connector
            .connect(&config.imap_host, tcp_stream)
            .await
            .map_err(|e| ConnectionTestError::Imap(format!("TLS handshake failed: {}", e)))?;

        let mut client = ImapClient::new(tls_stream.compat());

        client.read_response().await.map_err(|e| {
            ConnectionTestError::Imap(format!("Failed to read server greeting: {}", e))
        })?;

        match client.login(username, password).await {
            Ok(mut session) => {
                let _mailboxes: Vec<_> = session
                    .list(None, Some("*"))
                    .await
                    .map_err(|e| ConnectionTestError::Imap(format!("LIST failed: {}", e)))?
                    .try_collect()
                    .await
                    .map_err(|e| ConnectionTestError::Imap(format!("LIST stream failed: {}", e)))?;
                let _ = session.logout().await;
                tracing::info!(target: "postail", "[Test IMAP] Connection test successful");
                Ok(())
            }
            Err((e, _)) => Err(ConnectionTestError::Imap(format!(
                "Authentication failed: {}",
                e
            ))),
        }
    } else {
        let mut client = ImapClient::new(tcp_stream.compat());

        client.read_response().await.map_err(|e| {
            ConnectionTestError::Imap(format!("Failed to read server greeting: {}", e))
        })?;

        match client.login(username, password).await {
            Ok(mut session) => {
                let _mailboxes: Vec<_> = session
                    .list(None, Some("*"))
                    .await
                    .map_err(|e| ConnectionTestError::Imap(format!("LIST failed: {}", e)))?
                    .try_collect()
                    .await
                    .map_err(|e| ConnectionTestError::Imap(format!("LIST stream failed: {}", e)))?;
                let _ = session.logout().await;
                tracing::info!(target: "postail", "[Test IMAP] Connection test successful (plain)");
                Ok(())
            }
            Err((e, _)) => Err(ConnectionTestError::Imap(format!(
                "Authentication failed: {}",
                e
            ))),
        }
    }
}

async fn test_smtp_connection(config: &ManualServerConfig) -> Result<(), ConnectionTestError> {
    let username = config.get_username();
    let password = &config.password;

    tracing::info!(target: "postail", "[Test SMTP] Connecting to {}:{}", config.smtp_host, config.smtp_port);

    let tcp_stream = tokio::time::timeout(
        CONNECTION_TIMEOUT_SECS,
        TcpStream::connect((config.smtp_host.as_str(), config.smtp_port)),
    )
    .await
    .map_err(|_| ConnectionTestError::Smtp("Connection timeout".to_string()))?
    .map_err(|e| ConnectionTestError::Smtp(format!("TCP connection failed: {}", e)))?;

    let client = SmtpClient::new().hello_name(ClientId::new("postail".to_string()));

    if config.smtp_tls && config.smtp_port == 465 {
        let native_tls = NativeTlsConnector::builder()
            .build()
            .map_err(|e| ConnectionTestError::Smtp(format!("TLS setup failed: {}", e)))?;
        let tls_connector = TlsConnector::from(native_tls);
        let tls_stream = tls_connector
            .connect(&config.smtp_host, tcp_stream)
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("TLS handshake failed: {}", e)))?;

        let stream = BufStream::new(tls_stream);
        let mut transport = SmtpTransport::new(client, stream)
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("SMTP handshake failed: {}", e)))?;

        let credentials = SmtpCredentials::new(username.to_string(), password.clone());
        transport
            .try_login(&credentials, &[Mechanism::Plain])
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("Authentication failed: {}", e)))?;

        let _ = transport.quit().await;
        tracing::info!(target: "postail", "[Test SMTP] Connection test successful (TLS)");
        Ok(())
    } else if config.smtp_tls {
        let stream = BufStream::new(tcp_stream);
        let transport = SmtpTransport::new(client, stream)
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("SMTP handshake failed: {}", e)))?;

        let plain_stream = transport
            .starttls()
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("STARTTLS failed: {}", e)))?;

        let native_tls = NativeTlsConnector::builder()
            .build()
            .map_err(|e| ConnectionTestError::Smtp(format!("TLS setup failed: {}", e)))?;
        let tls_connector = TlsConnector::from(native_tls);
        let tls_stream = tls_connector
            .connect(&config.smtp_host, plain_stream)
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("TLS handshake failed: {}", e)))?;

        let stream = BufStream::new(tls_stream);
        let client = SmtpClient::new()
            .hello_name(ClientId::new("postail".to_string()))
            .without_greeting();
        let mut transport = SmtpTransport::new(client, stream).await.map_err(|e| {
            ConnectionTestError::Smtp(format!("SMTP handshake after STARTTLS failed: {}", e))
        })?;

        transport
            .get_mut()
            .ehlo(ClientId::new("postail".to_string()))
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("EHLO after STARTTLS failed: {}", e)))?;

        let credentials = SmtpCredentials::new(username.to_string(), password.clone());
        transport
            .try_login(&credentials, &[Mechanism::Plain])
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("Authentication failed: {}", e)))?;

        let _ = transport.quit().await;
        tracing::info!(target: "postail", "[Test SMTP] Connection test successful (STARTTLS)");
        Ok(())
    } else {
        let stream = BufStream::new(tcp_stream);
        let mut transport = SmtpTransport::new(client, stream)
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("SMTP handshake failed: {}", e)))?;

        let credentials = SmtpCredentials::new(username.to_string(), password.clone());
        transport
            .try_login(&credentials, &[Mechanism::Plain])
            .await
            .map_err(|e| ConnectionTestError::Smtp(format!("Authentication failed: {}", e)))?;

        let _ = transport.quit().await;
        tracing::info!(target: "postail", "[Test SMTP] Connection test successful (plain)");
        Ok(())
    }
}

#[command]
pub async fn add_custom_account(config: ManualServerConfig) -> Result<AccountMeta, String> {
    config.validate()?;

    tracing::info!(target: "postail", "[add_custom_account] Testing connections for {}", config.email);

    if let Err(e) = test_imap_connection(&config).await {
        tracing::error!(target: "postail", "[add_custom_account] IMAP test failed: {}", e);
        return Err(e.to_string());
    }

    if let Err(e) = test_smtp_connection(&config).await {
        tracing::error!(target: "postail", "[add_custom_account] SMTP test failed: {}", e);
        return Err(e.to_string());
    }

    tracing::info!(target: "postail", "[add_custom_account] Connection tests passed, saving account");

    let username = config.get_username().to_string();
    let account_input = AccountInput {
        name: config.account_name.clone(),
        email: config.email.clone(),
        provider_type: "custom".to_string(),
        auth_type: "password".to_string(),
        credentials: Credentials::Password(PasswordCredentials {
            username,
            password: config.password.clone(),
        }),
        imap_config: ImapConfig {
            host: config.imap_host.clone(),
            port: config.imap_port,
            tls: config.imap_tls,
        },
        smtp_config: SmtpConfig {
            host: config.smtp_host.clone(),
            port: config.smtp_port,
            tls: config.smtp_tls,
        },
    };

    let account = {
        let (conn_guard, security) = {
            let conn_guard = DB_CONN.lock().await;
            let security = SECURITY.lock().await;
            (conn_guard, security)
        };
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        db_add_account(conn, account_input, &security).map_err(|e| e.to_string())?
    };

    if let Err(e) = {
        let imap = IMAP_MANAGER.lock().await;
        imap.fetch_mailboxes(&account.id).await
    } {
        tracing::warn!(target: "postail", "[add_custom_account] Failed to sync mailbox list for new account {}: {}", account.id, e);
    } else {
        tracing::info!(target: "postail", "[add_custom_account] Synced mailbox list for new account {}", account.id);
    }

    Ok(account)
}

#[command]
pub async fn list_accounts() -> Result<Vec<AccountMeta>, String> {
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db_list_accounts(conn).map_err(|e| e.to_string())
}

#[command]
pub async fn remove_account(id: String) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db_remove_account(conn, &id).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct OAuthFlowResponse {
    pub url: String,
    pub port: u16,
}

#[command]
pub async fn start_oauth_flow(
    app: AppHandle,
    provider: String,
) -> Result<OAuthFlowResponse, String> {
    oauth_server::start(app.clone());
    let provider_kind =
        oauth::ProviderKind::parse(&provider).ok_or_else(|| "Unknown provider".to_string())?;
    let provider = oauth::Provider::from_kind(provider_kind);
    match oauth::start_oauth_flow(provider) {
        Ok((url, port)) => Ok(OAuthFlowResponse { url, port }),
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub async fn complete_oauth_flow(
    code: String,
    state: String,
    code_verifier: String,
    provider_type: String,
) -> Result<AccountMeta, String> {
    let (provider, tokens) =
        match oauth::complete_oauth_flow(code, state, code_verifier, provider_type).await {
            Ok(result) => result,
            Err(e) => return Err(e.to_string()),
        };

    let provider_info = oauth::ProviderInfo::get(provider.kind);
    let email = {
        let client = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT_SECS)
            .build()
            .map_err(|e| e.to_string())?;
        let response = client
            .get(provider_info.user_info_url())
            .bearer_auth(&tokens.access_token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            let status = response.status().to_string();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            tracing::error!(target: "postail", "Failed to fetch {} user info. Status: {}, Body: {}", provider.kind, status, body);
            return Err(format!("Failed to fetch {} user info", provider.kind));
        }
        let user_info: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        provider_info
            .extract_email(&user_info)
            .ok_or_else(|| format!("No email in {} response", provider.kind))?
    };

    let account_input = AccountInput {
        name: format!("{} Account", provider.kind.display_name()),
        email,
        provider_type: provider.kind.as_str().to_string(),
        auth_type: "oauth2".to_string(),
        credentials: Credentials::OAuth(OAuthCredentials {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_at: Utc::now().timestamp() + tokens.expires_in as i64,
            auth_type: "oauth2".to_string(),
            provider_type: provider.kind.as_str().to_string(),
        }),
        imap_config: ImapConfig {
            host: oauth::ProviderInfo::get(provider.kind)
                .imap_host
                .to_string(),
            port: 993,
            tls: true,
        },
        smtp_config: SmtpConfig {
            host: oauth::ProviderInfo::get(provider.kind)
                .smtp_host
                .to_string(),
            port: 587,
            tls: true,
        },
    };

    let account = {
        let (conn_guard, security) = {
            let conn_guard = DB_CONN.lock().await;
            let security = SECURITY.lock().await;
            (conn_guard, security)
        };
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        db_add_account(conn, account_input, &security).map_err(|e| e.to_string())?
    };

    // Only sync folder list
    if let Err(e) = {
        let imap = IMAP_MANAGER.lock().await;
        imap.fetch_mailboxes(&account.id).await
    } {
        tracing::warn!(target: "postail", "[Account] Failed to sync mailbox list for new OAuth account {}: {}", account.id, e);
    } else {
        tracing::info!(target: "postail", "[Account] Synced mailbox list for new OAuth account {}", account.id);
    }

    Ok(account)
}

#[derive(Serialize)]
pub struct AvailableProviders {
    pub providers: Vec<String>,
}

#[command]
pub fn get_available_providers() -> AvailableProviders {
    use crate::oauth::{ProviderInfo, ProviderKind};

    let providers = ProviderKind::all()
        .iter()
        .filter_map(|kind| {
            ProviderInfo::get(*kind)
                .client_id()
                .map(|_| kind.as_str().to_string())
        })
        .collect();

    AvailableProviders { providers }
}
