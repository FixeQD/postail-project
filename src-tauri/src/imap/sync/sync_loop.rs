use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::time::{timeout, Duration};
use tracing;

use crate::error::{AppError, ImapError, SyncError};
use crate::imap::sync_status::{
    mark_sync_complete, mark_sync_error, start_sync_status_tracking, stop_sync_status_tracking,
    update_sync_status,
};

const RFC_IDLE_TIMEOUT_SECS: u64 = 29 * 60;
const POLL_INTERVAL_SECS: u64 = 60;

lazy_static::lazy_static! {
    static ref SYNC_TASKS: std::sync::Mutex<Vec<thread::JoinHandle<()>>> = std::sync::Mutex::new(Vec::new());
    static ref STOP_FLAGS: std::sync::Mutex<std::collections::HashMap<String, Arc<AtomicBool>>> = std::sync::Mutex::new(std::collections::HashMap::new());
    static ref MAILBOX_TASKS: std::sync::Mutex<std::collections::HashMap<String, Vec<thread::JoinHandle<()>>>> = std::sync::Mutex::new(std::collections::HashMap::new());
}

impl crate::imap::ImapManager {
    pub fn start_sync(&self, account_id: &str) -> Result<(), AppError> {
        let manager = self.clone();
        let account_id_str = account_id.to_string();
        let account_id_for_error = account_id_str.clone();

        let handle = thread::Builder::new()
            .name(account_id_str.clone())
            .spawn(move || {
                tracing::info!(target: "postail", "[IMAP] Sync started for {}", account_id_str);
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| AppError::from(e.to_string()))
                    .and_then(|rt| {
                        rt.block_on(async {
                            start_sync_status_tracking(&account_id_str).await;
                            if let Err(e) = manager.start_sync_async(&account_id_str).await {
                                tracing::error!(target: "postail", "[IMAP] start_sync_async failed: {}", e);
                                mark_sync_error(&account_id_str, &e.to_string()).await;
                            }
                            tracing::info!(target: "postail", "[IMAP] Sync done");
                            Ok(())
                        })
                    });

                if let Err(ref e) = rt {
                    tracing::error!(target: "postail", "[IMAP] Sync runtime error: {}", e);
                }
            })
            .map_err(|e| {
                tracing::error!(target: "postail", "[IMAP] Failed to spawn thread for {}: {}", account_id_for_error, e);
                AppError::from(e.to_string())
            })?;

        let mut tasks = SYNC_TASKS.lock().unwrap();
        tasks.push(handle);

        tracing::info!(target: "postail", "[IMAP] start_sync completed");
        Ok(())
    }

    pub fn stop_sync(&self, account_id: &str) -> Result<(), AppError> {
        let (stop_flag, _handle_idx): (Arc<AtomicBool>, Option<usize>) = {
            let flags = STOP_FLAGS.lock().unwrap();
            let tasks = SYNC_TASKS.lock().unwrap();
            match (
                flags.get(account_id),
                Self::find_task_by_account_id(&tasks, account_id),
            ) {
                (Some(flag), Some((idx, _))) => (flag.clone(), Some(idx)),
                _ => (Arc::new(AtomicBool::new(false)), None),
            }
        };

        stop_flag.store(true, Ordering::SeqCst);

        {
            let mut mailbox_tasks = MAILBOX_TASKS.lock().unwrap();
            if let Some(tasks) = mailbox_tasks.get_mut(account_id) {
                for task in tasks.drain(..) {
                    let _ = task.join();
                }
            }
        }

        let handle = {
            let mut tasks = SYNC_TASKS.lock().unwrap();
            let idx = Self::find_task_by_account_id(&tasks, account_id)
                .map(|(idx, _)| idx)
                .ok_or_else(|| ImapError::NoSyncRunning {
                    account_id: account_id.to_string(),
                })?;
            tasks.remove(idx)
        };

        handle.join().map_err(|e| {
            let err = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "thread panicked".to_string()
            };
            SyncError::SyncThreadPanic {
                account_id: account_id.to_string(),
                error: err,
            }
        })?;

        {
            let mut flags = STOP_FLAGS.lock().unwrap();
            flags.remove(account_id);
        }

        {
            let mut mailbox_tasks = MAILBOX_TASKS.lock().unwrap();
            mailbox_tasks.remove(account_id);
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AppError::from(e.to_string()))?;

        rt.block_on(async {
            stop_sync_status_tracking(account_id).await;
        });

        Ok(())
    }

    fn find_task_by_account_id(
        tasks: &[thread::JoinHandle<()>],
        account_id: &str,
    ) -> Option<(usize, String)> {
        for (idx, handle) in tasks.iter().enumerate() {
            if let Some(id) = handle.thread().name() {
                if id == account_id {
                    return Some((idx, id.to_string()));
                }
            }
        }
        None
    }

    async fn start_sync_async(&self, account_id: &str) -> Result<(), AppError> {
        use crate::imap::sync_status::SYNC_STATUS_MANAGER;
        let stop_flag: Arc<AtomicBool> = SYNC_STATUS_MANAGER.get_stop_flag(account_id).await;

        loop {
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

        for mailbox in mailboxes {
            if stop_flag.load(Ordering::SeqCst) {
                return Ok(());
            }

            match self
                .idle_mailbox(account_id, &mailbox.name, stop_flag)
                .await
            {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!(target: "postail", "[IMAP] Mailbox error for {}: {}", mailbox.name, e);
                    mark_sync_error(account_id, &e.to_string()).await;
                }
            }
        }

        Ok(())
    }

    async fn idle_mailbox(
        &self,
        account_id: &str,
        mailbox_name: &str,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        update_sync_status(account_id, mailbox_name, 0, 0).await;

        let mut session = self.connect_imap(account_id).await?;
        let mailbox = session.select(mailbox_name).await.map_err(AppError::from)?;

        let uid_validity = mailbox.uid_validity.unwrap_or(0);
        let highest_uid = mailbox.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);

        self.check_uidvalidity(account_id, mailbox_name, uid_validity)
            .await?;

        let mut last_uid = self.get_last_synced_uid(account_id, mailbox_name).await?;

        if highest_uid > last_uid {
            self.fetch_missing_messages(account_id, mailbox_name, last_uid + 1, highest_uid)
                .await?;
            last_uid = highest_uid;
        }

        tracing::info!(target: "postail", "[IMAP] Starting sync for {}@{}", mailbox_name, account_id);

        let mut idle = session.idle();
        if idle.init().await.is_err() {
            session = idle.done().await?;
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
            async_native_tls::TlsStream<async_std::net::TcpStream>,
        >,
        account_id: &str,
        mailbox_name: &str,
        last_uid: &mut u32,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        loop {
            if stop_flag.load(Ordering::SeqCst) {
                let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                    mailbox: mailbox_name.to_string(),
                    error: e.to_string(),
                })?;
                let _ = session.logout().await;
                return Ok(());
            }

            let (wait_future, _interrupt) = idle.wait();
            match timeout(Duration::from_secs(RFC_IDLE_TIMEOUT_SECS), wait_future).await {
                Ok(Ok(_)) => {
                    let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;
                    let mailbox = session.select(mailbox_name).await.map_err(|e| {
                        ImapError::MailboxSyncError {
                            mailbox: mailbox_name.to_string(),
                            error: e.to_string(),
                        }
                    })?;
                    let new_highest_uid =
                        mailbox.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);
                    if new_highest_uid > *last_uid {
                        self.fetch_missing_messages(
                            account_id,
                            mailbox_name,
                            *last_uid + 1,
                            new_highest_uid,
                        )
                        .await?;
                        *last_uid = new_highest_uid;
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
        mut session: async_imap::Session<async_native_tls::TlsStream<async_std::net::TcpStream>>,
        account_id: &str,
        mailbox_name: &str,
        last_uid: &mut u32,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        loop {
            if stop_flag.load(Ordering::SeqCst) {
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
                        self.fetch_missing_messages(
                            account_id,
                            mailbox_name,
                            *last_uid + 1,
                            new_highest_uid,
                        )
                        .await?;
                        *last_uid = new_highest_uid;
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

    async fn get_last_synced_uid(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<u32, AppError> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or(AppError::from("Database not initialized"))?;
        let mut stmt = conn
            .prepare("SELECT last_synced_uid FROM mailboxes WHERE account_id = ? AND name = ?")
            .map_err(|e| AppError::from(e.to_string()))?;
        let last_uid: Option<i64> = stmt
            .query_row([account_id, mailbox_name], |row| row.get(0))
            .ok();
        Ok(last_uid.unwrap_or(0) as u32)
    }

    async fn fetch_missing_messages(
        &self,
        account_id: &str,
        mailbox_name: &str,
        start_uid: u32,
        end_uid: u32,
    ) -> Result<(), AppError> {
        if start_uid >= end_uid {
            return Ok(());
        }

        let total = end_uid - start_uid + 1;
        let limit: u32 = 100;
        let mut anchor = start_uid;
        let mut latest_uid = start_uid;
        let mut processed = 0u32;

        while anchor < end_uid {
            update_sync_status(account_id, mailbox_name, processed, total).await;

            let headers = self
                .fetch_headers(account_id, mailbox_name, Some(anchor), limit)
                .await?;

            if headers.is_empty() {
                break;
            }

            if let Some(h) = headers.last() {
                latest_uid = h.uid;
            }

            processed += headers.len() as u32;
            anchor = latest_uid + 1;

            if headers.len() < limit as usize {
                break;
            }
        }

        update_sync_status(account_id, mailbox_name, total, total).await;

        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or(AppError::from("Database not initialized"))?;
        let mut stmt = conn
            .prepare("UPDATE mailboxes SET last_synced_uid = ? WHERE account_id = ? AND name = ?")
            .map_err(|e| AppError::from(e.to_string()))?;
        stmt.execute([
            end_uid.to_string(),
            account_id.to_string(),
            mailbox_name.to_string(),
        ])
        .map_err(|e| AppError::from(e.to_string()))?;

        Ok(())
    }
}
