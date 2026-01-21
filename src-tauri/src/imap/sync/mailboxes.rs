use async_std::stream::StreamExt;

use crate::db::{fetch_mailboxes as db_fetch_mailboxes, upsert_mailbox, Mailbox};

impl crate::imap::ImapManager {
    pub fn fetch_mailboxes_sync(&self, account_id: &str) -> Result<Vec<Mailbox>, String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        db_fetch_mailboxes(conn, account_id).map_err(|e| e.to_string())
    }

    pub async fn fetch_mailboxes(&self, account_id: &str) -> Result<Vec<Mailbox>, String> {
        tracing::info!(target: "postail", "[IMAP] fetch_mailboxes: calling connect_imap for {}", account_id);
        let mut session = self.connect_imap(account_id).await?;
        tracing::info!(target: "postail", "[IMAP] fetch_mailboxes: connected, listing mailboxes");
        let mut result = Vec::new();
        {
            let mut mailboxes = session
                .list(None, Some("*"))
                .await
                .map_err(|e| e.to_string())?;
            while let Some(mb) = mailboxes.next().await {
                let mb = mb.map_err(|e| e.to_string())?;
                let name = mb.name().to_string();
                let mailbox = Mailbox {
                    name,
                    display_name: mb.name().to_string(), // Default
                    role: "other".to_string(), // Default
                    uid_validity: None,
                    highest_modseq: None,
                    last_synced_uid: None,
                };
                {
                    let conn_guard = self.conn.lock().unwrap();
                    let conn = conn_guard
                        .as_ref()
                        .ok_or("Database not initialized".to_string())?;
                    upsert_mailbox(conn, account_id, &mailbox).map_err(|e| e.to_string())?;
                }
                result.push(mailbox);
            }
        }
        session.logout().await.map_err(|e| e.to_string())?;
        tracing::info!(target: "postail", "[IMAP] fetch_mailboxes: done, got {} mailboxes", result.len());
        Ok(result)
    }
}
