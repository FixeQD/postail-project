use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Mutex;
use tokio::time::Duration;

use tracing;

use crate::db::accounts::get_account_email;
use crate::error::{AppError, ImapError};
use crate::globals::get_db_pool;
use crate::imap::sync_status::{
    SYNC_STATUS_MANAGER, mark_sync_complete, mark_sync_error, start_sync_status_tracking,
    update_sync_status,
};

const RFC_IDLE_TIMEOUT_SECS: u64 = 29 * 60;
const POLL_INTERVAL_SECS: u64 = 60;

// ── Stop / interrupt maps ─────────────────────────────────────────────────────

static STOP_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Interrupt handles for active IDLE waits (dropping StopSource cancels the IDLE).
static IDLE_INTERRUPTS: LazyLock<Mutex<HashMap<String, stop_token::StopSource>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Notify handles for active poll-loop sleeps.
static POLL_INTERRUPTS: LazyLock<Mutex<HashMap<String, Arc<tokio::sync::Notify>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static WATCH_STOP_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Interrupt any active IDLE wait for `account_id`.
async fn interrupt_idle(account_id: &str) {
    let mut interrupts = IDLE_INTERRUPTS.lock().await;
    if let Some(interrupt) = interrupts.remove(account_id) {
        tracing::info!(target: "postail", "[IMAP] Interrupting IDLE for {}", account_id);
        drop(interrupt);
    }
}

/// Wake any active poll-loop sleep for `account_id`.
async fn interrupt_poll(account_id: &str) {
    let interrupts = POLL_INTERRUPTS.lock().await;
    if let Some(notify) = interrupts.get(account_id) {
        notify.notify_one();
    }
}

// ── ImapManager impl ──────────────────────────────────────────────────────────

impl crate::imap::ImapManager {
    pub async fn start_sync(&self, account_id: &str) -> Result<(), AppError> {
        {
            let flags = STOP_FLAGS.lock().await;
            if let Some(flag) = flags.get(account_id) {
                if !flag.load(Ordering::Acquire) {
                    tracing::info!(target: "postail",
                        "[IMAP] start_sync called but already running for {}, ignoring",
                        account_id
                    );
                    return Ok(());
                }
            }
        }

        let account_email = {
            let pool = get_db_pool()
                .await
                .map_err(|e| AppError::from(e.to_string()))?;
            let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
            crate::db::accounts::get_account_email(&*conn, account_id)
                .map_err(|e| AppError::from(e.to_string()))?
                .unwrap_or_else(|| account_id.to_string())
        };

        start_sync_status_tracking(account_id, &account_email).await;

        let stop_flag = SYNC_STATUS_MANAGER.get_stop_flag(account_id).await;
        stop_flag.store(false, Ordering::SeqCst);
        {
            let mut flags = STOP_FLAGS.lock().await;
            flags.insert(account_id.to_string(), Arc::clone(&stop_flag));
        }

        let manager = self.clone();
        let account_id_str = account_id.to_string();

        tokio::spawn(async move {
            tracing::info!(target: "postail", "[IMAP] Sync started for {}", account_id_str);
            if let Err(e) = manager.start_sync_async(&account_id_str).await {
                tracing::error!(target: "postail", "[IMAP] start_sync_async failed: {}", e);
                mark_sync_error(&account_id_str, &e.to_string()).await;
            }
            tracing::info!(target: "postail", "[IMAP] Sync done for {}", account_id_str);
        });

        Ok(())
    }

    pub async fn stop_all_syncs(&self) -> Result<(), AppError> {
        let accounts: Vec<String> = {
            let flags = STOP_FLAGS.lock().await;
            flags.keys().cloned().collect()
        };
        for account_id in accounts {
            if let Err(e) = self.stop_sync(&account_id).await {
                tracing::error!(target: "postail",
                    "[IMAP] Failed to stop sync for {}: {}",
                    account_id, e
                );
            }
        }
        Ok(())
    }

    pub async fn force_idle_wakeup(&self, account_id: &str) {
        interrupt_idle(account_id).await;
        interrupt_poll(account_id).await;
    }

    pub async fn stop_sync(&self, account_id: &str) -> Result<(), AppError> {
        let stop_flag = {
            let flags = STOP_FLAGS.lock().await;
            flags
                .get(account_id)
                .cloned()
                .unwrap_or_else(|| Arc::new(AtomicBool::new(false)))
        };
        stop_flag.store(true, Ordering::SeqCst);

        interrupt_idle(account_id).await;
        interrupt_poll(account_id).await;

        tracing::info!(target: "postail", "[IMAP] Stop requested for {}", account_id);
        Ok(())
    }

    pub async fn start_watch_mailbox(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<(), AppError> {
        self.stop_watch_mailbox(account_id).await;

        let account_email = {
            let pool = get_db_pool()
                .await
                .map_err(|e| AppError::from(e.to_string()))?;
            let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
            get_account_email(&*conn, account_id)
                .map_err(|e| AppError::from(e.to_string()))?
                .unwrap_or_else(|| account_id.to_string())
        };
        SYNC_STATUS_MANAGER
            .register_account(account_id, &account_email)
            .await;
        tracing::info!(target: "postail", "[IMAP] Registered account {} for watch", account_id);

        let stop_flag = Arc::new(AtomicBool::new(false));
        {
            let mut flags = WATCH_STOP_FLAGS.lock().await;
            flags.insert(account_id.to_string(), stop_flag.clone());
        }

        let manager = self.clone();
        let account_id_owned = account_id.to_string();
        let mailbox_owned = mailbox_name.to_string();

        tokio::spawn(async move {
            tracing::info!(target: "postail",
                "[IMAP] Watch started for {}@{}",
                mailbox_owned, account_id_owned
            );
            match manager
                .idle_mailbox(&account_id_owned, &mailbox_owned, &stop_flag)
                .await
            {
                Ok(()) => tracing::info!(target: "postail",
                    "[IMAP] Watch ended cleanly for {}@{}",
                    mailbox_owned, account_id_owned
                ),
                Err(e) => tracing::error!(target: "postail",
                    "[IMAP] Watch error for {}@{}: {}",
                    mailbox_owned, account_id_owned, e
                ),
            }
        });

        tracing::info!(target: "postail",
            "[IMAP] Watch spawned for {}@{}",
            mailbox_name, account_id
        );
        Ok(())
    }

    pub async fn stop_watch_mailbox(&self, account_id: &str) {
        if let Some(f) = {
            let flags = WATCH_STOP_FLAGS.lock().await;
            flags.get(account_id).cloned()
        } {
            f.store(true, Ordering::SeqCst);
        }

        interrupt_idle(account_id).await;
        interrupt_poll(account_id).await;

        {
            let mut flags = WATCH_STOP_FLAGS.lock().await;
            flags.remove(account_id);
        }

        tracing::info!(target: "postail", "[IMAP] Watch stop requested for {}", account_id);
    }

    async fn start_sync_async(&self, account_id: &str) -> Result<(), AppError> {
        if let Err(e) = self.fetch_mailboxes(account_id).await {
            tracing::error!(target: "postail",
                "[IMAP] Failed to fetch mailbox list for {}: {}",
                account_id, e
            );
        }
        mark_sync_complete(account_id).await;
        Ok(())
    }

    pub(crate) async fn idle_mailbox(
        &self,
        account_id: &str,
        mailbox_name: &str,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;
        let mailbox = session.select(mailbox_name).await.map_err(AppError::from)?;

        let uid_validity = mailbox.uid_validity.unwrap_or(0);
        let highest_uid = mailbox.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);

        self.check_uidvalidity(account_id, mailbox_name, uid_validity)
            .await?;

        let mut last_uid = self.get_last_synced_uid(account_id, mailbox_name).await?;

        if highest_uid > last_uid {
            let _ = self
                .fetch_missing_messages(
                    account_id,
                    mailbox_name,
                    last_uid.saturating_add(1),
                    highest_uid,
                )
                .await?;
            last_uid = highest_uid;
            mark_sync_complete(account_id).await;
        }

        tracing::info!(target: "postail",
            "[IMAP] Starting IDLE for {}@{}",
            mailbox_name, account_id
        );

        let mut idle = session.idle();
        if idle.init().await.is_err() {
            let session = idle.done().await?;
            tracing::warn!(target: "postail",
                "[IMAP] IDLE init failed for {}@{}, falling back to polling",
                mailbox_name, account_id
            );
            return self
                .poll_loop(session, account_id, mailbox_name, &mut last_uid, stop_flag)
                .await;
        }

        tracing::info!(target: "postail",
            "[IMAP] Entering IDLE mode for {}@{}",
            mailbox_name, account_id
        );
        self.idle_loop(idle, account_id, mailbox_name, &mut last_uid, stop_flag)
            .await
    }

    async fn idle_loop(
        &self,
        mut idle: async_imap::extensions::idle::Handle<
            tokio_util::compat::Compat<tokio_native_tls::TlsStream<tokio::net::TcpStream>>,
        >,
        account_id: &str,
        mailbox_name: &str,
        last_uid: &mut u32,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        loop {
            // ── Stop check at top of every loop iteration ─────────────────
            if stop_flag.load(Ordering::Acquire) {
                mark_sync_complete(account_id).await;
                let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                    mailbox: mailbox_name.to_string(),
                    error: e.to_string(),
                })?;
                let _ = session.logout().await;
                return Ok(());
            }

            // ── Arm IDLE wait ─────────────────────────────────────────────
            // wait_future borrows idle, so idle.done() (which moves idle) must happen after wait_future is dropped at the end of this block
            let (wait_result, stop_early) = {
                let (wait_future, interrupt) = idle.wait();
                tokio::pin!(wait_future);
                {
                    let mut interrupts = IDLE_INTERRUPTS.lock().await;
                    interrupts.insert(account_id.to_string(), interrupt);
                }
                if stop_flag.load(Ordering::Acquire) {
                    interrupt_idle(account_id).await;
                    (None, true)
                } else {
                    // Wait for either timeout or IDLE notification.
                    let result = tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(RFC_IDLE_TIMEOUT_SECS)) => {
                            None // Timeout
                        }
                        result = &mut wait_future => {
                            Some(result)
                        }
                    };
                    (result, false)
                }
            };

            if stop_early {
                mark_sync_complete(account_id).await;
                let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                    mailbox: mailbox_name.to_string(),
                    error: e.to_string(),
                })?;
                let _ = session.logout().await;
                return Ok(());
            }

            match wait_result {
                None => {
                    // Timeout: send DONE and re-enter IDLE.
                    let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;

                    // Stop check before re-entering IDLE.
                    if stop_flag.load(Ordering::Acquire) {
                        mark_sync_complete(account_id).await;
                        let _ = session.logout().await;
                        return Ok(());
                    }

                    tracing::info!(target: "postail",
                        "[IMAP] IDLE RFC timeout for {}@{}, re-entering IDLE",
                        mailbox_name, account_id
                    );
                    idle = session.idle();
                    if idle.init().await.is_err() {
                        let session =
                            idle.done()
                                .await
                                .map_err(|_e| ImapError::IdleReinitFailed {
                                    mailbox: mailbox_name.to_string(),
                                })?;
                        tracing::warn!(target: "postail",
                            "[IMAP] IDLE reinit failed for {}@{}, switching to polling",
                            mailbox_name, account_id
                        );
                        return self
                            .poll_loop(session, account_id, mailbox_name, last_uid, stop_flag)
                            .await;
                    }
                }
                Some(Ok(_notification)) => {
                    // Server pushed a notification (new mail, flag change, expunge, etc.)
                    let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;

                    if stop_flag.load(Ordering::Acquire) {
                        mark_sync_complete(account_id).await;
                        let _ = session.logout().await;
                        return Ok(());
                    }

                    let mailbox = session.select(mailbox_name).await.map_err(
                        |e: async_imap::error::Error| ImapError::MailboxSyncError {
                            mailbox: mailbox_name.to_string(),
                            error: e.to_string(),
                        },
                    )?;
                    let new_highest_uid =
                        mailbox.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);

                    if new_highest_uid > *last_uid {
                        let (actual_new_count, subject, sender) = self
                            .fetch_missing_messages(
                                account_id,
                                mailbox_name,
                                last_uid.saturating_add(1),
                                new_highest_uid,
                            )
                            .await?;
                        *last_uid = new_highest_uid;
                        mark_sync_complete(account_id).await;
                        if actual_new_count > 0 {
                            SYNC_STATUS_MANAGER
                                .emit_new_messages(
                                    account_id,
                                    mailbox_name,
                                    actual_new_count,
                                    new_highest_uid,
                                    subject,
                                    sender,
                                )
                                .await;
                        }
                    } else {
                        tracing::debug!(target: "postail",
                            "[IMAP] IDLE wakeup but no new messages in {}@{}, syncing flags",
                            mailbox_name, account_id
                        );
                        if let Err(e) = self
                            .sync_flags_from_server(account_id, mailbox_name, None)
                            .await
                        {
                            tracing::warn!(target: "postail",
                                "[IMAP] Flag sync failed for {}@{}: {}",
                                mailbox_name, account_id, e
                            );
                        }
                    }

                    idle = session.idle();
                    if idle.init().await.is_err() {
                        let session =
                            idle.done()
                                .await
                                .map_err(|_e| ImapError::IdleReinitFailed {
                                    mailbox: mailbox_name.to_string(),
                                })?;
                        tracing::warn!(target: "postail",
                            "[IMAP] IDLE reinit failed for {}@{}, switching to polling",
                            mailbox_name, account_id
                        );
                        return self
                            .poll_loop(session, account_id, mailbox_name, last_uid, stop_flag)
                            .await;
                    }
                }
                Some(Err(e)) => {
                    // IDLE stream error - fall back to polling.
                    let session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;
                    tracing::warn!(target: "postail",
                        "[IMAP] IDLE wait error for {}@{}: {}, switching to polling",
                        mailbox_name, account_id, e
                    );
                    return self
                        .poll_loop(session, account_id, mailbox_name, last_uid, stop_flag)
                        .await;
                }
            }
        }
    }

    async fn poll_loop(
        &self,
        mut session: crate::imap::connection::ImapSession,
        account_id: &str,
        mailbox_name: &str,
        last_uid: &mut u32,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        loop {
            if stop_flag.load(Ordering::Acquire) {
                mark_sync_complete(account_id).await;
                let _ = session.logout().await;
                return Ok(());
            }

            // ── Interruptible sleep ───────────────────────────────────────
            // Register a Notify so stop_sync()/stop_watch_mailbox() can wake us immediately
            let notify = Arc::new(tokio::sync::Notify::new());
            {
                let mut interrupts = POLL_INTERRUPTS.lock().await;
                interrupts.insert(account_id.to_string(), notify.clone());
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)) => {}
                _ = notify.notified() => {
                    tracing::debug!(target: "postail",
                        "[IMAP] Poll sleep interrupted for {}@{}",
                        mailbox_name, account_id
                    );
                }
            }
            {
                let mut interrupts = POLL_INTERRUPTS.lock().await;
                interrupts.remove(account_id);
            }

            if stop_flag.load(Ordering::Acquire) {
                mark_sync_complete(account_id).await;
                let _ = session.logout().await;
                return Ok(());
            }

            // ── Poll ──────────────────────────────────────────────────────
            match session.noop().await {
                Ok(_) => {
                    let mailbox = session.select(mailbox_name).await?;
                    let new_highest_uid =
                        mailbox.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);

                    if new_highest_uid > *last_uid {
                        let (actual_new_count, subject, sender) = self
                            .fetch_missing_messages(
                                account_id,
                                mailbox_name,
                                last_uid.saturating_add(1),
                                new_highest_uid,
                            )
                            .await?;
                        *last_uid = new_highest_uid;
                        mark_sync_complete(account_id).await;
                        if actual_new_count > 0 {
                            SYNC_STATUS_MANAGER
                                .emit_new_messages(
                                    account_id,
                                    mailbox_name,
                                    actual_new_count,
                                    new_highest_uid,
                                    subject,
                                    sender,
                                )
                                .await;
                        }
                    } else if let Err(e) = self
                        .sync_flags_from_server(account_id, mailbox_name, None)
                        .await
                    {
                        tracing::warn!(target: "postail",
                            "[IMAP] Flag sync failed during poll for {}@{}: {}",
                            mailbox_name, account_id, e
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(target: "postail",
                        "[IMAP] NOOP failed for {}@{}: {}, reconnecting...",
                        mailbox_name, account_id, e
                    );
                    match self.connect_imap(account_id).await {
                        Ok(new_session) => {
                            session = new_session;
                            if let Err(select_err) = session.select(mailbox_name).await {
                                tracing::error!(target: "postail",
                                    "[IMAP] SELECT after reconnect failed for {}@{}: {}",
                                    mailbox_name, account_id, select_err
                                );
                                return Err(AppError::from(select_err));
                            }
                        }
                        Err(reconnect_err) => {
                            tracing::error!(target: "postail",
                                "[IMAP] Reconnect failed for {}@{}: {}",
                                mailbox_name, account_id, reconnect_err
                            );
                            return Err(AppError::Imap(reconnect_err));
                        }
                    }
                }
            }
        }
    }

    pub(crate) async fn get_last_synced_uid(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<u32, AppError> {
        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT last_synced_uid FROM mailboxes WHERE account_id = ? AND name = ?")
            .map_err(|e| AppError::from(e.to_string()))?;
        let last_uid: Option<i64> = stmt
            .query_row([account_id, mailbox_name], |row| row.get(0))
            .ok();
        Ok(last_uid.unwrap_or(0) as u32)
    }

    pub(crate) async fn fetch_missing_messages(
        &self,
        account_id: &str,
        mailbox_name: &str,
        start_uid: u32,
        end_uid: u32,
    ) -> Result<(u32, Option<String>, Option<String>), AppError> {
        if start_uid > end_uid {
            return Ok((0, None, None));
        }

        let total = end_uid.saturating_sub(start_uid).saturating_add(1);
        let limit: u32 = 100;
        let mut anchor = start_uid;
        let mut latest_uid = start_uid;
        let mut processed = 0u32;
        let mut actual_new_count = 0u32;
        let mut newest_subject: Option<String> = None;
        let mut newest_sender: Option<String> = None;

        while anchor <= end_uid {
            update_sync_status(account_id, mailbox_name, processed, total).await;

            let headers = self
                .fetch_headers(account_id, mailbox_name, Some(anchor), limit)
                .await?;

            if headers.is_empty() {
                break;
            }

            for h in &headers {
                if !h.flags.iter().any(|f| f.eq_ignore_ascii_case("\\Seen")) {
                    actual_new_count += 1;
                }
            }

            if let Some(h) = headers.last() {
                latest_uid = h.uid;
                newest_subject = h.subject.clone();
                newest_sender = h.from.first().cloned();
            }

            processed += headers.len() as u32;
            anchor = latest_uid.saturating_add(1);

            if headers.len() < limit as usize {
                break;
            }
        }

        update_sync_status(account_id, mailbox_name, total, total).await;

        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
        conn.execute(
            "UPDATE mailboxes SET last_synced_uid = ? WHERE account_id = ? AND name = ?",
            rusqlite::params![end_uid, account_id, mailbox_name],
        )
        .map_err(|e| AppError::from(e.to_string()))?;

        Ok((actual_new_count, newest_subject, newest_sender))
    }
}
