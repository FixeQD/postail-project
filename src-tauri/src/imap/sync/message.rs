use futures::StreamExt;
use rusqlite::params;

use crate::db;

impl crate::imap::ImapManager {
    pub async fn fetch_message_full_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        let conn_guard = self.conn.lock().await;
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        match crate::db::fetch_message_full(conn, account_id, mailbox, uid) {
            Ok(message) => Ok(message),
            Err(e) => Err(format!("Failed to fetch message: {}", e)),
        }
    }

    pub async fn fetch_message_full(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        let mut session = self.connect_imap(account_id).await?;
        session.select(mailbox).await.map_err(|e| e.to_string())?;

        let fetch_result = {
            let mut fetches = session
                .uid_fetch(format!("{}", uid), "(BODY[])")
                .await
                .map_err(|e| e.to_string())?;
            
            if let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let body_raw = fetch.body().ok_or("No body")?;
                
                // Get message_table_id
                let conn_guard = self.conn.lock().await;
                let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
                
                let id: i64 = conn.query_row(
                    "SELECT id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
                    params![account_id, mailbox, uid],
                    |row| row.get(0)
                ).map_err(|e| e.to_string())?;
                
                // Save to DB (cache)
                crate::db::message_bodies::save_message_body_with_fallback(conn, id, body_raw)
                    .map_err(|e| e.to_string())?;
                
                // Return fully populated struct from DB
                db::fetch_message_full(conn, account_id, mailbox, uid)
                    .map_err(|e| e.to_string())?
                    .ok_or("Message not found after sync".to_string())
                    .map(Some)
            } else {
                Ok(None)
            }
        };

        let _ = session.logout().await;
        fetch_result
    }
}
