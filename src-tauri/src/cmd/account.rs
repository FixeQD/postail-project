use crate::db::accounts::{
    add_account as db_add_account, list_accounts as db_list_accounts,
    remove_account as db_remove_account,
};
use crate::db::{AccountInput, AccountMeta, Credentials, ImapConfig, OAuthCredentials, SmtpConfig};
use crate::globals::{DB_CONN, IMAP_MANAGER, SECURITY};
use crate::oauth;
use crate::utils::oauth_server;
use chrono::Utc;
use serde::Serialize;
use std::time::Duration;
use tauri::{command, AppHandle};

const HTTP_TIMEOUT_SECS: Duration = Duration::from_secs(30);

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
pub async fn complete_oauth_flow(code: String, state: String) -> Result<AccountMeta, String> {
    let (provider, tokens) = match oauth::complete_oauth_flow(code, state).await {
        Ok(result) => result,
        Err(e) => return Err(e.to_string()),
    };

    let email = match provider.kind {
        oauth::ProviderKind::Gmail => {
            let client = reqwest::Client::builder()
                .timeout(HTTP_TIMEOUT_SECS)
                .build()
                .map_err(|e| e.to_string())?;
            let response = client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
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
                tracing::error!(target: "postail", "Failed to fetch Gmail user info. Status: {}, Body: {}", status, body);
                return Err("Failed to fetch Gmail user info".to_string());
            }
            let user_info: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            user_info["email"]
                .as_str()
                .ok_or("No email in Gmail response")?
                .to_string()
        }
        oauth::ProviderKind::Outlook => {
            let client = reqwest::Client::builder()
                .timeout(HTTP_TIMEOUT_SECS)
                .build()
                .map_err(|e| e.to_string())?;
            let response = client
                .get("https://graph.microsoft.com/v1.0/me")
                .bearer_auth(&tokens.access_token)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                return Err("Failed to fetch Outlook user info".to_string());
            }
            let user_info: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
            user_info["mail"]
                .as_str()
                .or_else(|| user_info["userPrincipalName"].as_str())
                .ok_or("No email in Outlook response")?
                .to_string()
        }
    };

    let account_input = AccountInput {
        name: format!("{} Account", provider.kind.display_name()),
        email,
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
