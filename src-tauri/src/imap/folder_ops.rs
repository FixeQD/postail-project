use crate::db::{Mailbox, upsert_mailbox};
use crate::error::AppError;
use crate::globals::get_db_pool;
use crate::imap::ImapManager;
use rusqlite::params;

impl ImapManager {
    pub async fn create_folder(
        &self,
        account_id: &str,
        name: &str,
        delimiter: Option<&str>,
    ) -> Result<(), AppError> {
        let fetched_delim;
        let delim = match delimiter {
            Some(d) => d,
            None => {
                fetched_delim = self.get_hierarchy_delimiter(account_id, "").await?;
                &fetched_delim
            }
        };

        let mut session = self.connect_imap(account_id).await?;

        tracing::info!(target: "postail", "[IMAP] Creating folder '{}' for {}", name, account_id);

        let parts: Vec<&str> = name.split(delim).collect();
        let mut current_path = String::new();

        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;

        for part in parts {
            if current_path.is_empty() {
                current_path = part.to_string();
            } else {
                current_path = format!("{}{}{}", current_path, delim, part);
            }

            match session.create(&current_path).await {
                Ok(_) => {
                    tracing::info!(target: "postail", "[IMAP] Created path component '{}'", current_path);
                    let mailbox = Mailbox {
                        name: current_path.clone(),
                        display_name: current_path
                            .rsplit(delim)
                            .next()
                            .unwrap_or(&current_path)
                            .to_string(),
                        role: "other".to_string(),
                        uid_validity: None,
                        highest_modseq: None,
                        last_synced_uid: None,
                        hidden: false,
                        separator: delim.to_string(),
                    };
                    upsert_mailbox(&conn, account_id, &mailbox).map_err(AppError::from)?;
                }
                Err(e) => {
                    // Ignore ALREADYEXISTS errors, surface others if it's the final part
                    if current_path == name {
                        return Err(AppError::from(e));
                    }
                }
            }
        }

        session.logout().await.map_err(AppError::from)?;

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

        let separator: String = tx
            .query_row(
                "SELECT separator FROM mailboxes WHERE account_id = ? AND name = ?",
                params![account_id, old_name],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "/".to_string());

        let old_prefix = format!("{}{}", old_name, separator);
        let new_prefix = format!("{}{}", new_name, separator);

        tx.execute(
            "UPDATE mailboxes SET name = ? WHERE account_id = ? AND name = ?",
            params![new_name, account_id, old_name],
        )
        .map_err(AppError::from)?;

        tx.execute(
            "UPDATE messages SET mailbox = ? WHERE account_id = ? AND mailbox = ?",
            params![new_name, account_id, old_name],
        )
        .map_err(AppError::from)?;

        tx.execute(
            "UPDATE mailboxes
             SET name = ? || substr(name, ?)
             WHERE account_id = ? AND name LIKE ?",
            params![
                new_prefix,
                (old_prefix.len() + 1) as i64,
                account_id,
                format!("{}%", old_prefix)
            ],
        )
        .map_err(AppError::from)?;

        tx.execute(
            "UPDATE messages
             SET mailbox = ? || substr(mailbox, ?)
             WHERE account_id = ? AND mailbox LIKE ?",
            params![
                new_prefix,
                (old_prefix.len() + 1) as i64,
                account_id,
                format!("{}%", old_prefix)
            ],
        )
        .map_err(AppError::from)?;

        tx.commit().map_err(AppError::from)?;

        tracing::info!(target: "postail",
            "[IMAP] Folder renamed '{}' -> '{}' in DB", old_name, new_name
        );

        Ok(())
    }

    pub async fn delete_folder(&self, account_id: &str, name: &str) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;

        tracing::info!(target: "postail", "[IMAP] Deleting folder '{}' for {}", name, account_id);

        session.delete(name).await.map_err(AppError::from)?;
        session.logout().await.map_err(AppError::from)?;

        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;

        // messages don't cascade from mailboxes, so clean up explicitly
        conn.execute(
            "DELETE FROM messages WHERE account_id = ? AND mailbox = ?",
            params![account_id, name],
        )
        .map_err(AppError::from)?;

        conn.execute(
            "DELETE FROM mailboxes WHERE account_id = ? AND name = ?",
            params![account_id, name],
        )
        .map_err(AppError::from)?;

        tracing::info!(target: "postail", "[IMAP] Folder '{}' deleted from DB", name);

        Ok(())
    }

    pub async fn subscribe_folder(&self, account_id: &str, name: &str) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;

        tracing::info!(target: "postail", "[IMAP] Subscribing to folder '{}' for {}", name, account_id);

        session.subscribe(name).await.map_err(AppError::from)?;
        session.logout().await.map_err(AppError::from)?;

        tracing::info!(target: "postail", "[IMAP] Subscribed to folder '{}'", name);

        Ok(())
    }

    pub async fn unsubscribe_folder(&self, account_id: &str, name: &str) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;

        tracing::info!(target: "postail", "[IMAP] Unsubscribing from folder '{}' for {}", name, account_id);

        session.unsubscribe(name).await.map_err(AppError::from)?;
        session.logout().await.map_err(AppError::from)?;

        tracing::info!(target: "postail", "[IMAP] Unsubscribed from folder '{}'", name);

        Ok(())
    }

    /// Returns the hierarchy delimiter for a given mailbox (or "/" as fallback).
    pub async fn get_hierarchy_delimiter(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<String, AppError> {
        let mut session = self.connect_imap(account_id).await?;

        // LIST "" "parent" returns just that one mailbox with its delimiter
        let pattern = format!("\"{}\"", mailbox_name.replace('"', "\\\""));
        let mut list = session
            .list(None, Some(&pattern))
            .await
            .map_err(AppError::from)?;

        use futures::StreamExt;
        let delimiter = if let Some(Ok(mb)) = list.next().await {
            mb.delimiter().unwrap_or("/").to_string()
        } else {
            "/".to_string()
        };

        drop(list);
        session.logout().await.map_err(AppError::from)?;

        Ok(delimiter)
    }

    pub async fn create_subfolder(
        &self,
        account_id: &str,
        parent_name: &str,
        child_name: &str,
    ) -> Result<String, AppError> {
        let delimiter = self
            .get_hierarchy_delimiter(account_id, parent_name)
            .await?;
        let full_name = format!("{}{}{}", parent_name, delimiter, child_name);

        tracing::info!(target: "postail",
            "[IMAP] Creating subfolder '{}' (delimiter='{}')",
            full_name, delimiter
        );

        self.create_folder(account_id, &full_name, Some(delimiter.as_str()))
            .await?;
        Ok(full_name)
    }
}
