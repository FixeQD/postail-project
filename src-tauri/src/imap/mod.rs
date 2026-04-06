use crate::db::{DbPool, PooledConn};
use crate::error::AppError;
use crate::imap::connection::ImapSession;
use crate::imap::sync_status::{SYNC_STATUS_MANAGER, mark_sync_complete, update_sync_status};
use crate::security::SecurityManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod connection;
pub mod flags;
pub mod folder_ops;
pub mod mailbox;
pub mod pool;
pub mod sync;
pub mod sync_status;

/// Guard that ensures IMAP session is properly logged out when dropped
pub struct SessionGuard {
    session: Option<ImapSession>,
}

impl SessionGuard {
    pub fn new(session: ImapSession) -> Self {
        Self {
            session: Some(session),
        }
    }

    pub fn get_mut(&mut self) -> &mut ImapSession {
        self.session.as_mut().unwrap()
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let Some(mut session) = self.session.take() {
            // Spawn a blocking task to logout since Drop is synchronous
            tokio::spawn(async move {
                let _ = session.logout().await;
            });
        }
    }
}

pub struct ImapManager {
    conn: Arc<Mutex<Option<DbPool>>>,
    security: Arc<Mutex<SecurityManager>>,
}

impl ImapManager {
    pub fn new(conn: Arc<Mutex<Option<DbPool>>>, security: Arc<Mutex<SecurityManager>>) -> Self {
        Self { conn, security }
    }

    pub fn get_conn(&self) -> Arc<Mutex<Option<DbPool>>> {
        Arc::clone(&self.conn)
    }

    pub async fn get_db(&self) -> Result<PooledConn, crate::error::DBError> {
        let pool = crate::globals::get_db_pool().await?;
        pool.get()
            .map_err(|e| crate::error::DBError::Pool(e.to_string()))
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

        // Use SessionGuard to ensure logout is called even on early return
        let mut session_guard = SessionGuard::new(self.connect_imap(account_id).await?);
        let selected = session_guard
            .get_mut()
            .select(mailbox_name)
            .await
            .map_err(AppError::from)?;

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

        if let Some(mut session) = session_guard.session.take() {
            session.logout().await.map_err(AppError::from)?;
        }

        mark_sync_complete(account_id).await;

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
