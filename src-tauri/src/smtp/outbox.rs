use std::fs;

use uuid::Uuid;

use crate::db::compose::outbox_db::extract_headers_from_raw;
use crate::db::{OutboxItem, enqueue_message, list_outbox};
use crate::globals::get_db_pool;
use crate::smtp::worker::OUTBOX_NOTIFY;

impl super::SmtpManager {
    pub async fn enqueue_message(
        &self,
        account_id: &str,
        raw_eml: &[u8],
    ) -> Result<String, String> {
        tracing::info!(
            target: "postail",
            "[Outbox] Enqueueing message for account: {}, size: {} bytes",
            account_id, raw_eml.len()
        );

        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("postail")
            .join("outbox");
        fs::create_dir_all(&data_dir).map_err(|e| {
            tracing::error!(target: "postail", "[Outbox] Failed to create outbox dir: {}", e);
            e.to_string()
        })?;
        let eml_path = data_dir.join(format!("{}.eml", Uuid::new_v4()));

        let security = self.security.lock().await;
        let encrypted_eml = security.encrypt(raw_eml).map_err(|e| {
            tracing::error!(target: "postail", "[Outbox] Failed to encrypt EML: {}", e);
            e.to_string()
        })?;
        drop(security);

        fs::write(&eml_path, encrypted_eml).map_err(|e| {
            tracing::error!(target: "postail", "[Outbox] Failed to write EML file: {}", e);
            e.to_string()
        })?;

        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        let (subject, recipient) = extract_headers_from_raw(raw_eml);
        let id = match enqueue_message(
            &*conn,
            account_id,
            &eml_path.to_string_lossy(),
            subject.as_deref(),
            &recipient,
        ) {
            Ok(id) => {
                tracing::info!(
                    target: "postail",
                    "[Outbox] Message enqueued successfully with ID: {}",
                    id
                );
                id
            }
            Err(e) => {
                tracing::error!(
                    target: "postail",
                    "[Outbox] Failed to enqueue message in DB: {}",
                    e
                );
                return Err(e.to_string());
            }
        };

        // Wake up the worker immediately
        OUTBOX_NOTIFY.notify_one();

        Ok(id)
    }

    pub async fn list_outbox(&self, account_id: &str) -> Result<Vec<OutboxItem>, String> {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        list_outbox(&*conn, account_id).map_err(|e| e.to_string())
    }

    /// Resets a failed/stuck message to PENDING and wakes the worker.
    pub async fn retry_sending(&self, outbox_id: &str) -> Result<(), String> {
        let (account_id,) = {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE outbox SET status = 'PENDING', last_error = NULL, next_retry = NULL \
                 WHERE id = ?",
                [outbox_id],
            )
            .map_err(|e| e.to_string())?;

            // Fetch account_id for the event payload.
            let account_id: String = conn
                .query_row(
                    "SELECT account_id FROM outbox WHERE id = ?",
                    [outbox_id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            (account_id,)
        };

        self.emit_outbox_event("outbox:message:pending", outbox_id, &account_id, None)
            .await;

        // Tell the worker there's something new to process.
        OUTBOX_NOTIFY.notify_one();

        Ok(())
    }

    /// Cancels a queued/failed message: deletes the EML file and the DB row, then emits a `outbox:message:cancelled` event so the UI can remove it.
    pub async fn cancel_sending(&self, outbox_id: &str) -> Result<(), String> {
        let (_eml_path, account_id) = {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let conn = pool.get().map_err(|e| e.to_string())?;

            let mut stmt = conn
                .prepare("SELECT raw_eml_path, account_id FROM outbox WHERE id = ?")
                .map_err(|e| e.to_string())?;

            let (path, acct): (Option<String>, String) = stmt
                .query_row([outbox_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?;
            drop(stmt);

            if let Some(p) = path {
                let _ = fs::remove_file(p);
            }

            conn.execute("DELETE FROM outbox WHERE id = ?", [outbox_id])
                .map_err(|e| e.to_string())?;

            ((), acct)
        };

        self.emit_outbox_event("outbox:message:cancelled", outbox_id, &account_id, None)
            .await;

        Ok(())
    }
}
