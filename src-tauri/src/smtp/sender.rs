use crate::security::SecurityManager;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rusqlite::Connection;
use serde_json;
use std::sync::{Arc, Mutex};

impl super::SmtpManager {
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

    pub async fn send_email(&self, account_id: &str, eml_content: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT smtp_host, smtp_port, smtp_tls, auth_type FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let (host, _port, _tls, auth_type): (String, u16, bool, String) = stmt
            .query_row([account_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, i64>(1)? as u16,
                    row.get::<_, i64>(2)? != 0,
                    row.get(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        drop(stmt);
        drop(conn);

        let creds_json = self.get_credentials(account_id)?;
        let creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| e.to_string())?;

        let username = if auth_type == "oauth2" {
            creds["email"].as_str().ok_or("No email for OAuth")?
        } else {
            creds["username"].as_str().ok_or("No username")?
        };
        let password = if auth_type == "oauth2" {
            creds["access_token"].as_str().ok_or("No access_token")?
        } else {
            creds["password"].as_str().ok_or("No password")?
        };

        let creds_smtp = Credentials::new(username.to_string(), password.to_string());

        let mailer = SmtpTransport::relay(&host)
            .map_err(|e| e.to_string())?
            .credentials(creds_smtp)
            .build();

        let message = Message::builder()
            .body(eml_content.to_vec())
            .map_err(|e| e.to_string())?;

        mailer.send(&message).map_err(|e| e.to_string())?;
        Ok(())
    }
}
