use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};
use mailparse::MailHeaderMap;
use serde_json;
use std::time::Duration;

impl super::SmtpManager {
    pub(crate) fn get_credentials(&self, account_id: &str) -> Result<String, String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let mut stmt = conn
            .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let creds_path: String = stmt
            .query_row([account_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        drop(stmt);
        drop(conn_guard);

        let security = self.security.lock().unwrap();
        let encrypted = std::fs::read(&creds_path).map_err(|e| e.to_string())?;
        let decrypted = security.decrypt(&encrypted).map_err(|e| e.to_string())?;
        let creds_json = String::from_utf8(decrypted).map_err(|e| e.to_string())?;
        Ok(creds_json)
    }

    pub(crate) fn process_outgoing_eml(&self, raw_eml: &[u8]) -> Result<Vec<u8>, String> {
        Ok(raw_eml.to_vec())
    }

    pub async fn send_email(&self, account_id: &str, eml_content: &[u8]) -> Result<(), String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let mut stmt = conn
            .prepare("SELECT smtp_host, smtp_port, smtp_tls, auth_type, email FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let (host, _port, _tls, auth_type, account_email): (String, u16, bool, String, String) =
            stmt.query_row([account_id], |row| {
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
        drop(conn_guard);

        let creds_json = self.get_credentials(account_id)?;
        let mut creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| e.to_string())?;

        // Refresh OAuth token if needed before sending
        if auth_type == "oauth2" {
            tracing::info!(target: "postail", "[SMTP] OAuth2 detected, refreshing token if needed");
            if let Err(e) = self.refresh_oauth_smtp(account_id, &mut creds).await {
                tracing::error!(target: "postail", "[SMTP] Failed to refresh OAuth token: {}", e);
                return Err(format!("OAuth token refresh failed: {}", e));
            }
        }

        let username = if auth_type == "oauth2" {
            account_email.as_str()
        } else {
            creds["username"].as_str().ok_or("No username")?
        };
        let password = if auth_type == "oauth2" {
            let token = creds["access_token"].as_str().ok_or("No access_token")?;
            tracing::info!(target: "postail", "[SMTP] Using OAuth2 access_token: {}...", &token[..token.len().min(20)]);
            token
        } else {
            creds["password"].as_str().ok_or("No password")?
        };

        let creds_smtp = Credentials::new(username.to_string(), password.to_string());

        // Port 465 = SMTPS (TLS wrapper)
        // Port 587 = STARTTLS
        // Port 25 = Plain text or STARTTLS
        tracing::info!(target: "postail", "[SMTP] Connecting to {}:{}", host, _port);

        let mailer = match _port {
            465 => {
                // SMTPS - implicit TLS
                tracing::info!(target: "postail", "[SMTP] Using TLS (SMTPS) for port 465");
                SmtpTransport::relay(&host)
                    .map_err(|e| e.to_string())?
                    .port(_port)
                    .credentials(creds_smtp)
                    .timeout(Some(Duration::from_secs(30)))
                    .build()
            }
            587 => {
                // STARTTLS - explicit TLS
                tracing::info!(target: "postail", "[SMTP] Using STARTTLS for port 587");
                SmtpTransport::starttls_relay(&host)
                    .map_err(|e| format!("Failed to configure STARTTLS: {}", e))?
                    .port(_port)
                    .credentials(creds_smtp)
                    .timeout(Some(Duration::from_secs(30)))
                    .build()
            }
            _ => {
                // Fallback - try STARTTLS first, then plain
                tracing::info!(target: "postail", "[SMTP] Using STARTTLS for port {}", _port);
                SmtpTransport::starttls_relay(&host)
                    .map_err(|e| format!("Failed to configure STARTTLS: {}", e))?
                    .port(_port)
                    .credentials(creds_smtp)
                    .timeout(Some(Duration::from_secs(30)))
                    .build()
            }
        };

        let processed_eml = self.process_outgoing_eml(eml_content)?;

        // Debug: print first 500 bytes of email
        let preview = String::from_utf8_lossy(&processed_eml[..processed_eml.len().min(500)]);
        tracing::info!(target: "postail", "[SMTP] Email preview:\n{}", preview);

        // Parse the email to extract From/To for lettre
        let mail = mailparse::parse_mail(&processed_eml)
            .map_err(|e| format!("Failed to parse email: {}", e))?;

        let from_str = mail
            .headers
            .get_first_header("From")
            .map(|h| h.get_value())
            .ok_or("Missing From header")?;

        let to_str = mail
            .headers
            .get_first_header("To")
            .map(|h| h.get_value())
            .ok_or("Missing To header")?;

        let from_addr: Mailbox = from_str
            .parse()
            .map_err(|e| format!("Invalid From address '{}': {}", from_str, e))?;

        let to_addr: Mailbox = to_str
            .parse()
            .map_err(|e| format!("Invalid To address '{}': {}", to_str, e))?;

        let message = Message::builder()
            .from(from_addr)
            .to(to_addr)
            .body(processed_eml)
            .map_err(|e| e.to_string())?;

        match mailer.send(&message) {
            Ok(_) => {
                tracing::info!(target: "postail", "[SMTP] Successfully sent email via {}:{}", host, _port);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "SMTP send failed: {} (host: {}:{}, tls: {})",
                    e, host, _port, _tls
                );
                tracing::error!(target: "postail", "[SMTP] {}", error_msg);
                Err(error_msg)
            }
        }
    }
}
