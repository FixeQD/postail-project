use crate::error::AppError;
use crate::globals::IMAP_MANAGER;
use crate::oauth::{ProviderInfo, ProviderKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinHandle;
use tracing;

const MAX_DEDUP_ENTRIES: usize = 10000;
const DEDUP_EVICTION_BATCH: usize = 2000;
/// Max concurrent IMAP connections during a poll sweep.
const MAX_CONCURRENT_POLLS: usize = 5;

pub type WatchKey = (String, String);

/// Watch mode for a mailbox.
#[derive(Debug)]
pub enum WatchMode {
    /// Full IDLE mode with a dedicated IMAP connection.
    Idle {
        stop_flag: Arc<AtomicBool>,
        task_handle: JoinHandle<()>,
    },
    /// Polling mode - checked periodically by the polling worker.
    Polling { last_check: Instant },
    /// Waiting for an available IDLE slot.
    Queued,
}

#[derive(Debug)]
pub struct MailboxWatch {
    pub account_id: String,
    pub mailbox: String,
    pub mode: WatchMode,
    pub last_activity: Instant,
    pub provider_kind: ProviderKind,
    pub is_virtual: bool,
}

#[derive(Debug, Clone)]
struct DedupEntry {
    mailboxes: Vec<(String, String)>,
    timestamp: Instant,
}

pub struct ConnectionPool {
    idle_watches: HashMap<WatchKey, MailboxWatch>,
    poll_queue: VecDeque<WatchKey>,
    watched_keys: HashSet<WatchKey>,
    dedup_tracker: HashMap<String, DedupEntry>,
    provider_cache: HashMap<String, ProviderKind>,
    polling_task: Option<JoinHandle<()>>,
    rebalance_task: Option<JoinHandle<()>>,
    cleanup_task: Option<JoinHandle<()>>,
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {
            idle_watches: HashMap::new(),
            poll_queue: VecDeque::new(),
            watched_keys: HashSet::new(),
            dedup_tracker: HashMap::new(),
            provider_cache: HashMap::new(),
            polling_task: None,
            rebalance_task: None,
            cleanup_task: None,
        }
    }

    pub fn start_workers(&mut self) {
        for task in [
            self.polling_task.take(),
            self.rebalance_task.take(),
            self.cleanup_task.take(),
        ]
        .into_iter()
        .flatten()
        {
            task.abort();
        }

        self.polling_task = Some(tokio::spawn(polling_worker()));
        self.rebalance_task = Some(tokio::spawn(rebalance_worker()));
        self.cleanup_task = Some(tokio::spawn(cleanup_worker()));
    }

    pub async fn stop_workers(&mut self) {
        for task in [
            self.polling_task.take(),
            self.rebalance_task.take(),
            self.cleanup_task.take(),
        ]
        .into_iter()
        .flatten()
        {
            task.abort();
        }
    }

    pub async fn watch_mailbox(&mut self, account_id: &str, mailbox: &str) -> Result<(), AppError> {
        let key = (account_id.to_string(), mailbox.to_string());

        if self.watched_keys.contains(&key) {
            tracing::debug!(target: "postail", "[Pool] Already watching {}@{}", mailbox, account_id);
            return Ok(());
        }

        tracing::info!(target: "postail", "[Pool] Starting watch for {}@{}", mailbox, account_id);

        let provider_kind = self.detect_provider(account_id).await?;
        let provider = ProviderInfo::get(provider_kind);

        let mailbox_role = self.get_mailbox_role(account_id, mailbox).await;
        let is_virtual = mailbox_role.as_deref() == Some("all");

        let account_idle_count = self.count_account_idle_connections(account_id);
        let max_idle = provider.max_idle_connections;

        if account_idle_count < max_idle {
            tracing::info!(target: "postail",
                "[Pool] Starting IDLE for {}@{} (slot {}/{})",
                mailbox, account_id, account_idle_count + 1, max_idle
            );
            self.start_idle_watch(account_id, mailbox, provider_kind, is_virtual)
                .await?;
        } else {
            tracing::info!(target: "postail",
                "[Pool] Adding {}@{} to polling queue (slots full {}/{})",
                mailbox, account_id, account_idle_count, max_idle
            );
            self.poll_queue.push_back(key.clone());
        }

        self.watched_keys.insert(key);
        Ok(())
    }

    pub async fn unwatch_mailbox(&mut self, account_id: &str, mailbox: &str) {
        let key = (account_id.to_string(), mailbox.to_string());

        if !self.watched_keys.contains(&key) {
            return;
        }

        tracing::info!(target: "postail", "[Pool] Stopping watch for {}@{}", mailbox, account_id);

        if let Some(watch) = self.idle_watches.remove(&key) {
            self.stop_idle_watch(watch).await;
            self.promote_from_queue().await;
        }

        self.poll_queue.retain(|k| k != &key);
        self.watched_keys.remove(&key);
    }

    pub async fn unwatch_all_for_account(&mut self, account_id: &str) {
        tracing::info!(target: "postail", "[Pool] Stopping all watches for account {}", account_id);

        let keys_to_remove: Vec<_> = self
            .watched_keys
            .iter()
            .filter(|(acc, _)| acc == account_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.unwatch_mailbox(&key.0, &key.1).await;
        }

        self.provider_cache.remove(account_id);
    }

    pub async fn record_activity(&mut self, account_id: &str, mailbox: &str) {
        let key = (account_id.to_string(), mailbox.to_string());
        if let Some(watch) = self.idle_watches.get_mut(&key) {
            watch.last_activity = Instant::now();
        }
    }

    pub async fn rebalance(&mut self) {
        tracing::debug!(target: "postail", "[Pool] Starting rebalance");

        // Find stale IDLE watches.
        let mut stale_keys: Vec<WatchKey> = Vec::new();
        for (key, watch) in &self.idle_watches {
            let provider = ProviderInfo::get(watch.provider_kind);
            if watch.last_activity.elapsed() > Duration::from_secs(provider.stale_threshold_seconds)
            {
                stale_keys.push(key.clone());
            }
        }

        // Demote stale watches to polling queue.
        for key in stale_keys {
            tracing::info!(target: "postail", "[Pool] Demoting {}@{} to polling (stale)", key.1, key.0);
            if let Some(watch) = self.idle_watches.remove(&key) {
                self.stop_idle_watch(watch).await;
                self.poll_queue.push_back(key);
            }
        }

        // Promote queued mailboxes if IDLE slots freed up.
        let hot_keys: Vec<(WatchKey, ProviderKind)> = self
            .poll_queue
            .iter()
            .filter_map(|(acc, mb)| {
                let provider_kind = self
                    .provider_cache
                    .get(acc)
                    .copied()
                    .unwrap_or(ProviderKind::Gmail);
                let provider = ProviderInfo::get(provider_kind);
                if self.count_account_idle_connections(acc) < provider.max_idle_connections {
                    Some(((acc.clone(), mb.clone()), provider_kind))
                } else {
                    None
                }
            })
            .collect();

        for (key, provider_kind) in hot_keys {
            self.poll_queue.retain(|k| k != &key);
            let is_virtual = self.get_mailbox_role_sync(&key.0, &key.1);
            tracing::info!(target: "postail", "[Pool] Promoting {}@{} to IDLE", key.1, key.0);
            if let Err(e) = self
                .start_idle_watch(&key.0, &key.1, provider_kind, is_virtual)
                .await
            {
                tracing::error!(target: "postail",
                    "[Pool] Failed to promote {}@{}: {}",
                    key.1, key.0, e
                );
                self.poll_queue.push_back(key);
            }
        }

        tracing::debug!(target: "postail",
            "[Pool] Rebalance complete: {} IDLE, {} polling",
            self.idle_watches.len(),
            self.poll_queue.len()
        );
    }

    pub async fn poll_all(&mut self) {
        let keys: Vec<WatchKey> = self.poll_queue.iter().cloned().collect();

        // Semaphore is shared across all spawned tasks to cap concurrent IMAP connections.
        static POLL_SEMAPHORE: LazyLock<Arc<Semaphore>> =
            LazyLock::new(|| Arc::new(Semaphore::new(MAX_CONCURRENT_POLLS)));

        for (account_id, mailbox) in keys {
            let sem = POLL_SEMAPHORE.clone();
            tokio::spawn(async move {
                let _permit = match sem.acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => return,
                };
                if let Err(e) = Self::quick_poll(&account_id, &mailbox).await {
                    tracing::warn!(target: "postail",
                        "[Pool] Poll failed for {}@{}: {}",
                        mailbox, account_id, e
                    );
                }
            });
        }
    }

    pub fn cleanup_dedup(&mut self) {
        let cutoff = Duration::from_secs(24 * 60 * 60);
        let now = Instant::now();

        let to_remove: Vec<String> = self
            .dedup_tracker
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.timestamp) > cutoff)
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_remove.len();
        for key in to_remove {
            self.dedup_tracker.remove(&key);
        }

        if count > 0 {
            tracing::debug!(target: "postail",
                "[Pool] Cleaned up {} old dedup entries",
                count
            );
        }
    }

    async fn start_idle_watch(
        &mut self,
        account_id: &str,
        mailbox: &str,
        provider_kind: ProviderKind,
        is_virtual: bool,
    ) -> Result<(), AppError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let account_id_owned = account_id.to_string();
        let mailbox_owned = mailbox.to_string();

        let task = tokio::spawn(async move {
            if let Err(e) =
                Self::idle_loop(&account_id_owned, &mailbox_owned, stop_flag_clone).await
            {
                tracing::error!(target: "postail",
                    "[Pool] IDLE loop exited with error for {}@{}: {}",
                    mailbox_owned, account_id_owned, e
                );
            }
        });

        let watch = MailboxWatch {
            account_id: account_id.to_string(),
            mailbox: mailbox.to_string(),
            mode: WatchMode::Idle {
                stop_flag,
                task_handle: task,
            },
            last_activity: Instant::now(),
            provider_kind,
            is_virtual,
        };

        self.idle_watches
            .insert((account_id.to_string(), mailbox.to_string()), watch);
        Ok(())
    }

    async fn stop_idle_watch(&mut self, watch: MailboxWatch) {
        if let WatchMode::Idle {
            stop_flag,
            task_handle,
        } = watch.mode
        {
            stop_flag.store(true, Ordering::SeqCst);

            // Wake the IMAP IDLE wait immediately via the existing interrupt maps.
            let manager = {
                let guard = IMAP_MANAGER.lock().await;
                guard.clone()
            };
            manager.force_idle_wakeup(&watch.account_id).await;

            tokio::select! {
                _ = task_handle => {}
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    tracing::warn!(target: "postail",
                        "[Pool] IDLE task did not exit within 5s for {}@{}",
                        watch.mailbox, watch.account_id
                    );
                }
            }
        }
    }

    async fn promote_from_queue(&mut self) {
        if let Some(key) = self.poll_queue.pop_front() {
            if let Ok(provider_kind) = self.detect_provider(&key.0).await {
                let is_virtual = self.get_mailbox_role_sync(&key.0, &key.1);
                tracing::info!(target: "postail", "[Pool] Promoting {}@{} to IDLE (from queue)", key.1, key.0);
                let _ = self
                    .start_idle_watch(&key.0, &key.1, provider_kind, is_virtual)
                    .await;
            }
        }
    }

    fn count_account_idle_connections(&self, account_id: &str) -> usize {
        self.idle_watches
            .iter()
            .filter(|((acc, _), _)| acc == account_id)
            .count()
    }

    async fn detect_provider(&mut self, account_id: &str) -> Result<ProviderKind, AppError> {
        // Return cached value if available.
        if let Some(&kind) = self.provider_cache.get(account_id) {
            return Ok(kind);
        }

        let kind = if let Ok(pool) = crate::globals::get_db_pool().await {
            if let Ok(conn) = pool.get() {
                match conn
                    .prepare("SELECT provider_type FROM accounts WHERE id = ?")
                    .and_then(|mut stmt| {
                        stmt.query_row([account_id], |row| row.get::<_, String>(0))
                    }) {
                    Ok(provider_type) => {
                        ProviderKind::parse(&provider_type).unwrap_or(ProviderKind::Gmail)
                    }
                    Err(_) => ProviderKind::Gmail,
                }
            } else {
                ProviderKind::Gmail
            }
        } else {
            ProviderKind::Gmail
        };

        self.provider_cache.insert(account_id.to_string(), kind);
        Ok(kind)
    }

    async fn get_mailbox_role(&self, account_id: &str, mailbox: &str) -> Option<String> {
        let pool = crate::globals::get_db_pool().await.ok()?;
        let conn = pool.get().ok()?;
        conn.query_row(
            "SELECT role FROM mailboxes WHERE account_id = ? AND name = ?",
            [account_id, mailbox],
            |row| row.get::<_, String>(0),
        )
        .ok()
    }

    /// Synchronous role guess using only already-known state
    fn get_mailbox_role_sync(&self, account_id: &str, mailbox: &str) -> bool {
        // Check the idle_watches map first - we might already know.
        let key = (account_id.to_string(), mailbox.to_string());
        if let Some(watch) = self.idle_watches.get(&key) {
            return watch.is_virtual;
        }
        // Fallback heuristic for common "All Mail" names.
        mailbox.contains("All Mail") || mailbox.contains("Wszystkie")
    }

    /// Connects and delegates to `ImapManager::idle_mailbox`, which uses IMAP IDLE (not polling)
    /// Retries with backoff on error.
    async fn idle_loop(
        account_id: &str,
        mailbox: &str,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<(), AppError> {
        // Clone the manager once upfront; the lock is held only during clone.
        let manager = {
            let guard = IMAP_MANAGER.lock().await;
            guard.clone()
        };

        let mut backoff_secs = 5u64;

        loop {
            if stop_flag.load(Ordering::Acquire) {
                tracing::info!(target: "postail",
                    "[Pool] IDLE stop requested for {}@{}, exiting loop",
                    mailbox, account_id
                );
                break;
            }

            match manager.idle_mailbox(account_id, mailbox, &stop_flag).await {
                Ok(()) => {
                    // idle_mailbox exited cleanly (stop flag set).
                    tracing::info!(target: "postail",
                        "[Pool] IDLE ended cleanly for {}@{}",
                        mailbox, account_id
                    );
                    break;
                }
                Err(e) => {
                    if stop_flag.load(Ordering::Acquire) {
                        break;
                    }
                    tracing::error!(target: "postail",
                        "[Pool] IDLE error for {}@{}: {}. Retrying in {}s",
                        mailbox, account_id, e, backoff_secs
                    );
                    // Interruptible backoff sleep.
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(backoff_secs)) => {}
                        // The stop mechanism wakes via force_idle_wakeup -> interrupt_idle/interrupt_poll - recheck stop_flag at the top of the next iteration
                        _ = tokio::time::sleep(Duration::from_millis(200)) => {
                            if stop_flag.load(Ordering::Acquire) {
                                break;
                            }
                            // Not stopped, continue sleeping
                        }
                    }
                    // Exponential backoff capped at 5 minutes.
                    backoff_secs = (backoff_secs * 2).min(300);
                }
            }
        }

        Ok(())
    }

    async fn quick_poll(account_id: &str, mailbox: &str) -> Result<(), AppError> {
        let manager = {
            let guard = IMAP_MANAGER.lock().await;
            guard.clone()
        };

        tracing::debug!(target: "postail", "[Pool] Polling {}@{}", mailbox, account_id);
        manager
            .sync_single_mailbox_messages(account_id, mailbox)
            .await?;
        Ok(())
    }

    pub fn is_duplicate(&self, account_id: &str, mailbox: &str, message_id: &str) -> bool {
        if let Some(entry) = self.dedup_tracker.get(message_id) {
            for (acc, mb) in &entry.mailboxes {
                if acc == account_id && mb != mailbox {
                    return true;
                }
            }
        }
        false
    }

    pub fn track_message(
        &mut self,
        account_id: &str,
        mailbox: &str,
        message_id: String,
        is_virtual: bool,
    ) {
        if is_virtual {
            return;
        }

        if self.dedup_tracker.len() >= MAX_DEDUP_ENTRIES {
            self.evict_oldest_entries();
        }

        let entry = self
            .dedup_tracker
            .entry(message_id)
            .or_insert_with(|| DedupEntry {
                mailboxes: Vec::new(),
                timestamp: Instant::now(),
            });

        entry
            .mailboxes
            .push((account_id.to_string(), mailbox.to_string()));
    }

    fn evict_oldest_entries(&mut self) {
        let mut entries: Vec<_> = self
            .dedup_tracker
            .iter()
            .map(|(k, v)| (k.clone(), v.timestamp))
            .collect();

        entries.sort_by(|a, b| a.1.cmp(&b.1));

        let to_remove = entries.len().min(DEDUP_EVICTION_BATCH);
        for (key, _) in entries.into_iter().take(to_remove) {
            self.dedup_tracker.remove(&key);
        }

        tracing::info!(target: "postail",
            "[Pool] Evicted {} old entries from dedup_tracker, remaining: {}",
            to_remove,
            self.dedup_tracker.len()
        );
    }

    pub fn is_mailbox_virtual(&self, account_id: &str, mailbox: &str) -> bool {
        let key = (account_id.to_string(), mailbox.to_string());
        self.idle_watches
            .get(&key)
            .map(|w| w.is_virtual)
            .unwrap_or(false)
    }
}

pub static CONNECTION_POOL: LazyLock<Arc<Mutex<ConnectionPool>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ConnectionPool::new())));

async fn polling_worker() {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let mut pool = CONNECTION_POOL.lock().await;
        pool.poll_all().await;
    }
}

async fn rebalance_worker() {
    let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
    loop {
        interval.tick().await;
        let mut pool = CONNECTION_POOL.lock().await;
        pool.rebalance().await;
    }
}

async fn cleanup_worker() {
    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
    loop {
        interval.tick().await;
        let mut pool = CONNECTION_POOL.lock().await;
        pool.cleanup_dedup();
    }
}

pub async fn init_pool() {
    let mut pool = CONNECTION_POOL.lock().await;
    pool.start_workers();
    tracing::info!(target: "postail", "[Pool] Connection pool initialized");
}

pub async fn shutdown_pool() {
    let mut pool = CONNECTION_POOL.lock().await;
    pool.stop_workers().await;
    tracing::info!(target: "postail", "[Pool] Connection pool shutdown");
}
