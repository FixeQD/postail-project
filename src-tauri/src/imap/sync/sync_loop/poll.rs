use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;
use crate::error::AppError;
use crate::imap::sync_status::{mark_sync_complete, SYNC_STATUS_MANAGER};
use super::utils::*;

impl crate::imap::ImapManager {
    pub(crate) async fn poll_loop(
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
}
