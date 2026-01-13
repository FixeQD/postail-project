use crate::security::SecurityManager;
use async_imap::{Client, Session};
use async_native_tls::{TlsConnector, TlsStream};
use async_std::net::TcpStream;
use rusqlite::Connection;
use serde_json;
use std::sync::{Arc, Mutex};

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
        // TODO: Implement OAuth refresh logic
        // Check expires_at, refresh, update DB
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
        let (host, port, tls, auth_type, email): (String, u16, bool, String, String) = stmt
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
