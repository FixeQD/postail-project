use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

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

static STOP_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static IDLE_INTERRUPTS: LazyLock<Mutex<HashMap<String, stop_token::StopSource>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Per-account mailbox watch

static WATCH_STOP_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl crate::imap::ImapManager {
    pub async fn start_sync(&self, account_id: &str) -> Result<(), AppError> {
        // Guard: don't spawn a second task if one is already running for this account
        {
            let flags = STOP_FLAGS.lock().await;
            if let Some(flag) = flags.get(account_id) {
                if !flag.load(Ordering::SeqCst) {
                    // Flag exists and is NOT set to stop — sync is already running
                    tracing::info!(target: "postail", "[IMAP] start_sync called but already running for {}, ignoring", account_id);
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

        // Use SYNC_STATUS_MANAGER's stop flag so stop_sync() can actually halt the IDLE loop
        let stop_flag = SYNC_STATUS_MANAGER.get_stop_flag(account_id).await;
        // Reset the flag in case it was previously set
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
            tracing::info!(target: "postail", "[IMAP] Sync done");
        });

        tracing::info!(target: "postail", "[IMAP] start_sync completed");
        Ok(())
    }

    pub async fn stop_all_syncs(&self) -> Result<(), AppError> {
        let accounts: Vec<String> = {
            let flags = STOP_FLAGS.lock().await;
            flags.keys().cloned().collect()
        };

        for account_id in accounts {
            if let Err(e) = self.stop_sync(&account_id).await {
                tracing::error!(target: "postail", "[IMAP] Failed to stop sync for {}: {}", account_id, e);
            }
        }
        Ok(())
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

        // Interrupt any active IDLE
        {
            let mut interrupts = IDLE_INTERRUPTS.lock().await;
            if let Some(interrupt) = interrupts.remove(account_id) {
                tracing::info!(target: "postail", "[IMAP] Interrupting IDLE for {}", account_id);
                drop(interrupt);
            }
        }

        tracing::info!(target: "postail", "[IMAP] Stop requested for {}", account_id);
        Ok(())
    }

    /// Start IDLE/poll watch for a single mailbox
    pub async fn start_watch_mailbox(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<(), AppError> {
        // Stop previous watch for this account if running
        self.stop_watch_mailbox(account_id).await;

        // Register account for status tracking so frontend gets events
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
            tracing::info!(target: "postail", "[IMAP] Watch started for {}@{}", mailbox_owned, account_id_owned);
            match manager
                .idle_mailbox(&account_id_owned, &mailbox_owned, &stop_flag)
                .await
            {
                Ok(()) => {
                    tracing::info!(target: "postail", "[IMAP] Watch ended cleanly for {}@{}", mailbox_owned, account_id_owned);
                }
                Err(e) => {
                    tracing::error!(target: "postail", "[IMAP] Watch error for {}@{}: {}", mailbox_owned, account_id_owned, e);
                }
            }
        });

        tracing::info!(target: "postail", "[IMAP] Watch spawned for {}@{}", mailbox_name, account_id);
        Ok(())
    }

    /// Stop the active mailbox watch for an account.
    pub async fn stop_watch_mailbox(&self, account_id: &str) {
        // Signal stop
        let flag = {
            let flags = WATCH_STOP_FLAGS.lock().await;
            flags.get(account_id).cloned()
        };
        if let Some(f) = flag {
            f.store(true, Ordering::SeqCst);
        }

        // Interrupt IDLE so the thread wakes up
        {
            let mut interrupts = IDLE_INTERRUPTS.lock().await;
            if let Some(interrupt) = interrupts.remove(account_id) {
                tracing::info!(target: "postail", "[IMAP] Interrupting watch IDLE for {}", account_id);
                drop(interrupt);
            }
        }

        // Cleanup flag
        {
            let mut flags = WATCH_STOP_FLAGS.lock().await;
            flags.remove(account_id);
        }

        tracing::info!(target: "postail", "[IMAP] Watch stop requested for {}", account_id);
    }

    async fn start_sync_async(&self, account_id: &str) -> Result<(), AppError> {
        use crate::imap::sync_status::SYNC_STATUS_MANAGER;
        let stop_flag: Arc<AtomicBool> = SYNC_STATUS_MANAGER.get_stop_flag(account_id).await;

        loop {
            // Check if stop was requested via SYNC_STATUS_MANAGER
            if SYNC_STATUS_MANAGER.is_stop_requested(account_id).await {
                mark_sync_complete(account_id).await;
                return Ok(());
            }

            if stop_flag.load(Ordering::SeqCst) {
                mark_sync_complete(account_id).await;
                return Ok(());
            }

            match self.sync_all_mailboxes_async(account_id, &stop_flag).await {
                Ok(()) => {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
                Err(e) => {
                    tracing::error!(target: "postail", "[IMAP] Sync error for {}: {}", account_id, e);
                    mark_sync_error(account_id, &e.to_string()).await;
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            }
        }
    }

    async fn sync_all_mailboxes_async(
        &self,
        account_id: &str,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        let mailboxes = self.fetch_mailboxes(account_id).await?;
        let total_mailboxes = mailboxes.len() as u32;

        // Set total mailbox count
        SYNC_STATUS_MANAGER
            .set_mailbox_counters(account_id, 0, total_mailboxes)
            .await;

        for (idx, mailbox) in mailboxes.iter().enumerate() {
            if stop_flag.load(Ordering::SeqCst) {
                return Ok(());
            }

            // Update current mailbox counter
            SYNC_STATUS_MANAGER
                .set_mailbox_counters(account_id, idx as u32 + 1, total_mailboxes)
                .await;

            match self.sync_mailbox(account_id, &mailbox.name).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!(target: "postail", "[IMAP] Mailbox error for {}@{}: {}", mailbox.name, account_id, e);
                    // Continue with other mailboxes even if one fails
                }
            }
        }

        if stop_flag.load(Ordering::SeqCst) {
            return Ok(());
        }

        mark_sync_complete(account_id).await;

        let inbox_name = mailboxes
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case("INBOX"))
            .map(|m| m.name.as_str())
            .unwrap_or("INBOX");

        tracing::info!(target: "postail", "[IMAP] Entering IDLE phase for {}", inbox_name);
        match self.idle_mailbox(account_id, inbox_name, stop_flag).await {
            Ok(()) => {}
            Err(e) => {
                tracing::error!(target: "postail", "[IMAP] IDLE error for {}@{}: {}", inbox_name, account_id, e);
                mark_sync_error(account_id, &e.to_string()).await;
            }
        }

        Ok(())
    }

    async fn sync_mailbox(&self, account_id: &str, mailbox_name: &str) -> Result<(), AppError> {
        update_sync_status(account_id, mailbox_name, 0, 0).await;

        let mut session = self.connect_imap(account_id).await?;
        let mailbox = session.select(mailbox_name).await.map_err(AppError::from)?;

        let uid_validity = mailbox.uid_validity.unwrap_or(0);
        let highest_uid = mailbox.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);

        self.check_uidvalidity(account_id, mailbox_name, uid_validity)
            .await?;

        let last_uid = self.get_last_synced_uid(account_id, mailbox_name).await?;

        if highest_uid > last_uid {
            let start = if highest_uid > last_uid.saturating_add(50) {
                highest_uid.saturating_sub(50)
            } else {
                last_uid.saturating_add(1)
            };

            tracing::info!(target: "postail", "[IMAP] Catching up {}@{} (local: {}, remote: {}, fetch_start: {})", mailbox_name, account_id, last_uid, highest_uid, start);
            // Startup catch-up: silent, no notification emitted
            let _ = self
                .fetch_missing_messages(account_id, mailbox_name, start, highest_uid)
                .await?;
        }

        if let Err(e) = self
            .sync_flags_from_server(account_id, mailbox_name, None)
            .await
        {
            tracing::warn!(target: "postail",
                "[IMAP] Initial flag sync failed for {}@{}: {}",
                mailbox_name, account_id, e
            );
        }

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
            // Startup catch-up before IDLE: silent, no notification emitted
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

        tracing::info!(target: "postail", "[IMAP] Starting IDLE for {}@{}", mailbox_name, account_id);

        let mut idle = session.idle();
        if idle.init().await.is_err() {
            let session = idle.done().await?;
            tracing::warn!(target: "postail",
                "[IMAP] IDLE init failed for {}@{}, switching to polling",
                mailbox_name, account_id
            );
            self.poll_loop(session, account_id, mailbox_name, &mut last_uid, stop_flag)
                .await
        } else {
            tracing::info!(target: "postail",
                "[IMAP] Entering IDLE mode for {}@{}",
                mailbox_name, account_id
            );
            self.idle_loop(idle, account_id, mailbox_name, &mut last_uid, stop_flag)
                .await
        }
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
            if stop_flag.load(Ordering::SeqCst) {
                mark_sync_complete(account_id).await;
                let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                    mailbox: mailbox_name.to_string(),
                    error: e.to_string(),
                })?;
                let _ = session.logout().await;
                return Ok(());
            }

            let (wait_future, interrupt) = idle.wait();
            {
                let mut interrupts = IDLE_INTERRUPTS.lock().await;
                interrupts.insert(account_id.to_string(), interrupt);
            }
            match timeout(Duration::from_secs(RFC_IDLE_TIMEOUT_SECS), wait_future).await {
                Ok(Ok(_)) => {
                    let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;
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
                        session = idle
                            .done()
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
                Ok(Err(e)) => {
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
                Err(_) => {
                    let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;
                    tracing::info!(target: "postail",
                        "[IMAP] IDLE timeout for {}@{}, re-entering IDLE",
                        mailbox_name, account_id
                    );
                    idle = session.idle();
                    if idle.init().await.is_err() {
                        session = idle
                            .done()
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
            if stop_flag.load(Ordering::SeqCst) {
                mark_sync_complete(account_id).await;
                let _ = session.logout().await;
                return Ok(());
            }

            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

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
                        "[IMAP] NOOP error for {}@{}: {}, reconnecting...",
                        mailbox_name, account_id, e
                    );
                    session = self.connect_imap(account_id).await?;
                    let _ = session.select(mailbox_name).await?;
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
        let mut stmt = conn
            .prepare("UPDATE mailboxes SET last_synced_uid = ? WHERE account_id = ? AND name = ?")
            .map_err(|e| AppError::from(e.to_string()))?;
        stmt.execute([
            end_uid.to_string(),
            account_id.to_string(),
            mailbox_name.to_string(),
        ])
        .map_err(|e| AppError::from(e.to_string()))?;

        Ok((actual_new_count, newest_subject, newest_sender))
    }
}
