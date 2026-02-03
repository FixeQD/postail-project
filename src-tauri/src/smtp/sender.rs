// Bro why this protocol have to be so shitty 😭

use async_smtp::authentication::{Credentials, Mechanism};
use async_smtp::error::Error;
use async_smtp::extension::ClientId;
use async_smtp::{EmailAddress, Envelope, SmtpClient, SmtpTransport};
use mailparse::MailHeaderMap;
use std::io;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector as NativeTlsConnector;
use tokio_native_tls::TlsConnector;

use crate::smtp::EncryptionType;

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
    /// Retrieve and decrypt the stored credentials JSON for the given account.
    ///
    /// This reads the credentials blob path for `account_id` from the database, loads the
    /// encrypted blob from the filesystem, decrypts it using the manager's security service,
    /// and returns the resulting JSON string.
    ///
    /// # Returns
    ///
    /// `Ok` with the credentials JSON string on success, `Err` with a string describing the
    /// failure (e.g., database not initialized, SQL error, file I/O error, decryption failure,
    /// or invalid UTF-8).
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `manager` is an initialized SmtpManager and an account with id "acct1" exists.
    /// let creds_json = manager.get_credentials("acct1").expect("failed to load credentials");
    /// assert!(creds_json.trim_start().starts_with('{'));
    /// ```
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

    /// Produce an owned byte vector containing the EML content prepared for outgoing delivery.
    ///
    /// This function currently returns a direct copy of the provided raw EML bytes and exists as
    /// the extension point for any future outgoing EML processing (sanitization, header adjustments,
    /// etc.).
    ///
    /// # Examples
    ///
    /// ```
    /// // `mgr` is an instance of the SMTP manager type that provides this method.
    /// let raw = b"From: alice@example.com\r\nTo: bob@example.com\r\n\r\nHello";
    /// let processed = mgr.process_outgoing_eml(raw).unwrap();
    /// assert_eq!(processed, raw);
    /// ```
    pub(crate) fn process_outgoing_eml(&self, raw_eml: &[u8]) -> Result<Vec<u8>, String> {
        Ok(raw_eml.to_vec())
    }

    /// Sends an EML payload using the SMTP configuration for the given account.
    ///
    /// This looks up the account's SMTP settings and credentials, optionally refreshes an OAuth2
    /// SMTP token, parses the provided raw EML to build an SMTP envelope, selects the appropriate
    /// connection mode (TLS, STARTTLS, or plain), authenticates with the SMTP server, and delivers
    /// the message.
    ///
    /// # Parameters
    ///
    /// - `account_id`: Identifier of the account whose SMTP settings and credentials should be used.
    /// - `eml_content`: Raw EML bytes to be delivered.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message was accepted by the remote SMTP server, `Err(String)` with a
    /// diagnostic message on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(manager: &crate::smtp::SmtpManager) -> Result<(), String> {
    /// let account_id = "account-123";
    /// let eml = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hi\r\n\r\nHello";
    /// manager.send_email(account_id, eml).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_email(&self, account_id: &str, eml_content: &[u8]) -> Result<(), String> {
        // Extract account data in a separate scope to ensure lock is dropped before await
        let (host, port, tls_enabled, auth_type, account_email) = {
            let conn_guard = self.conn.lock().unwrap();
            let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
            let mut stmt = conn
                .prepare("SELECT smtp_host, smtp_port, smtp_tls, auth_type, email FROM accounts WHERE id = ?")
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
            // Port 587 or others use STARTTLS
            EncryptionType::StartTls
        };

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

        let processed_eml = self.process_outgoing_eml(eml_content)?;
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

        tracing::info!(target: "postail", "[SMTP] Connecting to {}:{}", host, port);
        tracing::info!(target: "postail", "[SMTP] Encryption: {:?}", encryption);
        tracing::info!(target: "postail", "[SMTP] From: {}, To: {}", from_str, to_str);

        let from_addr = EmailAddress::new(from_str.clone())
            .map_err(|e| format!("Invalid from address: {}", e))?;
        let to_addr =
            EmailAddress::new(to_str.clone()).map_err(|e| format!("Invalid to address: {}", e))?;
        let envelope = Envelope::new(Some(from_addr), vec![to_addr])
            .map_err(|e| format!("Failed to create envelope: {}", e))?;

        let email = async_smtp::SendableEmail::new(envelope, processed_eml);

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
                tracing::warn!(target: "postail", "[SMTP] Using plain connection on port {} (insecure!)", port);
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

    /// Sends a prepared email over an implicit TLS SMTP connection (typically port 465).
    ///
    /// Establishes a TCP connection, upgrades it to TLS, authenticates using the provided
    /// credentials and mechanism, sends the email, and then closes the SMTP session.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::test]
    /// # async fn send_with_tls_example() {
    /// // Construct a SmtpSendConfig named `config` with a ready-to-send `SendableEmail`.
    /// // let config = SmtpSendConfig { .. };
    /// // let manager = SmtpManager::new(...);
    /// // The call below performs the send and will return an Error on failure.
    /// // manager.send_with_tls(config).await.unwrap();
    /// # }
    /// ```
    async fn send_with_tls(&self, config: SmtpSendConfig<'_>) -> Result<(), Error> {
        // Connect with TLS on port 465
        let tcp_stream = TcpStream::connect((config.host, config.port))
            .await
            .map_err(|e| Error::Io(io::Error::other(format!("TCP connection failed: {}", e))))?;

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

        match send_result {
            Ok(_) => {
                tracing::info!(target: "postail", "[SMTP] Successfully sent email via {}:{} (TLS)", config.host, config.port);
                Ok(())
            }
            Err(e) => {
                tracing::error!(target: "postail", "[SMTP] Failed to send email via {}:{} (TLS): {}", config.host, config.port, e);
                Err(e)
            }
        }
    }

    /// Sends the provided email over an SMTP connection upgraded to TLS using STARTTLS.
    ///
    /// Performs the STARTTLS upgrade, authenticates with the configured mechanism and credentials, transmits the message, and then closes the SMTP session.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(manager: &crate::smtp::SmtpManager, cfg: crate::smtp::SmtpSendConfig<'_>) {
    /// manager.send_with_starttls(cfg).await.unwrap();
    /// # }
    /// ```
    ///
    /// Returns `Ok(())` on success, `Err(Error)` on failure.
    async fn send_with_starttls(&self, config: SmtpSendConfig<'_>) -> Result<(), Error> {
        let tcp_stream = TcpStream::connect((config.host, config.port))
            .await
            .map_err(|e| Error::Io(io::Error::other(format!("TCP connection failed: {}", e))))?;

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

        match send_result {
            Ok(_) => {
                tracing::info!(target: "postail", "[SMTP] Successfully sent email via {}:{} (STARTTLS)", config.host, config.port);
                Ok(())
            }
            Err(e) => {
                tracing::error!(target: "postail", "[SMTP] Failed to send email via {}:{} (STARTTLS): {}", config.host, config.port, e);
                Err(e)
            }
        }
    }

    /// Sends the prepared email over an unencrypted TCP connection.
    ///
    /// Connects to the SMTP server at `config.host:config.port`, performs SMTP authentication according to `config.auth_type` using credentials in `config.creds`, transmits `config.email`, and issues a QUIT on the transport.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the TCP connection, authentication, message transmission, or transport shutdown fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_smtp::Error;
    /// # use crate::smtp::{SmtpManager, SmtpSendConfig};
    /// # async fn example(manager: &SmtpManager, config: SmtpSendConfig<'_>) -> Result<(), Error> {
    /// manager.send_plain(config).await?;
    /// # Ok::<(), Error>(())
    /// # }
    /// ```
    async fn send_plain(&self, config: SmtpSendConfig<'_>) -> Result<(), Error> {
        let tcp_stream = TcpStream::connect((config.host, config.port))
            .await
            .map_err(|e| Error::Io(io::Error::other(format!("TCP connection failed: {}", e))))?;

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

        tracing::info!(target: "postail", "[SMTP] Successfully sent email via {}:{} (plain)", config.host, config.port);
        Ok(())
    }

    /// Authenticate an SMTP transport using either XOAUTH2 (when `auth_type` is `"oauth2"`) or PLAIN credentials.
    ///
    /// If `auth_type` equals `"oauth2"`, the function uses the `access_token` field from `creds` and the provided
    /// `account_email` as the username with the XOAUTH2 mechanism. Otherwise it uses `username` and `password` from
    /// `creds` with the PLAIN mechanism.
    ///
    /// # Errors
    ///
    /// Returns an error if required credential fields are missing or if the transport login fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_smtp::SmtpTransport;
    /// # use serde_json::json;
    /// # async fn example(mut transport: SmtpTransport<impl tokio::io::AsyncBufRead + tokio::io::AsyncWrite + Unpin>) -> Result<(), async_smtp::Error> {
    /// let creds = json!({"access_token": "ya29.ABCDE..."}); // for XOAUTH2
    /// // manager.authenticate(&mut transport, "oauth2", "me@example.com", &creds).await?;
    /// # Ok(())
    /// # }
    /// ```
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