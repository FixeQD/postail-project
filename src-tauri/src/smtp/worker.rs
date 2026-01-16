use std::fs;
use std::thread;
use std::time::Duration;
use tokio::runtime::Builder;

use crate::db::update_outbox_status;
use crate::error::DBError;
use crate::oauth;
use crate::oauth::ProviderKind;
use crate::smtp::SmtpManager;
use rusqlite::params;

const WORKER_INTERVAL_SECS: u64 = 10;

lazy_static::lazy_static! {
    static ref OUTBOX_WORKER_RUNNING: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
    static ref OUTBOX_WORKER_HANDLE: std::sync::Mutex<Option<thread::JoinHandle<()>>> = std::sync::Mutex::new(None);
}

impl SmtpManager {
    pub fn start_outbox_worker(&self) {
        let mut running = OUTBOX_WORKER_RUNNING.lock().unwrap();
        if *running {
            return;
        }
        *running = true;

        let manager = self.clone();
        let handle = thread::Builder::new()
            .name("outbox-worker".to_string())
            .spawn(move || {
                let rt = Builder::new_current_thread().enable_all().build().unwrap();

                rt.block_on(async {
                    manager.outbox_worker_loop().await;
                });
            })
            .expect("Failed to spawn outbox worker thread");

        let mut outbox_handle = OUTBOX_WORKER_HANDLE.lock().unwrap();
        *outbox_handle = Some(handle);
    }

    pub fn stop_outbox_worker(&self) {
        let mut running = OUTBOX_WORKER_RUNNING.lock().unwrap();
        *running = false;

        let mut handle = OUTBOX_WORKER_HANDLE.lock().unwrap();
        if let Some(h) = handle.take() {
            let _ = h.join();
        }
    }

    async fn outbox_worker_loop(&self) {
        eprintln!("[Outbox] Worker started");

        loop {
            let running = *OUTBOX_WORKER_RUNNING.lock().unwrap();
            if !running {
                eprintln!("[Outbox] Worker stopping");
                break;
            }

            match self.process_pending_outbox().await {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("[Outbox] Error processing outbox: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(WORKER_INTERVAL_SECS)).await;
        }
    }

    async fn process_pending_outbox(&self) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp();

        let pending_items = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, account_id, raw_eml_path FROM outbox WHERE status = 'PENDING' OR (status = 'RETRY' AND next_retry < ?)")
                .map_err(|e| e.to_string())?;
            let items: Vec<(String, String, String)> = stmt
                .query_map([now], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            items
        };

        for (outbox_id, account_id, eml_path) in pending_items {
            match self.process_single_message(&account_id, &eml_path).await {
                Ok(()) => {
                    self.mark_outbox_sent(&outbox_id)
                        .map_err(|e| e.to_string())?;
                    eprintln!("[Outbox] Successfully sent message {}", outbox_id);
                }
                Err(e) => {
                    eprintln!("[Outbox] Failed to send message {}: {}", outbox_id, e);
                    self.update_outbox_error(&outbox_id, &e)
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
    }

    async fn process_single_message(&self, account_id: &str, eml_path: &str) -> Result<(), String> {
        let eml_content = {
            let security = self.security.lock().unwrap();
            let encrypted = fs::read(eml_path).map_err(|e| e.to_string())?;

            security.decrypt(&encrypted).map_err(|e| e.to_string())?
        };

        {
            let conn = self.conn.lock().unwrap();
            update_outbox_status(&conn, eml_path, "PROCESSING", None).map_err(|e| e.to_string())?;
        }

        self.send_email(account_id, &eml_content).await?;

        self.append_to_sent_folder(account_id, &eml_content).await?;

        Ok(())
    }

    fn mark_outbox_sent(&self, outbox_id: &str) -> Result<(), DBError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE outbox SET status = 'SENT' WHERE id = ?",
            [outbox_id],
        )?;
        Ok(())
    }

    fn update_outbox_error(&self, outbox_id: &str, error: &str) -> Result<(), DBError> {
        use crate::db::calculate_backoff;

        let conn = self.conn.lock().unwrap();
        let attempts: u32 = conn.query_row(
            "SELECT attempts FROM outbox WHERE id = ?",
            [outbox_id],
            |row| row.get::<_, i64>(0).map(|a| a as u32),
        )?;

        let next_retry = calculate_backoff(attempts + 1);
        conn.execute(
            "UPDATE outbox SET status = 'RETRY', last_error = ?, attempts = ?, next_retry = ? WHERE id = ?",
            params![error, (attempts + 1).to_string(), next_retry.to_string(), outbox_id],
        )?;
        Ok(())
    }

    async fn append_to_sent_folder(
        &self,
        account_id: &str,
        eml_content: &[u8],
    ) -> Result<(), String> {
        let sent_folder = self.get_sent_folder_name(account_id).await?;

        let mut session = self.connect_imap_for_sent(account_id).await?;

        let append_result = session.append(sent_folder, eml_content.to_vec()).await;

        match append_result {
            Ok(_) => {
                session.logout().await.map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                eprintln!("[Outbox] Failed to append to Sent folder: {}", e);
                session.logout().await.ok();
                Ok(())
            }
        }
    }

    async fn get_sent_folder_name(&self, account_id: &str) -> Result<String, String> {
        let provider_type = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT provider_type FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            stmt.query_row([account_id], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())
        }?;

        let provider_kind = ProviderKind::parse(&provider_type).ok_or("Unknown provider")?;
        let info = oauth::ProviderInfo::get(provider_kind);

        Ok(info.sent_folder.to_string())
    }

    async fn connect_imap_for_sent(
        &self,
        account_id: &str,
    ) -> Result<async_imap::Session<async_native_tls::TlsStream<async_std::net::TcpStream>>, String>
    {
        use async_imap::Client;
        use async_native_tls::TlsConnector;
        use async_std::net::TcpStream;

        let (host, port, auth_type, email) = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT imap_host, imap_port, auth_type, email FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            stmt.query_row([account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? as u16,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
        };

        let creds_json = self.get_credentials(account_id)?;
        let mut creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| e.to_string())?;

        self.refresh_oauth_smtp(account_id, &mut creds).await?;

        let tcp_stream = TcpStream::connect((host.as_str(), port))
            .await
            .map_err(|e| e.to_string())?;
        let tls_connector = TlsConnector::new();
        let tls_stream = tls_connector
            .connect(&host, tcp_stream)
            .await
            .map_err(|e| e.to_string())?;
        let client = Client::new(tls_stream);

        let session = if auth_type == "oauth2" {
            let access_token = creds["access_token"]
                .as_str()
                .ok_or_else(|| "No access_token".to_string())?;
            let username = &email;
            match client.login(username, access_token).await {
                Ok(session) => session,
                Err((e, _)) => return Err(e.to_string()),
            }
        } else {
            let username = creds["username"]
                .as_str()
                .ok_or_else(|| "No username".to_string())?;
            let password = creds["password"]
                .as_str()
                .ok_or_else(|| "No password".to_string())?;
            match client.login(username, password).await {
                Ok(session) => session,
                Err((e, _)) => return Err(e.to_string()),
            }
        };

        Ok(session)
    }

    async fn refresh_oauth_smtp(
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

        if expires_in < 300 {
            if let Some(refresh_token) = refresh_token {
                let provider_kind =
                    ProviderKind::parse(provider_type).ok_or("Unknown OAuth provider")?;
                let provider = oauth::Provider::from_kind(provider_kind);

                match oauth::refresh_access_token(provider, refresh_token.to_string()).await {
                    Ok(new_tokens) => {
                        creds["access_token"] = serde_json::Value::String(new_tokens.access_token);
                        if let Some(rt) = new_tokens.refresh_token {
                            creds["refresh_token"] = serde_json::Value::String(rt);
                        }
                        creds["expires_in"] =
                            serde_json::Number::from(new_tokens.expires_in).into();

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
        }

        Ok(())
    }
}
