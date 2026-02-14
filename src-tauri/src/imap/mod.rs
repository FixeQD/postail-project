use crate::error::AppError;
use crate::imap::sync_status::{update_sync_status, SYNC_STATUS_MANAGER};
use crate::security::SecurityManager;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod connection;
pub mod flags;
pub mod mailbox;
pub mod sync;
pub mod sync_status;

pub struct ImapManager {
    conn: Arc<Mutex<Option<Connection>>>,
    security: Arc<Mutex<SecurityManager>>,
}

impl ImapManager {
    pub fn new(
        conn: Arc<Mutex<Option<Connection>>>,
        security: Arc<Mutex<SecurityManager>>,
    ) -> Self {
        Self { conn, security }
    }

    pub fn get_conn(&self) -> Arc<Mutex<Option<Connection>>> {
        Arc::clone(&self.conn)
    }

    /// Syncs messages for a single mailbox
    pub async fn sync_single_mailbox_messages(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<(), AppError> {
        SYNC_STATUS_MANAGER
            .set_mailbox_counters(account_id, 1, 1)
            .await;

        update_sync_status(account_id, mailbox_name, 0, 0).await;

        let mut session = self.connect_imap(account_id).await?;
        let selected = session.select(mailbox_name).await.map_err(AppError::from)?;

        let uid_validity = selected.uid_validity.unwrap_or(0);
        let highest_uid = selected.uid_next.map(|u| u.saturating_sub(1)).unwrap_or(0);

        self.check_uidvalidity(account_id, mailbox_name, uid_validity)
            .await
            .map_err(AppError::from)?;

        let last_uid = self.get_last_synced_uid(account_id, mailbox_name).await?;

        if highest_uid > last_uid {
            let start = if highest_uid > last_uid + 50 {
                highest_uid - 50
            } else {
                last_uid + 1
            };

            tracing::info!(target: "postail",
                "[IMAP] Single mailbox sync {}@{} (local: {}, remote: {}, fetch_start: {})",
                mailbox_name, account_id, last_uid, highest_uid, start
            );
            self.fetch_missing_messages(account_id, mailbox_name, start, highest_uid)
                .await?;
        }

        session.logout().await.map_err(AppError::from)?;
        Ok(())
    }
}

impl Clone for ImapManager {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            security: Arc::clone(&self.security),
        }
    }
}
