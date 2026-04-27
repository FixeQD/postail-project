// Bro why this protocol have to be so shitty 😭

use async_smtp::authentication::{Credentials, Mechanism};
use async_smtp::error::Error;
use async_smtp::extension::ClientId;
use async_smtp::{EmailAddress, Envelope, SmtpClient, SmtpTransport};
use mailparse::MailHeaderMap;
use std::io;
use std::time::Duration;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_native_tls::native_tls::TlsConnector as NativeTlsConnector;

use crate::globals::get_db_pool;
use crate::smtp::EncryptionType;

/// TCP connect + TLS handshake timeout.
const CONNECT_TIMEOUT_SECS: u64 = 30;

struct SmtpSendConfig<'a> {
    host: &'a str,
    port: u16,
    client: SmtpClient,
    auth_type: &'a str,
    account_email: &'a str,
    creds: &'a serde_json::Value,
    email: async_smtp::SendableEmail,
}

impl super::SmtpManager {
    pub(crate) async fn get_credentials(&self, account_id: &str) -> Result<String, String> {
        let creds_path: String = {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            let path: String = stmt
                .query_row([account_id], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            path
        };
        let creds_path = crate::db::resolve_creds_path(&creds_path);

        let security = self.security.lock().await;
        let encrypted = std::fs::read(&creds_path).map_err(|e| e.to_string())?;
        let decrypted = security.decrypt(&encrypted).map_err(|e| e.to_string())?;
        let creds_json = String::from_utf8(decrypted).map_err(|e| e.to_string())?;
        Ok(creds_json)
    }

    pub async fn send_email(&self, account_id: &str, eml_content: &[u8]) -> Result<(), String> {
        let (host, port, tls_enabled, auth_type, account_email) = {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            let mut stmt = conn
                .prepare(
                    "SELECT smtp_host, smtp_port, smtp_tls, auth_type, email \
                     FROM accounts WHERE id = ?",
                )
                .map_err(|e| e.to_string())?;
            let result: (String, u16, bool, String, String) = stmt
                .query_row([account_id], |row| {
                    Ok((
                        row.get(0)?,
                        row.get::<_, i64>(1)? as u16,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            result
        };

        let encryption = if !tls_enabled {
            EncryptionType::Plain
        } else if port == 465 {
            EncryptionType::Tls
        } else {
            EncryptionType::StartTls
        };

        let creds_json = self.get_credentials(account_id).await?;
        let mut creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| e.to_string())?;

        if auth_type == "oauth2" {
            tracing::info!(target: "postail", "[SMTP] OAuth2 detected, refreshing token if needed");
            if let Err(e) = self.refresh_oauth_smtp(account_id, &mut creds).await {
                tracing::error!(target: "postail", "[SMTP] Failed to refresh OAuth token: {}", e);
                return Err(format!("OAuth token refresh failed: {}", e));
            }
        }

        let mail = mailparse::parse_mail(eml_content)
            .map_err(|e| format!("Failed to parse email: {}", e))?;

        let from_str = mail
            .headers
            .get_first_header("From")
            .map(|h| h.get_value())
            .ok_or("Missing From header")?;

        let all_recipients = collect_envelope_recipients(&mail);
        if all_recipients.is_empty() {
            return Err("No valid recipients found in To/Cc/Bcc headers".to_string());
        }

        tracing::info!(
            target: "postail",
            "[SMTP] Connecting to {}:{} | From: {} | Recipients: {}",
            host, port, from_str, all_recipients.len()
        );
        tracing::info!(target: "postail", "[SMTP] Encryption: {:?}", encryption);

        let from_addr =
            EmailAddress::new(from_str).map_err(|e| format!("Invalid from address: {}", e))?;

        let envelope = Envelope::new(Some(from_addr), all_recipients)
            .map_err(|e| format!("Failed to create envelope: {}", e))?;

        let email = async_smtp::SendableEmail::new(envelope, eml_content.to_vec());
        let client = SmtpClient::new().hello_name(ClientId::new("postail".to_string()));

        let config = SmtpSendConfig {
            host: &host,
            port,
            client,
            auth_type: &auth_type,
            account_email: &account_email,
            creds: &creds,
            email,
        };

        match encryption {
            EncryptionType::Tls => {
                tracing::info!(target: "postail", "[SMTP] Using TLS on port {}", port);
                self.send_with_tls(config).await
            }
            EncryptionType::StartTls => {
                tracing::info!(target: "postail", "[SMTP] Using STARTTLS on port {}", port);
                self.send_with_starttls(config).await
            }
            EncryptionType::Plain => {
                tracing::warn!(
                    target: "postail",
                    "[SMTP] Using plain connection on port {} (insecure!)",
                    port
                );
                self.send_plain(config).await
            }
        }
        .map_err(|e| {
            let error_msg = format!(
                "SMTP send failed: {} (host: {}:{}, encryption: {:?})",
                e, host, port, encryption
            );
            tracing::error!(target: "postail", "[SMTP] {}", error_msg);
            error_msg
        })
    }

    async fn send_with_tls(&self, config: SmtpSendConfig<'_>) -> Result<(), Error> {
        let tcp_stream = timed_connect(config.host, config.port).await?;

        let native_tls = NativeTlsConnector::builder()
            .build()
            .map_err(|e| Error::Io(io::Error::other(format!("TLS builder failed: {}", e))))?;
        let tls_connector = TlsConnector::from(native_tls);

        let tls_stream = tls_connector
            .connect(config.host, tcp_stream)
            .await
            .map_err(|e| Error::Io(io::Error::other(format!("TLS handshake failed: {}", e))))?;

        let stream = BufStream::new(tls_stream);
        let mut transport = SmtpTransport::new(config.client, stream).await?;

        self.authenticate(
            &mut transport,
            config.auth_type,
            config.account_email,
            config.creds,
        )
        .await?;

        let send_result = transport.send(config.email).await;
        let _ = transport.quit().await;

        send_result.map(|_| {
            tracing::info!(
                target: "postail",
                "[SMTP] Successfully sent via {}:{} (TLS)",
                config.host, config.port
            );
        })
    }

    async fn send_with_starttls(&self, config: SmtpSendConfig<'_>) -> Result<(), Error> {
        let tcp_stream = timed_connect(config.host, config.port).await?;
        let stream = BufStream::new(tcp_stream);
        let transport = SmtpTransport::new(config.client, stream).await?;
        let plain_stream = transport.starttls().await?;

        let native_tls = NativeTlsConnector::builder()
            .build()
            .map_err(|e| Error::Io(io::Error::other(format!("TLS builder failed: {}", e))))?;
        let tls_connector = TlsConnector::from(native_tls);

        let tls_stream = tls_connector
            .connect(config.host, plain_stream)
            .await
            .map_err(|e| Error::Io(io::Error::other(format!("TLS handshake failed: {}", e))))?;

        let stream = BufStream::new(tls_stream);
        let client = SmtpClient::new()
            .hello_name(ClientId::new("postail".to_string()))
            .without_greeting();
        let mut transport = SmtpTransport::new(client, stream).await?;

        transport
            .get_mut()
            .ehlo(ClientId::new("postail".to_string()))
            .await?;

        self.authenticate(
            &mut transport,
            config.auth_type,
            config.account_email,
            config.creds,
        )
        .await?;

        let send_result = transport.send(config.email).await;
        let _ = transport.quit().await;

        send_result.map(|_| {
            tracing::info!(
                target: "postail",
                "[SMTP] Successfully sent via {}:{} (STARTTLS)",
                config.host, config.port
            );
        })
    }

    async fn send_plain(&self, config: SmtpSendConfig<'_>) -> Result<(), Error> {
        let tcp_stream = timed_connect(config.host, config.port).await?;
        let stream = BufStream::new(tcp_stream);
        let mut transport = SmtpTransport::new(config.client, stream).await?;

        self.authenticate(
            &mut transport,
            config.auth_type,
            config.account_email,
            config.creds,
        )
        .await?;

        transport.send(config.email).await?;
        let _ = transport.quit().await;

        tracing::info!(
            target: "postail",
            "[SMTP] Successfully sent via {}:{} (plain)",
            config.host, config.port
        );
        Ok(())
    }

    async fn authenticate<S>(
        &self,
        transport: &mut SmtpTransport<S>,
        auth_type: &str,
        account_email: &str,
        creds: &serde_json::Value,
    ) -> Result<(), Error>
    where
        S: tokio::io::AsyncBufRead + tokio::io::AsyncWrite + Unpin,
    {
        let (username, password, mechanism) = if auth_type == "oauth2" {
            let token = creds["access_token"]
                .as_str()
                .ok_or_else(|| Error::Io(io::Error::other("No access_token in credentials")))?;
            tracing::info!(target: "postail", "[SMTP] Using OAuth2 XOAUTH2 authentication");
            (
                account_email.to_string(),
                token.to_string(),
                Mechanism::Xoauth2,
            )
        } else {
            let username = creds["username"]
                .as_str()
                .ok_or_else(|| Error::Io(io::Error::other("No username in credentials")))?
                .to_string();
            let password = creds["password"]
                .as_str()
                .ok_or_else(|| Error::Io(io::Error::other("No password in credentials")))?
                .to_string();
            tracing::info!(target: "postail", "[SMTP] Using PLAIN authentication");
            (username, password, Mechanism::Plain)
        };

        let credentials = Credentials::new(username, password);
        transport.try_login(&credentials, &[mechanism]).await?;

        tracing::info!(target: "postail", "[SMTP] Authentication successful");
        Ok(())
    }
}

/// Parses all To, Cc, and Bcc headers and returns a deduplicated list of valid SMTP envelope recipients.
fn collect_envelope_recipients(mail: &mailparse::ParsedMail) -> Vec<EmailAddress> {
    let mut recipients: Vec<EmailAddress> = Vec::new();

    for header_name in &["To", "Cc", "Bcc"] {
        for header in mail.headers.get_all_headers(header_name) {
            let Ok(addrs) = mailparse::addrparse(&header.get_value()) else {
                continue;
            };
            for addr in addrs.iter() {
                match addr {
                    mailparse::MailAddr::Single(s) => {
                        if let Ok(ea) = EmailAddress::new(s.addr.clone()) {
                            if !recipients
                                .iter()
                                .any(|r| r.as_ref() as &str == ea.as_ref() as &str)
                            {
                                recipients.push(ea);
                            }
                        }
                    }
                    mailparse::MailAddr::Group(g) => {
                        for s in &g.addrs {
                            if let Ok(ea) = EmailAddress::new(s.addr.clone()) {
                                if !recipients
                                    .iter()
                                    .any(|r| r.as_ref() as &str == ea.as_ref() as &str)
                                {
                                    recipients.push(ea);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    recipients
}

/// Opens a TCP connection with a hard timeout so a non-responsive SMTP server can't block the worker task indefinitely.
async fn timed_connect(host: &str, port: u16) -> Result<TcpStream, Error> {
    tokio::time::timeout(
        Duration::from_secs(CONNECT_TIMEOUT_SECS),
        TcpStream::connect((host, port)),
    )
    .await
    .map_err(|_| {
        Error::Io(io::Error::other(format!(
            "TCP connection to {}:{} timed out after {}s",
            host, port, CONNECT_TIMEOUT_SECS
        )))
    })?
    .map_err(|e| Error::Io(io::Error::other(format!("TCP connection failed: {}", e))))
}
