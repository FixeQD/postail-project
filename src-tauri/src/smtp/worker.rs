use std::sync::LazyLock;
use std::time::Duration;
use tokio::task;
use tokio::task::JoinSet;
use tracing;

use tauri::Emitter;

use crate::db::update_outbox_status;
use crate::globals::get_db_pool;
use crate::oauth;
use crate::oauth::ProviderKind;
use crate::smtp::SmtpManager;
use rusqlite::params;

/// How long to wait between polling cycles when no notify fires
const WORKER_POLL_INTERVAL_SECS: u64 = 30;

/// Max number of messages sent in parallel per cycle.
const MAX_CONCURRENT_SENDS: usize = 4;

/// Signals the worker to wake up and process the queue immediately.
pub(crate) static OUTBOX_NOTIFY: LazyLock<std::sync::Arc<tokio::sync::Notify>> =
    LazyLock::new(|| std::sync::Arc::new(tokio::sync::Notify::new()));

static OUTBOX_WORKER_RUNNING: LazyLock<std::sync::Mutex<bool>> =
    LazyLock::new(|| std::sync::Mutex::new(false));
static OUTBOX_WORKER_HANDLE: LazyLock<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>> =
    LazyLock::new(|| std::sync::Mutex::new(None));

impl SmtpManager {
    pub fn start_outbox_worker(&self) {
        let mut running = OUTBOX_WORKER_RUNNING.lock().unwrap();
        if *running {
            return;
        }
        *running = true;

        let manager = self.clone();
        let handle = task::spawn(async move {
            manager.outbox_worker_loop().await;
        });

        let mut outbox_handle = OUTBOX_WORKER_HANDLE.lock().unwrap();
        *outbox_handle = Some(handle);
    }

    pub fn stop_outbox_worker(&self) {
        {
            let mut running = OUTBOX_WORKER_RUNNING.lock().unwrap();
            *running = false;
        }

        // Wake up the worker so it can see the running=false and exit cleanly, then abort as a hard stop
        OUTBOX_NOTIFY.notify_one();

        let handle = {
            let mut handle_guard = OUTBOX_WORKER_HANDLE.lock().unwrap();
            handle_guard.take()
        };

        if let Some(h) = handle {
            tracing::info!(target: "postail", "[Outbox] Aborting worker task...");
            h.abort();
            tracing::info!(target: "postail", "[Outbox] Worker task aborted");
        }
    }

    async fn outbox_worker_loop(&self) {
        tracing::info!(target: "postail", "[Outbox] Worker started");

        loop {
            if !*OUTBOX_WORKER_RUNNING.lock().unwrap() {
                tracing::info!(target: "postail", "[Outbox] Worker stopping");
                break;
            }

            match self.process_pending_outbox().await {
                Ok(0) => {}
                Ok(n) => tracing::info!(target: "postail", "[Outbox] Processed {} message(s)", n),
                Err(e) => {
                    tracing::error!(target: "postail", "[Outbox] Error processing outbox: {}", e);
                }
            }

            // Sleep until either a new message arrives or the poll interval fires.
            tokio::select! {
                _ = OUTBOX_NOTIFY.notified() => {
                    tracing::debug!(target: "postail", "[Outbox] Worker woken by notify");
                }
                _ = tokio::time::sleep(Duration::from_secs(WORKER_POLL_INTERVAL_SECS)) => {
                    tracing::debug!(target: "postail", "[Outbox] Worker poll interval fired");
                }
            }
        }

        tracing::info!(target: "postail", "[Outbox] Worker stopped");
    }

    async fn process_pending_outbox(&self) -> Result<usize, String> {
        let now = chrono::Utc::now().timestamp();

        let pending_items = {
            let pool = match get_db_pool().await {
                Ok(p) => p,
                Err(_) => return Ok(0),
            };
            let conn = match pool.get() {
                Ok(c) => c,
                Err(_) => return Ok(0),
            };
            let mut stmt = conn
                .prepare(
                    "SELECT id, account_id, raw_eml_path FROM outbox \
                     WHERE status = 'PENDING' OR (status = 'RETRY' AND next_retry < ?)",
                )
                .map_err(|e| e.to_string())?;
            let items: Vec<(String, String, String)> = stmt
                .query_map([now], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            items
        };

        if pending_items.is_empty() {
            return Ok(0);
        }

        let total = pending_items.len();
        tracing::info!(target: "postail", "[Outbox] Found {} pending message(s)", total);

        // Process in batches of MAX_CONCURRENT_SENDS so we don't hammer the SMTP server.
        let mut chunks = pending_items.chunks(MAX_CONCURRENT_SENDS).peekable();

        while let Some(batch) = chunks.next() {
            let mut join_set: JoinSet<(String, String, Result<(), String>)> = JoinSet::new();

            for (outbox_id, account_id, eml_path) in batch {
                let manager = self.clone();
                let outbox_id = outbox_id.clone();
                let account_id = account_id.clone();
                let eml_path = eml_path.clone();

                join_set.spawn(async move {
                    let result = manager
                        .process_single_message(&account_id, &eml_path, &outbox_id)
                        .await;
                    (outbox_id, account_id, result)
                });
            }

            while let Some(join_result) = join_set.join_next().await {
                match join_result {
                    Ok((outbox_id, account_id, Ok(()))) => {
                        if let Err(e) = self.mark_outbox_sent(&outbox_id, &account_id).await {
                            tracing::error!(
                                target: "postail",
                                "[Outbox] Failed to mark {} as sent: {}",
                                outbox_id, e
                            );
                        } else {
                            tracing::info!(
                                target: "postail",
                                "[Outbox] Successfully sent message {}",
                                outbox_id
                            );
                        }
                    }
                    Ok((outbox_id, account_id, Err(e))) => {
                        tracing::error!(
                            target: "postail",
                            "[Outbox] Failed to send message {}: {}",
                            outbox_id, e
                        );
                        if let Err(ue) = self.update_outbox_error(&outbox_id, &account_id, &e).await
                        {
                            tracing::error!(
                                target: "postail",
                                "[Outbox] Failed to record error for {}: {}",
                                outbox_id, ue
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(target: "postail", "[Outbox] Task panicked: {}", e);
                    }
                }
            }
        }

        Ok(total)
    }

    async fn process_single_message(
        &self,
        account_id: &str,
        eml_path: &str,
        outbox_id: &str,
    ) -> Result<(), String> {
        // Read the encrypted file BEFORE acquiring the security lock so concurrent tasks can do their I/O in parallel and only serialize on the crypto step
        let encrypted = tokio::fs::read(eml_path)
            .await
            .map_err(|e| format!("Failed to read EML file: {}", e))?;

        let eml_content = {
            let security = self.security.lock().await;
            security.decrypt(&encrypted).map_err(|e| e.to_string())?
        };

        {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            update_outbox_status(&*conn, outbox_id, "PROCESSING", None)
                .map_err(|e| e.to_string())?;
        }

        self.emit_outbox_event("outbox:message:processing", outbox_id, account_id, None)
            .await;

        self.send_email(account_id, &eml_content).await?;

        // Upsert contacts from To/Cc headers (respects the auto-create-contacts setting).
        self.maybe_upsert_contacts(&eml_content).await;

        Ok(())
    }

    /// Extracts To/Cc addresses from the sent mail and upserts them into contacts, but only if the `auto-create-contacts` setting is enabled
    async fn maybe_upsert_contacts(&self, eml_content: &[u8]) {
        if let Ok(Some(val)) =
            crate::db::account::settings::get_setting("auto-create-contacts").await
        {
            if val == "false" {
                return;
            }
        }

        let Ok(mail) = mailparse::parse_mail(eml_content) else {
            return;
        };
        let Ok(pool) = get_db_pool().await else {
            return;
        };
        let Ok(conn) = pool.get() else {
            return;
        };

        use mailparse::MailHeaderMap;
        let headers = mail.get_headers();
        let to_addrs = headers.get_all_values("To");
        let cc_addrs = headers.get_all_values("Cc");

        for value in to_addrs.into_iter().chain(cc_addrs.into_iter()) {
            if let Ok(parsed_addrs) = mailparse::addrparse(&value) {
                for addr in parsed_addrs.iter() {
                    match addr {
                        mailparse::MailAddr::Single(single) => {
                            let _ = crate::db::account::contacts::upsert_contact(
                                &conn,
                                &single.addr,
                                single.display_name.as_deref(),
                            );
                        }
                        mailparse::MailAddr::Group(group) => {
                            for single in &group.addrs {
                                let _ = crate::db::account::contacts::upsert_contact(
                                    &conn,
                                    &single.addr,
                                    single.display_name.as_deref(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    async fn mark_outbox_sent(&self, outbox_id: &str, account_id: &str) -> Result<(), String> {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE outbox SET status = 'SENT' WHERE id = ?",
            [outbox_id],
        )
        .map_err(|e| e.to_string())?;

        self.emit_outbox_event("outbox:message:sent", outbox_id, account_id, None)
            .await;
        Ok(())
    }

    async fn update_outbox_error(
        &self,
        outbox_id: &str,
        account_id: &str,
        error: &str,
    ) -> Result<(), String> {
        use crate::db::calculate_backoff;

        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
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
            )
            .await;
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
            self.emit_outbox_event("outbox:message:retry", outbox_id, account_id, Some(details))
                .await;
        }

        Ok(())
    }

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
                        tracing::info!(
                            target: "postail",
                            "[OAuth] Token refreshed successfully, new expires_in: {}",
                            new_tokens.expires_in
                        );
                        creds["access_token"] = serde_json::Value::String(new_tokens.access_token);
                        if let Some(rt) = new_tokens.refresh_token {
                            creds["refresh_token"] = serde_json::Value::String(rt);
                        }
                        creds["expires_at"] = serde_json::Value::Number(serde_json::Number::from(
                            Utc::now().timestamp() + new_tokens.expires_in as i64,
                        ));

                        let creds_path: String = {
                            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
                            let conn = pool.get().map_err(|e| e.to_string())?;
                            let mut stmt = conn
                                .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
                                .map_err(|e| e.to_string())?;
                            stmt.query_row([account_id], |row| row.get::<_, String>(0))
                                .map_err(|e| e.to_string())
                        }?;
                        let creds_path = crate::db::resolve_creds_path(&creds_path);

                        let creds_json = creds.to_string();
                        let security = self.security.lock().await;
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

    pub(crate) async fn emit_outbox_event(
        &self,
        event_name: &str,
        outbox_id: &str,
        account_id: &str,
        details: Option<serde_json::Value>,
    ) {
        let guard = self.app_handle.lock().await;
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
