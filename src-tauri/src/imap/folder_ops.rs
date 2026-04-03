use crate::db::{Mailbox, upsert_mailbox};
use crate::error::AppError;
use crate::globals::get_db_pool;
use crate::imap::ImapManager;

impl ImapManager {
    pub async fn create_folder(&self, account_id: &str, name: &str) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;

        tracing::info!(target: "postail", "[IMAP] Creating folder '{}' for {}", name, account_id);

        session.create(name).await.map_err(AppError::from)?;

        session.logout().await.map_err(AppError::from)?;

        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;

        let mailbox = Mailbox {
            name: name.to_string(),
            display_name: name.rsplit('/').next().unwrap_or(name).to_string(),
            role: "other".to_string(),
            uid_validity: None,
            highest_modseq: None,
            last_synced_uid: None,
        };

        upsert_mailbox(&conn, account_id, &mailbox).map_err(AppError::from)?;

        tracing::info!(target: "postail", "[IMAP] Folder '{}' created and saved to DB", name);

        Ok(())
    }
}
