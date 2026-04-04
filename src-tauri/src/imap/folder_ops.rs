use crate::db::{Mailbox, upsert_mailbox};
use crate::error::AppError;
use crate::globals::get_db_pool;
use crate::imap::ImapManager;
use rusqlite::params;

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

    pub async fn rename_folder(
        &self,
        account_id: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;

        tracing::info!(target: "postail",
            "[IMAP] Renaming folder '{}' -> '{}' for {}",
            old_name, new_name, account_id
        );

        session
            .rename(old_name, new_name)
            .await
            .map_err(AppError::from)?;

        session.logout().await.map_err(AppError::from)?;

        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let mut conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
        let tx = conn.transaction().map_err(AppError::from)?;

        // Update the mailbox row itself
        tx.execute(
            "UPDATE mailboxes SET name = ? WHERE account_id = ? AND name = ?",
            params![new_name, account_id, old_name],
        )
        .map_err(AppError::from)?;

        // Re-point all messages that lived in the old folder
        tx.execute(
            "UPDATE messages SET mailbox = ? WHERE account_id = ? AND mailbox = ?",
            params![new_name, account_id, old_name],
        )
        .map_err(AppError::from)?;

        tx.commit().map_err(AppError::from)?;

        tracing::info!(target: "postail",
            "[IMAP] Folder renamed '{}' -> '{}' in DB", old_name, new_name
        );

        Ok(())
    }
}
