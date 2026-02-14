use std::fs;

use uuid::Uuid;

use crate::db::{enqueue_message, list_outbox, OutboxItem};

impl super::SmtpManager {
    pub async fn enqueue_message(
        &self,
        account_id: &str,
        raw_eml: &[u8],
    ) -> Result<String, String> {
        tracing::info!(target: "postail", "[Outbox] Enqueueing message for account: {}, size: {} bytes", account_id, raw_eml.len());

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
        tracing::info!(target: "postail", "[Outbox] EML file written successfully");

        let conn_guard = self.conn.lock().await;
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;

        match enqueue_message(conn, account_id, &eml_path.to_string_lossy()) {
            Ok(id) => {
                tracing::info!(target: "postail", "[Outbox] Message enqueued successfully with ID: {}", id);
                Ok(id)
            }
            Err(e) => {
                tracing::error!(target: "postail", "[Outbox] Failed to enqueue message in DB: {}", e);
                Err(e.to_string())
            }
        }
    }

    pub async fn list_outbox(&self, account_id: &str) -> Result<Vec<OutboxItem>, String> {
        let conn_guard = self.conn.lock().await;
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        list_outbox(conn, account_id).map_err(|e| e.to_string())
    }

    pub async fn retry_sending(&self, outbox_id: &str) -> Result<(), String> {
        let conn_guard = self.conn.lock().await;
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        conn.execute(
            "UPDATE outbox SET status = 'PENDING', next_retry = NULL WHERE id = ?",
            [outbox_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn cancel_sending(&self, outbox_id: &str) -> Result<(), String> {
        let conn_guard = self.conn.lock().await;
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        let mut stmt = conn
            .prepare("SELECT raw_eml_path FROM outbox WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let eml_path: Option<String> = stmt.query_row([outbox_id], |row| row.get(0)).ok();
        drop(stmt);
        if let Some(path) = eml_path {
            let _ = fs::remove_file(path);
        }
        conn.execute("DELETE FROM outbox WHERE id = ?", [outbox_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
