use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;
use crate::error::{AppError, ImapError};
use crate::imap::sync_status::{mark_sync_complete, SYNC_STATUS_MANAGER};
use super::utils::*;

impl crate::imap::ImapManager {
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
                    let mut session = idle.done().await.map_err(|e| ImapError::IdleWaitError {
                        mailbox: mailbox_name.to_string(),
                        error: e.to_string(),
                    })?;

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
}
