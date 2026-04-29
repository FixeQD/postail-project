use crate::error::AppError;
use crate::globals::get_db_pool;
use crate::imap::sync_status::update_sync_status;

impl crate::imap::ImapManager {
    pub(crate) async fn get_last_synced_uid(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<u32, AppError> {
        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT last_synced_uid FROM mailboxes WHERE account_id = ? AND name = ?")
            .map_err(|e| AppError::from(e.to_string()))?;
        let last_uid: Option<i64> = stmt
            .query_row([account_id, mailbox_name], |row| row.get(0))
            .ok();
        Ok(last_uid.unwrap_or(0) as u32)
    }

    pub(crate) async fn fetch_missing_messages(
        &self,
        account_id: &str,
        mailbox_name: &str,
        start_uid: u32,
        end_uid: u32,
    ) -> Result<(u32, Option<String>, Option<String>), AppError> {
        if start_uid > end_uid {
            return Ok((0, None, None));
        }

        let total = end_uid.saturating_sub(start_uid).saturating_add(1);
        let limit: u32 = 100;
        let mut anchor = start_uid;
        let mut latest_uid = start_uid;
        let mut processed = 0u32;
        let mut actual_new_count = 0u32;
        let mut newest_subject: Option<String> = None;
        let mut newest_sender: Option<String> = None;

        while anchor <= end_uid {
            update_sync_status(account_id, mailbox_name, processed, total).await;

            let headers = self
                .fetch_headers(account_id, mailbox_name, Some(anchor), limit)
                .await?;

            if headers.is_empty() {
                break;
            }

            for h in &headers {
                if !h.flags.iter().any(|f| f.eq_ignore_ascii_case("\\Seen")) {
                    actual_new_count += 1;
                }
            }

            if let Some(h) = headers.last() {
                latest_uid = h.uid;
                newest_subject = h.subject.clone();
                newest_sender = h.from.first().cloned();
            }

            processed += headers.len() as u32;
            anchor = latest_uid.saturating_add(1);

            if headers.len() < limit as usize {
                break;
            }
        }

        update_sync_status(account_id, mailbox_name, total, total).await;

        let pool = get_db_pool()
            .await
            .map_err(|e| AppError::from(e.to_string()))?;
        let conn = pool.get().map_err(|e| AppError::from(e.to_string()))?;
        conn.execute(
            "UPDATE mailboxes SET last_synced_uid = ? WHERE account_id = ? AND name = ?",
            rusqlite::params![end_uid, account_id, mailbox_name],
        )
        .map_err(|e| AppError::from(e.to_string()))?;

        Ok((actual_new_count, newest_subject, newest_sender))
    }
}
