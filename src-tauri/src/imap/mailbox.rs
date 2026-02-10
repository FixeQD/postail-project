use crate::imap::ImapManager;
use rusqlite::params;
use tracing;

impl ImapManager {
    pub async fn check_uidvalidity(
        &self,
        account_id: &str,
        mailbox_name: &str,
        server_uidvalidity: u32,
    ) -> Result<(), String> {
        let db_uidvalidity: Option<u32> = {
            let conn_guard = self.conn.lock().unwrap();
            let conn = conn_guard
                .as_ref()
                .ok_or("Database not initialized".to_string())?;
            let mut stmt = conn
                .prepare("SELECT uid_validity FROM mailboxes WHERE account_id = ? AND name = ?")
                .map_err(|e| e.to_string())?;
            let result: Option<i64> = stmt
                .query_row([account_id, mailbox_name], |row| row.get(0))
                .ok();
            result.map(|v| v as u32)
        };

        match db_uidvalidity {
            Some(db_uidv) if db_uidv == server_uidvalidity => {
                tracing::debug!(target: "postail", "[IMAP] UIDVALIDITY match for {}", mailbox_name);
                Ok(())
            }
            Some(db_uidv) if db_uidv != server_uidvalidity => {
                tracing::warn!(target: "postail",
                    "[IMAP] UIDVALIDITY mismatch for {}: DB={}, Server={}. Triggering full resync.",
                    mailbox_name, db_uidv, server_uidvalidity
                );
                self.full_resync_mailbox(account_id, mailbox_name, server_uidvalidity)
                    .await
            }
            None => {
                tracing::info!(target: "postail",
                    "[IMAP] No UIDVALIDITY stored for {}, creating mailbox entry",
                    mailbox_name
                );
                let conn_guard = self.conn.lock().unwrap();
                let conn = conn_guard
                    .as_ref()
                    .ok_or("Database not initialized".to_string())?;
                let mut stmt = conn
                    .prepare(
                        "INSERT INTO mailboxes (account_id, name, uid_validity, last_synced_uid) VALUES (?, ?, ?, 0)
                         ON CONFLICT(account_id, name) DO UPDATE SET
                            uid_validity = excluded.uid_validity,
                            last_synced_uid = CASE 
                                WHEN mailboxes.uid_validity != excluded.uid_validity THEN 0 
                                ELSE mailboxes.last_synced_uid 
                            END",
                    )
                    .map_err(|e| e.to_string())?;
                stmt.execute([account_id, mailbox_name, &server_uidvalidity.to_string()])
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            Some(_) => Ok(()),
        }
    }

    async fn full_resync_mailbox(
        &self,
        account_id: &str,
        mailbox_name: &str,
        uid_validity: u32,
    ) -> Result<(), String> {
        tracing::info!(target: "postail", "[IMAP] Performing full resync for {}", mailbox_name);

        let mut conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_mut()
            .ok_or("Database not initialized".to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "DELETE FROM messages WHERE account_id = ? AND mailbox = ?",
            [account_id, mailbox_name],
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE mailboxes SET uid_validity = ?, last_synced_uid = 0 WHERE account_id = ? AND name = ?",
            [&uid_validity.to_string(), account_id, mailbox_name],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_mailbox_metadata(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<Option<MailboxMetadata>, String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT uid_validity, highest_modseq, last_synced_uid FROM mailboxes WHERE account_id = ? AND name = ?",
            )
            .map_err(|e| e.to_string())?;
        let result: Result<MailboxMetadata, rusqlite::Error> =
            stmt.query_row([account_id, mailbox_name], |row| {
                Ok(MailboxMetadata {
                    uid_validity: row.get::<_, Option<i64>>(0)?.unwrap_or(0) as u32,
                    highest_modseq: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    last_synced_uid: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u32,
                })
            });
        drop(stmt);
        drop(conn_guard);
        match result.map_err(|e| e.to_string()) {
            Ok(metadata) => Ok(Some(metadata)),
            Err(_) => Ok(None),
        }
    }

    pub async fn update_highest_modseq(
        &self,
        account_id: &str,
        mailbox_name: &str,
        modseq: i64,
    ) -> Result<(), String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        let mut stmt = conn
            .prepare(
                "UPDATE mailboxes SET highest_modseq = ? WHERE account_id = ? AND name = ? AND highest_modseq < ?",
            )
            .map_err(|e| e.to_string())?;
        stmt.execute(params![modseq, account_id, mailbox_name, modseq])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub struct MailboxMetadata {
    pub uid_validity: u32,
    pub highest_modseq: i64,
    pub last_synced_uid: u32,
}
