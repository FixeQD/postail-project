use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::time::Duration;

lazy_static::lazy_static! {
    static ref SYNC_TASKS: std::sync::Mutex<Vec<thread::JoinHandle<()>>> = std::sync::Mutex::new(Vec::new());
    static ref STOP_FLAGS: std::sync::Mutex<std::collections::HashMap<String, Arc<AtomicBool>>> = std::sync::Mutex::new(std::collections::HashMap::new());
}

impl crate::imap::ImapManager {
    pub fn start_sync(&self, account_id: &str) -> Result<(), String> {
        let manager = self.clone();
        let account_id = account_id.to_string();

        let stop_flag = Arc::new(AtomicBool::new(false));
        {
            let mut flags = STOP_FLAGS.lock().unwrap();
            flags.insert(account_id.clone(), Arc::clone(&stop_flag));
        }

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let sync_loop: Result<(), String> =
                    async { manager.account_sync_loop(&account_id, &stop_flag).await }.await;

                if let Err(e) = sync_loop {
                    eprintln!("Sync loop failed for {}: {}", account_id, e);
                }

                let mut flags = STOP_FLAGS.lock().unwrap();
                flags.remove(&account_id);
            });
        });

        let mut tasks = SYNC_TASKS.lock().unwrap();
        tasks.push(handle);
        Ok(())
    }

    pub fn stop_sync(&self, account_id: &str) -> Result<(), String> {
        let flags = STOP_FLAGS.lock().unwrap();
        if let Some(stop_flag) = flags.get(account_id) {
            stop_flag.store(true, Ordering::SeqCst);
            Ok(())
        } else {
            Err(format!("No sync running for account {}", account_id))
        }
    }

    async fn account_sync_loop(
        &self,
        account_id: &str,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), String> {
        loop {
            if stop_flag.load(Ordering::SeqCst) {
                return Ok(());
            }

            match self.sync_all_mailboxes(account_id, stop_flag).await {
                Ok(_) => {
                    eprintln!("[IMAP] Sync completed for {}, entering IDLE", account_id);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
                Err(e) => {
                    eprintln!(
                        "[IMAP] Sync error for {}: {}, retrying in 30s",
                        account_id, e
                    );
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            }
        }
    }

    async fn sync_all_mailboxes(
        &self,
        account_id: &str,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), String> {
        let mailboxes = self.fetch_mailboxes(account_id).await?;

        for mailbox in mailboxes {
            if stop_flag.load(Ordering::SeqCst) {
                return Ok(());
            }
            if let Err(e) = self
                .idle_mailbox(account_id, &mailbox.name, stop_flag)
                .await
            {
                eprintln!("[IMAP] Mailbox error for {}: {}", mailbox.name, e);
            }
        }

        Ok(())
    }

    async fn idle_mailbox(
        &self,
        account_id: &str,
        mailbox_name: &str,
        stop_flag: &Arc<AtomicBool>,
    ) -> Result<(), String> {
        let mut session = self.connect_imap(account_id).await?;
        let mailbox = session
            .select(mailbox_name)
            .await
            .map_err(|e| e.to_string())?;

        let uid_validity = mailbox.uid_validity.unwrap_or(0);
        let current_uid = mailbox.exists;

        self.check_uidvalidity(account_id, mailbox_name, uid_validity)
            .await?;

        let mut last_uid = self.get_last_synced_uid(account_id, mailbox_name).await?;

        if current_uid > last_uid {
            self.fetch_missing_messages(account_id, mailbox_name, last_uid + 1, current_uid)
                .await?;
            last_uid = current_uid;
        }

        eprintln!("[IMAP] Entering IDLE for {}@{}", mailbox_name, account_id);

        loop {
            if stop_flag.load(Ordering::SeqCst) {
                let _ = session.logout().await;
                return Ok(());
            }

            session = self.wait_for_idle(session).await?;
            let mailbox = session
                .select(mailbox_name)
                .await
                .map_err(|e| e.to_string())?;
            let new_uid_count = mailbox.exists;
            if new_uid_count > last_uid {
                self.fetch_missing_messages(account_id, mailbox_name, last_uid + 1, new_uid_count)
                    .await?;
                last_uid = new_uid_count;
            }
        }
    }

    async fn wait_for_idle(
        &self,
        session: async_imap::Session<async_native_tls::TlsStream<async_std::net::TcpStream>>,
    ) -> Result<async_imap::Session<async_native_tls::TlsStream<async_std::net::TcpStream>>, String>
    {
        let mut idle = session.idle();
        idle.init().await.map_err(|e| e.to_string())?;

        loop {
            let (wait_future, _interrupt) = idle.wait();
            match wait_future.await {
                Ok(_) => {
                    let session = idle.done().await.map_err(|e| e.to_string())?;
                    return Ok(session);
                }
                Err(e) => {
                    return Err(format!("IDLE wait error: {}", e));
                }
            }
        }
    }

    async fn get_last_synced_uid(
        &self,
        account_id: &str,
        mailbox_name: &str,
    ) -> Result<u32, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT last_synced_uid FROM mailboxes WHERE account_id = ? AND name = ?")
            .map_err(|e| e.to_string())?;
        let last_uid: Option<i64> = stmt
            .query_row([account_id, mailbox_name], |row| row.get(0))
            .ok();
        Ok(last_uid.unwrap_or(0) as u32)
    }

    async fn fetch_missing_messages(
        &self,
        account_id: &str,
        mailbox_name: &str,
        start_uid: u32,
        end_uid: u32,
    ) -> Result<(), String> {
        if start_uid >= end_uid {
            return Ok(());
        }

        let limit: u32 = 100;
        let mut anchor = start_uid;
        let mut latest_uid = start_uid;

        while anchor < end_uid {
            let headers = self
                .fetch_headers(account_id, mailbox_name, Some(anchor), limit)
                .await?;

            if headers.is_empty() {
                break;
            }

            if let Some(h) = headers.last() {
                latest_uid = h.uid;
            }

            anchor = latest_uid + 1;

            if headers.len() < limit as usize {
                break;
            }
        }

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("UPDATE mailboxes SET last_synced_uid = ? WHERE account_id = ? AND name = ?")
            .map_err(|e| e.to_string())?;
        stmt.execute([
            end_uid.to_string(),
            account_id.to_string(),
            mailbox_name.to_string(),
        ])
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}
