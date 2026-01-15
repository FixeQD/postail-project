use std::fs;

use uuid::Uuid;

use crate::db::{enqueue_message, extract_headers_from_eml, list_outbox, OutboxItem};

impl super::SmtpManager {
    pub fn enqueue_message(&self, account_id: &str, raw_eml: &[u8]) -> Result<String, String> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("postail")
            .join("outbox");
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        let eml_path = data_dir.join(format!("{}.eml", Uuid::new_v4()));

        let security = self.security.lock().unwrap();
        let encrypted_eml = security.encrypt(raw_eml).map_err(|e| e.to_string())?;
        drop(security);
        fs::write(&eml_path, encrypted_eml).map_err(|e| e.to_string())?;

        let (_subject, _recipient) =
            extract_headers_from_eml(&eml_path.to_string_lossy()).map_err(|e| e.to_string())?;

        let conn = self.conn.lock().unwrap();
        let id = enqueue_message(&conn, account_id, &eml_path.to_string_lossy())
            .map_err(|e| e.to_string());
        drop(conn);
        id
    }

    pub fn list_outbox(&self, account_id: &str) -> Result<Vec<OutboxItem>, String> {
        let conn = self.conn.lock().unwrap();
        list_outbox(&conn, account_id).map_err(|e| e.to_string())
    }

    pub fn retry_sending(&self, outbox_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE outbox SET status = 'PENDING', next_retry = NULL WHERE id = ?",
            [outbox_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn cancel_sending(&self, outbox_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
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

    pub fn start_outbox_worker(&self) {}
}
