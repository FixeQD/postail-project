pub mod idle;
pub mod poll;
pub mod ops;
pub mod utils;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::db::accounts::get_account_email;
use crate::error::AppError;
use crate::globals::get_db_pool;
use crate::imap::sync_status::{
    SYNC_STATUS_MANAGER, mark_sync_complete, mark_sync_error, start_sync_status_tracking,
};
use utils::*;

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
}
