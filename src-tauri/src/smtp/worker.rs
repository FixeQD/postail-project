use std::fs;
use std::thread;
use std::time::Duration;
use tokio::runtime::Builder;
use tracing;

use tauri::Emitter;

use crate::db::update_outbox_status;
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
        tracing::info!(target: "postail", "[Outbox] Worker started");

        loop {
            let running = *OUTBOX_WORKER_RUNNING.lock().unwrap();
            if !running {
                tracing::info!(target: "postail", "[Outbox] Worker stopping");
                break;
            }

            match self.process_pending_outbox().await {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!(target: "postail", "[Outbox] Error processing outbox: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(WORKER_INTERVAL_SECS)).await;
        }
    }

    /// Processes pending outbox messages from the database and attempts delivery.
    ///
    /// This scans the outbox for items with status `PENDING` or `RETRY` whose `next_retry` has passed, reads each item's `id`, `account_id`, and `raw_eml_path`, and attempts to deliver the message. On successful delivery the item is marked `SENT`; on failure the item's error, attempt count, and retry scheduling are updated (or it is marked `FAILED` when attempts are exhausted). Database-not-initialized state is treated as a no-op.
    ///
    /// # Returns
    ///
    /// `Ok(())` when processing completes without a database-level error; `Err(String)` if a database operation (query or update) fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::smtp::SmtpManager;
    /// # async fn example(manager: &SmtpManager) {
    /// manager.process_pending_outbox().await.unwrap();
    /// # }
    /// ```
    async fn process_pending_outbox(&self) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp();

        let pending_items = {
            let conn_guard = self.conn.lock().unwrap();
            if let Some(conn) = conn_guard.as_ref() {
                let mut stmt = conn
                    .prepare("SELECT id, account_id, raw_eml_path FROM outbox WHERE status = 'PENDING' OR (status = 'RETRY' AND next_retry < ?)")
                    .map_err(|e| e.to_string())?;
                let items: Vec<(String, String, String)> = stmt
                    .query_map([now], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .collect();
                items
            } else {
                return Ok(()); // DB not ready
            }
        };

        for (outbox_id, account_id, eml_path) in pending_items {
            match self
                .process_single_message(&account_id, &eml_path, &outbox_id)
                .await
            {
                Ok(()) => {
                    self.mark_outbox_sent(&outbox_id, &account_id)
                        .map_err(|e| e.to_string())?;
                    tracing::info!(target: "postail", "[Outbox] Successfully sent message {}", outbox_id);
                }
                Err(e) => {
                    tracing::error!(target: "postail", "[Outbox] Failed to send message {}: {}", outbox_id, e);
                    self.update_outbox_error(&outbox_id, &account_id, &e)
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
    }

    /// Processes a single outbox message: decrypts the stored EML, marks the item as processing, emits a processing event, and sends the email for the given account.
    ///
    /// The function returns `Ok(())` when the message was successfully sent; on failure it returns an `Err(String)` describing the error (for example I/O, decryption, database, or send errors).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Run inside an async context; this example shows the call pattern.
    /// // let manager: SmtpManager = /* obtain or construct manager */ ;
    /// // tokio::spawn(async move {
    /// //     manager.process_single_message("account-id", "/path/to/message.eml", "outbox-id").await.unwrap();
    /// // }).await;
    /// ```
    async fn process_single_message(
        &self,
        account_id: &str,
        eml_path: &str,
        outbox_id: &str,
    ) -> Result<(), String> {
        let eml_content = {
            let security = self.security.lock().unwrap();
            let encrypted = fs::read(eml_path).map_err(|e| e.to_string())?;

            security.decrypt(&encrypted).map_err(|e| e.to_string())?
        };

        {
            let conn_guard = self.conn.lock().unwrap();
            let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
            update_outbox_status(conn, outbox_id, "PROCESSING", None).map_err(|e| e.to_string())?;
        }

        self.emit_outbox_event("outbox:message:processing", outbox_id, account_id, None);

        self.send_email(account_id, &eml_content).await?;

        Ok(())
    }

    /// Mark an outbox record as sent and emit an "outbox:message:sent" event.
    ///
    /// Updates the outbox row identified by `outbox_id` to have status `SENT`, then emits an event
    /// containing the `outbox_id` and `account_id`.
    ///
    /// `outbox_id` is the database identifier of the outbox item. `account_id` is included in the
    /// emitted event payload to identify which account the message belonged to.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; `Err(String)` if the database is not initialized or the update fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Mark the item as sent and emit the event
    /// let _ = manager.mark_outbox_sent("outbox-123", "account-xyz").unwrap();
    /// ```
    fn mark_outbox_sent(&self, outbox_id: &str, account_id: &str) -> Result<(), String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        conn.execute(
            "UPDATE outbox SET status = 'SENT' WHERE id = ?",
            [outbox_id],
        )
        .map_err(|e| e.to_string())?;

        self.emit_outbox_event("outbox:message:sent", outbox_id, account_id, None);
        Ok(())
    }

    /// Updates an outbox entry after a delivery error, incrementing attempt count, setting RETRY or FAILED status, and emitting the corresponding outbox event.
    ///
    /// On success this updates the outbox row's `last_error`, `attempts`, and `next_retry` (or sets `next_retry` to NULL for permanent failures) and emits either `outbox:message:retry` (with `nextRetry`) or `outbox:message:failed` (with final attempt count).
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(String)` if the database is not initialized or a database operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Illustrative usage; assumes `manager` is an initialized SmtpManager.
    /// // let result = manager.update_outbox_error("outbox-id", "account-id", "SMTP timeout");
    /// // result.unwrap();
    /// ```
    fn update_outbox_error(
        &self,
        outbox_id: &str,
        account_id: &str,
        error: &str,
    ) -> Result<(), String> {
        use crate::db::calculate_backoff;

        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        let attempts: u32 = conn
            .query_row(
                "SELECT attempts FROM outbox WHERE id = ?",
                [outbox_id],
                |row| row.get::<_, i64>(0).map(|a| a as u32),
            )
            .map_err(|e| e.to_string())?;

        let new_attempts = attempts + 1;
        const MAX_ATTEMPTS: u32 = 5;
        let is_permanent_failure = new_attempts >= MAX_ATTEMPTS;

        if is_permanent_failure {
            conn.execute(
                "UPDATE outbox SET status = 'FAILED', last_error = ?, attempts = ?, next_retry = NULL WHERE id = ?",
                params![error, new_attempts.to_string(), outbox_id],
            )
            .map_err(|e| e.to_string())?;

            let details = serde_json::json!({
                "error": error,
                "attempts": new_attempts,
            });
            self.emit_outbox_event(
                "outbox:message:failed",
                outbox_id,
                account_id,
                Some(details),
            );
        } else {
            let next_retry = calculate_backoff(new_attempts);
            conn.execute(
                "UPDATE outbox SET status = 'RETRY', last_error = ?, attempts = ?, next_retry = ? WHERE id = ?",
                params![error, new_attempts.to_string(), next_retry.to_string(), outbox_id],
            )
            .map_err(|e| e.to_string())?;

            let details = serde_json::json!({
                "error": error,
                "attempts": new_attempts,
                "nextRetry": next_retry,
            });
            self.emit_outbox_event("outbox:message:retry", outbox_id, account_id, Some(details));
        }

        Ok(())
    }

    /// Refreshes an account's OAuth2 SMTP credentials if the access token is expired or will expire soon.
    ///
    /// Checks `creds.auth_type` and, when it equals `"oauth2"` and the token expiry is within 300 seconds,
    /// uses the stored `refresh_token` to request new tokens from the configured provider. On success this
    /// updates `creds` (replacing `access_token`, optionally `refresh_token`, and updating `expires_at`)
    /// and writes the encrypted credentials blob back to the account's `creds_blob_path` in the database.
    ///
    /// This function is a no-op when `auth_type` is not `"oauth2"` or when the token does not need refresh.
    ///
    /// # Errors
    ///
    /// Returns an `Err(String)` if a refresh is required but fails (including unknown provider or refresh API
    /// errors), if no refresh token is available when one is required, or if persisting the encrypted credentials
    /// to disk or the database lookup fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(manager: &crate::smtp::SmtpManager) -> Result<(), String> {
    /// let mut creds = serde_json::json!({
    ///     "auth_type": "oauth2",
    ///     "access_token": "old",
    ///     "refresh_token": "refresh-token",
    ///     "expires_at": 0i64,
    ///     "provider_type": "generic"
    /// });
    ///
    /// manager.refresh_oauth_smtp("account-id", &mut creds).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) async fn refresh_oauth_smtp(
        &self,
        account_id: &str,
        creds: &mut serde_json::Value,
    ) -> Result<(), String> {
        use chrono::Utc;

        let auth_type = creds
            .get("auth_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if auth_type != "oauth2" {
            return Ok(());
        }

        let expires_at = creds
            .get("expires_at")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let refresh_token = creds.get("refresh_token").and_then(|v| v.as_str());
        let provider_type = creds
            .get("provider_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        let now = Utc::now().timestamp();
        let seconds_until_expiry = expires_at.saturating_sub(now);

        if seconds_until_expiry < 300 {
            tracing::info!(target: "postail", "[OAuth] Token expired or expiring soon, refreshing...");
            if let Some(refresh_token) = refresh_token {
                let provider_kind =
                    ProviderKind::parse(provider_type).ok_or("Unknown OAuth provider")?;
                let provider = oauth::Provider::from_kind(provider_kind);

                match oauth::refresh_access_token(provider, refresh_token.to_string()).await {
                    Ok(new_tokens) => {
                        tracing::info!(target: "postail", "[OAuth] Token refreshed successfully, new expires_in: {}", new_tokens.expires_in);
                        creds["access_token"] = serde_json::Value::String(new_tokens.access_token);
                        if let Some(rt) = new_tokens.refresh_token {
                            creds["refresh_token"] = serde_json::Value::String(rt);
                        }
                        creds["expires_at"] = serde_json::Value::Number(serde_json::Number::from(
                            Utc::now().timestamp() + new_tokens.expires_in as i64,
                        ));

                        let creds_path: String = {
                            let conn_guard = self.conn.lock().unwrap();
                            let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
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
            } else {
                return Err("Token expired and no refresh token available".to_string());
            }
        }

        Ok(())
    }

    /// Emits an outbox-related event to the application handle if one is registered.
    ///
    /// The event payload includes `outboxId`, `accountId`, and an optional `details` object.
    /// If no application handle is available, the call is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// // Sends a "sent" event for an outbox item with no additional details
    /// manager.emit_outbox_event("outbox:message:sent", "outbox-123", "account-abc", None);
    ///
    /// // Sends a retry event with extra detail
    /// let details = serde_json::json!({ "error": "temporary failure", "attempts": 2 });
    /// manager.emit_outbox_event("outbox:message:retry", "outbox-456", "account-xyz", Some(details));
    /// ```
    fn emit_outbox_event(
        &self,
        event_name: &str,
        outbox_id: &str,
        account_id: &str,
        details: Option<serde_json::Value>,
    ) {
        let guard = self.app_handle.lock().unwrap();
        if let Some(ref handle) = *guard {
            let payload = serde_json::json!({
                "outboxId": outbox_id,
                "accountId": account_id,
                "details": details,
            });
            let _ = handle.emit(event_name, payload);
        }
    }
}