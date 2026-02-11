use crate::error::AppError;
use crate::globals::DB_CONN;
use crate::imap::ImapManager;
use async_std::stream::StreamExt;

fn flag_to_string(flag: &async_imap::types::Flag) -> String {
    match flag {
        async_imap::types::Flag::Seen => "\\Seen".to_string(),
        async_imap::types::Flag::Answered => "\\Answered".to_string(),
        async_imap::types::Flag::Flagged => "\\Flagged".to_string(),
        async_imap::types::Flag::Deleted => "\\Deleted".to_string(),
        async_imap::types::Flag::Draft => "\\Draft".to_string(),
        async_imap::types::Flag::Recent => "\\Recent".to_string(),
        async_imap::types::Flag::MayCreate => "\\*".to_string(),
        async_imap::types::Flag::Custom(s) => s.to_string(),
    }
}

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

        {
            let mut store_stream = session
                .uid_store(&uid_set, &store_query)
                .await
                .map_err(AppError::from)?;

            // Consume the stream
            while let Some(_) = store_stream.next().await {}
        }

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
            let flags: Vec<String> = fetch.flags().map(|flag| flag_to_string(&flag)).collect();

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
            // Skip if this UID has pending or recently synced flag changes in the queue
            let recent_threshold = chrono::Utc::now().timestamp() - 10;
            let has_pending: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM flag_sync_queue
                     WHERE account_id = ? AND mailbox = ? AND uid = ?
                     AND (synced_at IS NULL OR synced_at > ?)",
                    rusqlite::params![account_id, mailbox, uid, recent_threshold],
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

            let flags_json =
                serde_json::to_string(&flags).map_err(|e| AppError::from(e.to_string()))?;

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

        // Cleanup old synced operations (older than 10 seconds)
        let deleted = crate::db::cleanup_old_synced_operations(conn)
            .map_err(|e| AppError::from(e.to_string()))?;

        if deleted > 0 {
            tracing::debug!(target: "postail",
                "[FLAG_SYNC] Cleaned up {} old synced queue items",
                deleted
            );
        }

        Ok(())
    }

    pub async fn move_messages_remote(
        &self,
        account_id: &str,
        source_mailbox: &str,
        target_mailbox: &str,
        uids: &[u32],
    ) -> Result<(), AppError> {
        if uids.is_empty() {
            return Ok(());
        }

        let mut session = self.connect_imap(account_id).await?;
        session.select(source_mailbox).await.map_err(AppError::from)?;

        let uid_set = format_uid_set(uids);

        tracing::info!(target: "postail",
            "[IMAP] Moving messages from {} to {}: {}",
            source_mailbox, target_mailbox, uid_set
        );

        // Try UID MOVE first (RFC 6851)
        match session.uid_mv(&uid_set, target_mailbox).await {
            Ok(_) => {
                tracing::info!(target: "postail",
                    "[IMAP] Successfully moved {} messages using UID MOVE", uids.len()
                );
            }
            Err(e) => {
                // Fallback: COPY + STORE \Deleted + EXPUNGE
                tracing::warn!(target: "postail",
                    "[IMAP] UID MOVE failed ({}), falling back to COPY+DELETE", e
                );

                session
                    .uid_copy(&uid_set, target_mailbox)
                    .await
                    .map_err(AppError::from)?;

                {
                    let mut store_stream = session
                        .uid_store(&uid_set, "+FLAGS.SILENT (\\Deleted)")
                        .await
                        .map_err(AppError::from)?;

                    while let Some(_) = store_stream.next().await {}
                }

                {
                    let mut expunge_stream = Box::pin(session.expunge().await.map_err(AppError::from)?);
                    while let Some(_) = expunge_stream.next().await {}
                }

                tracing::info!(target: "postail",
                    "[IMAP] Successfully moved {} messages using COPY+DELETE", uids.len()
                );
            }
        }

        session.logout().await.map_err(AppError::from)?;

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
