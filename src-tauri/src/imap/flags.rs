use crate::error::AppError;
use crate::globals::DB_CONN;
use crate::imap::ImapManager;
use async_std::stream::StreamExt;

pub fn format_uid_set(uids: &[u32]) -> String {
    if uids.is_empty() {
        return String::new();
    }

    let mut sorted_uids = uids.to_vec();
    sorted_uids.sort_unstable();
    sorted_uids.dedup();

    let mut result = Vec::new();
    let mut start = sorted_uids[0];
    let mut end = sorted_uids[0];

    for &uid in &sorted_uids[1..] {
        if uid == end + 1 {
            end = uid;
        } else {
            if start == end {
                result.push(start.to_string());
            } else {
                result.push(format!("{}:{}", start, end));
            }
            start = uid;
            end = uid;
        }
    }

    if start == end {
        result.push(start.to_string());
    } else {
        result.push(format!("{}:{}", start, end));
    }

    result.join(",")
}

impl ImapManager {
    pub async fn set_flags_remote(
        &self,
        account_id: &str,
        mailbox: &str,
        uids: &[u32],
        operation: &str,
        flags: &[String],
    ) -> Result<(), AppError> {
        if uids.is_empty() {
            return Ok(());
        }

        let mut session = self.connect_imap(account_id).await?;
        session.select(mailbox).await.map_err(AppError::from)?;

        let uid_set = format_uid_set(uids);
        let flag_list = flags
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let store_query = match operation {
            "add" => format!("+FLAGS.SILENT ({})", flag_list),
            "remove" => format!("-FLAGS.SILENT ({})", flag_list),
            "set" => format!("FLAGS.SILENT ({})", flag_list),
            _ => return Err(AppError::from(format!("Unknown operation: {}", operation))),
        };

        tracing::debug!(target: "postail",
            "[IMAP] Setting flags on {}@{}: {} {}",
            mailbox, account_id, uid_set, store_query
        );

        session
            .uid_store(&uid_set, &store_query)
            .await
            .map_err(AppError::from)?;

        session.logout().await.map_err(AppError::from)?;

        Ok(())
    }

    pub async fn sync_flags_from_server(
        &self,
        account_id: &str,
        mailbox: &str,
        uid_range: Option<(u32, u32)>,
    ) -> Result<(), AppError> {
        let mut session = self.connect_imap(account_id).await?;
        session.select(mailbox).await.map_err(AppError::from)?;

        let uid_set = if let Some((start, end)) = uid_range {
            format!("{}:{}", start, end)
        } else {
            "1:*".to_string()
        };

        tracing::debug!(target: "postail",
            "[IMAP] Syncing flags from server for {}@{}: {}",
            mailbox, account_id, uid_set
        );

        let mut fetches = session
            .uid_fetch(uid_set, "FLAGS")
            .await
            .map_err(AppError::from)?;

        let mut flag_updates = Vec::new();

        while let Some(fetch) = fetches.next().await {
            let fetch = fetch.map_err(AppError::from)?;
            let uid = fetch.uid.ok_or_else(|| AppError::from("No UID"))?;
            let flags: Vec<String> = fetch
                .flags()
                .map(|flag| format!("{:?}", flag))
                .collect();

            flag_updates.push((uid, flags));
        }

        drop(fetches);
        session.logout().await.map_err(AppError::from)?;

        let conn_guard = DB_CONN.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AppError::from("Database not initialized"))?;

        let update_count = flag_updates.len();

        for (uid, flags) in flag_updates {
            // Skip if this UID has pending flag changes in the queue
            let has_pending: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM flag_sync_queue 
                     WHERE account_id = ? AND mailbox = ? AND uid = ?",
                    rusqlite::params![account_id, mailbox, uid],
                    |row| row.get::<_, i64>(0),
                )
                .map(|count| count > 0)
                .unwrap_or(false);

            if has_pending {
                tracing::debug!(target: "postail",
                    "[IMAP] Skipping flag sync for uid={} (pending queue item)",
                    uid
                );
                continue;
            }

            let flags_json = serde_json::to_string(&flags)
                .map_err(|e| AppError::from(e.to_string()))?;

            let rows_updated = conn.execute(
                "UPDATE messages SET flags_json = ? WHERE account_id = ? AND mailbox = ? AND uid = ?",
                rusqlite::params![flags_json, account_id, mailbox, uid],
            )
            .map_err(|e| AppError::from(e.to_string()))?;

            if rows_updated == 0 {
                tracing::debug!(target: "postail",
                    "[IMAP] Flag sync: message not found in DB: uid={} mailbox={} account={}",
                    uid, mailbox, account_id
                );
            }
        }

        tracing::debug!(target: "postail",
            "[IMAP] Synced {} flag updates for {}@{}",
            update_count, mailbox, account_id
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uid_set() {
        assert_eq!(format_uid_set(&[1]), "1");
        assert_eq!(format_uid_set(&[1, 2, 3]), "1:3");
        assert_eq!(format_uid_set(&[1, 2, 5, 6, 7, 10]), "1:2,5:7,10");
        assert_eq!(format_uid_set(&[1, 3, 5, 7]), "1,3,5,7");
        assert_eq!(format_uid_set(&[5, 1, 3, 2, 4]), "1:5");
        assert_eq!(format_uid_set(&[]), "");
    }
}

