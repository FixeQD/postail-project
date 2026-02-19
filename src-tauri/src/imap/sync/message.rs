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
            tracing::info!(target: "postail", "[IMAP] fetch_message_full: calling uid_fetch for {}", uid);
            let mut fetches = session
                .uid_fetch(format!("{}", uid), "(BODY[])")
                .await
                .map_err(|e| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: uid_fetch failed: {}", e);
                    e.to_string()
                })?;
            
            if let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: fetch.next() failed: {}", e);
                    e.to_string()
                })?;
                let body_owned: Vec<u8> = fetch.body().ok_or_else(|| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: No body in fetch result for uid={}", uid);
                    "No body".to_string()
                })?.to_vec(); // owned

                tracing::info!(target: "postail", "[IMAP] fetch_message_full: got {} bytes for uid={}", body_owned.len(), uid);

                drop(fetch);
                drop(fetches);
                
                tracing::info!(target: "postail", "[IMAP] fetch_message_full: locking connection to save body for uid={}", uid);
                let conn_guard = self.conn.lock().await;
                let conn = conn_guard.as_ref().ok_or_else(|| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: Database not initialized when saving uid={}", uid);
                    "Database not initialized".to_string()
                })?;
                
                let id: i64 = conn.query_row(
                    "SELECT id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
                    params![account_id, mailbox, uid],
                    |row| row.get(0)
                ).map_err(|e| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: Failed to find id for uid={}: {}", uid, e);
                    e.to_string()
                })?;
                
                tracing::info!(target: "postail", "[IMAP] fetch_message_full: found message id={} for uid={}", id, uid);

                // Save to DB (cache)
                crate::db::message_bodies::save_message_body_with_fallback(conn, id, &body_owned)
                    .map_err(|e| {
                        tracing::error!(target: "postail", "[IMAP] fetch_message_full: save_message_body failed for uid={}: {}", uid, e);
                        e.to_string()
                    })?;
                
                tracing::info!(target: "postail", "[IMAP] fetch_message_full: saved body to DB for uid={}, refetching full struct", uid);

                // Return fully populated struct from DB
                db::fetch_message_full(conn, account_id, mailbox, uid)
                    .map_err(|e| {
                        tracing::error!(target: "postail", "[IMAP] fetch_message_full: db::fetch_message_full failed after save for uid={}: {}", uid, e);
                        e.to_string()
                    })?
                    .ok_or_else(|| {
                        tracing::error!(target: "postail", "[IMAP] fetch_message_full: db::fetch_message_full returned None after save for uid={}", uid);
                        "Message not found after sync".to_string()
                    })
                    .map(Some)
            } else {
                tracing::warn!(target: "postail", "[IMAP] fetch_message_full: No fetch results for uid={}", uid);
                Ok(None)
            }
        };

        let _ = session.logout().await;
        fetch_result
    }
}
