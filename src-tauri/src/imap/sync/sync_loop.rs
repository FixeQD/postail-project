use std::thread;
use tokio::time::Duration;

struct SyncTask {
    account_id: String,
    manager: crate::imap::ImapManager,
}

lazy_static::lazy_static! {
    static ref SYNC_TASKS: std::sync::Mutex<Vec<SyncTask>> = std::sync::Mutex::new(Vec::new());
}

impl crate::imap::ImapManager {
    pub fn start_sync(&self, account_id: &str) -> Result<(), String> {
        let manager = self.clone();
        let account_id = account_id.to_string();

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let sync_loop: Result<(), String> = async {
                    loop {
                        match manager.sync_account(&account_id).await {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Sync error for account {}: {}", account_id, e);
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(300)).await;
                    }
                }
                .await;

                if let Err(e) = sync_loop {
                    eprintln!("Sync loop failed: {}", e);
                }
            });
        });

        Ok(())
    }

    async fn sync_account(&self, account_id: &str) -> Result<(), String> {
        let mailboxes = self.fetch_mailboxes(account_id).await?;

        for mailbox in mailboxes {
            self.sync_mailbox(account_id, &mailbox.name).await?;
        }

        Ok(())
    }

    async fn sync_mailbox(&self, account_id: &str, mailbox_name: &str) -> Result<(), String> {
        let mut session = self.connect_imap(account_id).await?;
        session
            .select(mailbox_name)
            .await
            .map_err(|e| e.to_string())?;

        let last_synced: Option<u32> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT last_synced_uid FROM mailboxes WHERE account_id = ? AND name = ?")
                .map_err(|e| e.to_string())?;
            stmt.query_row([account_id, mailbox_name], |row| row.get(0))
                .ok()
        };

        let search_result = session
            .search(format!("UID {}:*", last_synced.unwrap_or(0)))
            .await
            .map_err(|e| e.to_string())?;

        let max_uid: u32 = search_result.iter().max().copied().unwrap_or(0);

        if max_uid > last_synced.unwrap_or(0) {
            let limit: u32 = 100;
            let mut anchor: u32 = last_synced.unwrap_or(0);
            let mut latest_uid: u32 = anchor;
            loop {
                let headers = self
                    .fetch_headers(account_id, mailbox_name, Some(anchor), limit)
                    .await?;
                if headers.is_empty() {
                    break;
                }
                if let Some(h) = headers.last() {
                    latest_uid = h.uid;
                }
                if headers.len() < limit as usize {
                    break;
                }
                anchor = latest_uid;
            }

            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare(
                    "UPDATE mailboxes SET last_synced_uid = ? WHERE account_id = ? AND name = ?",
                )
                .map_err(|e| e.to_string())?;
            stmt.execute([
                latest_uid.to_string(),
                account_id.to_string(),
                mailbox_name.to_string(),
            ])
            .map_err(|e| e.to_string())?;
        }

        session.logout().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
