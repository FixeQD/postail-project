use crate::oauth;
use async_imap::{Client, Session};
use async_native_tls::{TlsConnector, TlsStream};
use async_std::net::TcpStream;
use serde_json;

impl super::ImapManager {
    fn get_credentials(&self, account_id: &str) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let creds_path: String = stmt
            .query_row([account_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        drop(stmt);
        drop(conn);

        let security = self.security.lock().unwrap();
        let encrypted = std::fs::read(&creds_path).map_err(|e| e.to_string())?;
        let decrypted = security.decrypt(&encrypted).map_err(|e| e.to_string())?;
        let creds_json = String::from_utf8(decrypted).map_err(|e| e.to_string())?;
        Ok(creds_json)
    }

    async fn refresh_oauth_if_needed(
        &self,
        account_id: &str,
        creds: &mut serde_json::Value,
    ) -> Result<(), String> {
        let auth_type = creds
            .get("auth_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if auth_type != "oauth2" {
            return Ok(());
        }

        let expires_in = creds
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let refresh_token = creds.get("refresh_token").and_then(|v| v.as_str());
        let provider_type = creds
            .get("provider_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        if expires_in < 300 && refresh_token.is_some() {
            let provider = match provider_type {
                "gmail" => oauth::Provider::Gmail,
                "outlook" => oauth::Provider::Outlook,
                _ => return Err("Unknown OAuth provider".to_string()),
            };

            match oauth::refresh_access_token(provider, refresh_token.unwrap().to_string()).await {
                Ok(new_tokens) => {
                    creds["access_token"] = serde_json::Value::String(new_tokens.access_token);
                    if let Some(rt) = new_tokens.refresh_token {
                        creds["refresh_token"] = serde_json::Value::String(rt);
                    }
                    creds["expires_in"] = serde_json::Number::from(new_tokens.expires_in).into();

                    let creds_path: String = {
                        let conn = self.conn.lock().unwrap();
                        let mut stmt = conn
                            .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
                            .map_err(|e| e.to_string())?;
                        stmt.query_row([account_id], |row| row.get::<_, String>(0))
                            .map_err(|e| e.to_string())
                    }?;

                    let creds_json = creds.to_string();
                    let security = self.security.lock().unwrap();
                    let encrypted = security
                        .encrypt(creds_json.as_bytes())
                        .map_err(|e| e.to_string())?;
                    std::fs::write(&creds_path, encrypted).map_err(|e| e.to_string())?;
                }
                Err(e) => {
                    return Err(format!("OAuth refresh failed: {}", e));
                }
            }
        }

        Ok(())
    }

    pub async fn connect_imap(
        &self,
        account_id: &str,
    ) -> Result<Session<TlsStream<TcpStream>>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT imap_host, imap_port, imap_tls, auth_type, email FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let (host, port, _tls, auth_type, email): (String, u16, bool, String, String) = stmt
            .query_row([account_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, i64>(1)? as u16,
                    row.get::<_, i64>(2)? != 0,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        drop(stmt);
        drop(conn);

        let creds_json = self.get_credentials(account_id)?;
        let mut creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| e.to_string())?;
        self.refresh_oauth_if_needed(account_id, &mut creds).await?;

        let tcp_stream = async_std::net::TcpStream::connect((host.as_str(), port))
            .await
            .map_err(|e| e.to_string())?;
        let tls_connector = TlsConnector::new();
        let tls_stream = tls_connector
            .connect(&host, tcp_stream)
            .await
            .map_err(|e| e.to_string())?;
        let client = Client::new(tls_stream);

        let session = if auth_type == "oauth2" {
            let access_token = creds["access_token"].as_str().ok_or("No access_token")?;
            let username = &email;
            match client.login(username, access_token).await {
                Ok(session) => session,
                Err((e, _)) => return Err(format!("Login failed: {:?}", e)),
            }
        } else {
            let username = creds["username"].as_str().ok_or("No username")?;
            let password = creds["password"].as_str().ok_or("No password")?;
            match client.login(username, password).await {
                Ok(session) => session,
                Err((e, _)) => return Err(format!("Login failed: {:?}", e)),
            }
        };

        Ok(session)
    }
}
