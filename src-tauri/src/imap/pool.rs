use crate::error::AppError;
use crate::globals::IMAP_MANAGER;
use crate::oauth::{ProviderInfo, ProviderKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing;

const MAX_DEDUP_ENTRIES: usize = 10000;
const DEDUP_EVICTION_BATCH: usize = 2000; // Remove 20% when limit hit

pub type WatchKey = (String, String);

/// Watch mode for a mailbox
#[derive(Debug)]
pub enum WatchMode {
    /// Full IDLE mode with dedicated connection
    Idle {
        stop_source: Arc<tokio::sync::Notify>,
        task_handle: JoinHandle<()>,
    },
    /// Polling mode - checked periodically
    Polling { last_check: Instant },
    /// Waiting for available IDLE slot
    Queued,
}

#[derive(Debug)]
pub struct MailboxWatch {
    pub account_id: String,
    pub mailbox: String,
    pub mode: WatchMode,
    pub last_activity: Instant,
    pub provider_kind: ProviderKind,
    pub is_virtual: bool, // True if mailbox has \All flag
}

#[derive(Debug, Clone)]
struct DedupEntry {
    mailboxes: Vec<(String, String)>, // (account, mailbox) pairs
    timestamp: Instant,
}

pub struct ConnectionPool {
    idle_watches: HashMap<WatchKey, MailboxWatch>,
    poll_queue: VecDeque<WatchKey>,
    watched_keys: HashSet<WatchKey>,
    dedup_tracker: HashMap<String, DedupEntry>,
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
            polling_task: None,
            rebalance_task: None,
            cleanup_task: None,
        }
    }

    /// Start background workers
    pub fn start_workers(&mut self) {
        self.polling_task = Some(tokio::spawn(polling_worker()));
        self.rebalance_task = Some(tokio::spawn(rebalance_worker()));
        self.cleanup_task = Some(tokio::spawn(cleanup_worker()));
    }

    /// Stop all workers
    pub async fn stop_workers(&mut self) {
        if let Some(task) = self.polling_task.take() {
            task.abort();
        }
        if let Some(task) = self.rebalance_task.take() {
            task.abort();
        }
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
    }

    pub async fn watch_mailbox(&mut self, account_id: &str, mailbox: &str) -> Result<(), AppError> {
        let key = (account_id.to_string(), mailbox.to_string());

        // Check if already watching
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
            // Start IDLE watch
            tracing::info!(target: "postail", "[Pool] Starting IDLE for {}@{} (slot {}/{}",
                mailbox, account_id, account_idle_count + 1, max_idle);
            self.start_idle_watch(account_id, mailbox, provider_kind, is_virtual)
                .await?;
        } else {
            // Add to polling queue
            tracing::info!(target: "postail", "[Pool] Adding {}@{} to polling queue (slots full {}/{})",
                mailbox, account_id, account_idle_count, max_idle);
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
    }

    pub async fn record_activity(&mut self, account_id: &str, mailbox: &str) {
        let key = (account_id.to_string(), mailbox.to_string());

        if let Some(watch) = self.idle_watches.get_mut(&key) {
            watch.last_activity = Instant::now();
            tracing::debug!(target: "postail", "[Pool] Activity recorded for {}@{}", mailbox, account_id);
        }
    }

    pub async fn rebalance(&mut self) {
        tracing::debug!(target: "postail", "[Pool] Starting rebalance");

        // Find stale IDLE watches
        let mut stale_watches: Vec<(WatchKey, ProviderKind)> = Vec::new();
        for (key, watch) in &self.idle_watches {
            let provider = ProviderInfo::get(watch.provider_kind);
            let stale_threshold = Duration::from_secs(provider.stale_threshold_seconds);

            if watch.last_activity.elapsed() > stale_threshold {
                stale_watches.push((key.clone(), watch.provider_kind));
            }
        }

        // Demote stale watches to polling
        for (key, _provider_kind) in stale_watches {
            tracing::info!(target: "postail", "[Pool] Demoting {}@{} to polling (stale)", key.1, key.0);
            if let Some(watch) = self.idle_watches.remove(&key) {
                self.stop_idle_watch(watch).await;
                self.poll_queue.push_back(key);
            }
        }

        // Find hot mailboxes in polling queue
        let hot_keys: Vec<WatchKey> = self
            .poll_queue
            .iter()
            .filter(|(acc, _)| {
                let provider_kind = self.detect_provider_sync(acc);
                let provider = ProviderInfo::get(provider_kind);
                let account_idle = self.count_account_idle_connections(acc);
                account_idle < provider.max_idle_connections
            })
            .cloned()
            .collect();

        // Promote hot mailboxes to IDLE
        for key in hot_keys {
            self.poll_queue.retain(|k| k != &key);

            let provider_kind = self.detect_provider_sync(&key.0);
            let is_virtual = self.get_mailbox_role_sync(&key.0, &key.1) == Some("all".to_string());
            tracing::info!(target: "postail", "[Pool] Promoting {}@{} to IDLE", key.1, key.0);

            if let Err(e) = self
                .start_idle_watch(&key.0, &key.1, provider_kind, is_virtual)
                .await
            {
                tracing::error!(target: "postail", "[Pool] Failed to promote {}@{}: {}", key.1, key.0, e);
                // Put back in queue
                self.poll_queue.push_back(key);
            }
        }

        tracing::debug!(target: "postail", "[Pool] Rebalance complete: {} IDLE, {} polling",
            self.idle_watches.len(), self.poll_queue.len());
    }

    pub async fn poll_all(&mut self) {
        let keys: Vec<WatchKey> = self.poll_queue.iter().cloned().collect();

        for (account_id, mailbox) in keys {
            tokio::spawn(async move {
                if let Err(e) = Self::quick_poll(&account_id, &mailbox).await {
                    tracing::warn!(target: "postail", "[Pool] Poll failed for {}@{}: {}", mailbox, account_id, e);
                }
            });
        }
    }

    pub fn cleanup_dedup(&mut self) {
        let cutoff = Duration::from_secs(24 * 60 * 60); // 24 hours
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
            tracing::debug!(target: "postail", "[Pool] Cleaned up {} old dedup entries", count);
        }
    }

    async fn start_idle_watch(
        &mut self,
        account_id: &str,
        mailbox: &str,
        provider_kind: ProviderKind,
        is_virtual: bool,
    ) -> Result<(), AppError> {
        let stop_notify = Arc::new(tokio::sync::Notify::new());
        let stop_notify_clone = stop_notify.clone();

        let account_id_owned = account_id.to_string();
        let mailbox_owned = mailbox.to_string();

        // Spawn IDLE task
        let task = tokio::spawn(async move {
            if let Err(e) = Self::idle_loop(
                &account_id_owned,
                &mailbox_owned,
                provider_kind,
                stop_notify_clone,
            )
            .await
            {
                tracing::error!(target: "postail", "[Pool] IDLE loop error for {}@{}: {}", mailbox_owned, account_id_owned, e);
            }
        });

        let watch = MailboxWatch {
            account_id: account_id.to_string(),
            mailbox: mailbox.to_string(),
            mode: WatchMode::Idle {
                stop_source: stop_notify,
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
            stop_source,
            task_handle,
        } = watch.mode
        {
            // Signal stop
            stop_source.notify_one();

            // Wait for task to finish
            tokio::select! {
                _ = task_handle => {},
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    tracing::warn!(target: "postail", "[Pool] IDLE task timeout for {}@{}", watch.mailbox, watch.account_id);
                }
            }
        }
    }

    async fn promote_from_queue(&mut self) {
        if let Some(key) = self.poll_queue.pop_front() {
            if let Ok(provider_kind) = self.detect_provider(&key.0).await {
                let is_virtual =
                    self.get_mailbox_role(&key.0, &key.1).await == Some("all".to_string());
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

    async fn detect_provider(&self, account_id: &str) -> Result<ProviderKind, AppError> {
        if let Ok(pool) = crate::globals::get_db_pool().await {
            if let Ok(conn) = pool.get() {
                let mut stmt = conn
                    .prepare("SELECT provider_type FROM accounts WHERE id = ?")
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

                if let Ok(provider_type) =
                    stmt.query_row([account_id], |row| row.get::<_, String>(0))
                {
                    if let Some(kind) = ProviderKind::parse(&provider_type) {
                        return Ok(kind);
                    }
                }
            }
        }

        // Default to Gmail
        Ok(ProviderKind::Gmail)
    }

    async fn get_mailbox_role(&self, account_id: &str, mailbox: &str) -> Option<String> {
        let pool = match crate::globals::get_db_pool().await {
            Ok(p) => p,
            Err(_) => return None,
        };
        let conn = match pool.get() {
            Ok(c) => c,
            Err(_) => return None,
        };
        let result: Result<String, _> = conn.query_row(
            "SELECT role FROM mailboxes WHERE account_id = ? AND name = ?",
            [account_id, mailbox],
            |row| row.get(0),
        );
        result.ok()
    }

    fn detect_provider_sync(&self, _account_id: &str) -> ProviderKind {
        // Blocking version for when can't await - fallback
        ProviderKind::Gmail
    }

    fn get_mailbox_role_sync(&self, _account_id: &str, mailbox: &str) -> Option<String> {
        // Blocking version - this is another fallback
        if mailbox.contains("All Mail") || mailbox.contains("Wszystkie") {
            Some("all".to_string())
        } else {
            None
        }
    }

    async fn idle_loop(
        account_id: &str,
        mailbox: &str,
        provider_kind: ProviderKind,
        stop_notify: Arc<tokio::sync::Notify>,
    ) -> Result<(), AppError> {
        let manager = IMAP_MANAGER.lock().await.clone();
        let provider = ProviderInfo::get(provider_kind);

        loop {
            tokio::select! {
                _ = stop_notify.notified() => {
                    tracing::info!(target: "postail", "[Pool] Stopping IDLE for {}@{}", mailbox, account_id);
                    break;
                }
                result = Self::idle_iteration(&manager, account_id, mailbox, &provider) => {
                    if let Err(e) = result {
                        tracing::error!(target: "postail", "[Pool] IDLE iteration failed for {}@{}: {}", mailbox, account_id, e);
                        // Wait before retry
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn idle_iteration(
        manager: &crate::imap::ImapManager,
        account_id: &str,
        mailbox: &str,
        provider: &ProviderInfo,
    ) -> Result<(), AppError> {
        let timeout = Duration::from_secs(provider.idle_timeout_seconds);

        // Check for new messages
        manager
            .sync_single_mailbox_messages(account_id, mailbox)
            .await?;

        // Wait for timeout or notification
        tokio::time::sleep(timeout).await;

        Ok(())
    }

    async fn quick_poll(account_id: &str, mailbox: &str) -> Result<(), AppError> {
        let manager = IMAP_MANAGER.lock().await.clone();

        tracing::debug!(target: "postail", "[Pool] Polling {}@{}", mailbox, account_id);

        // Quick sync - just check for new messages
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
        // Don't track messages from virtual folders
        if is_virtual {
            return;
        }

        // Check if we need to evict old entries
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

    /// Evict oldest entries when dedup_tracker reaches max size
    fn evict_oldest_entries(&mut self) {
        let mut entries: Vec<_> = self
            .dedup_tracker
            .iter()
            .map(|(k, v)| (k.clone(), v.timestamp))
            .collect();

        // Sort by timestamp (oldest first)
        entries.sort_by(|a, b| a.1.cmp(&b.1));

        let to_remove = entries.len().min(DEDUP_EVICTION_BATCH);
        for (key, _) in entries.into_iter().take(to_remove) {
            self.dedup_tracker.remove(&key);
        }

        tracing::info!(
            target: "postail",
            "[Pool] Evicted {} old entries from dedup_tracker, remaining: {}",
            to_remove,
            self.dedup_tracker.len()
        );
    }

    pub fn is_mailbox_virtual(&self, account_id: &str, mailbox: &str) -> bool {
        let key = (account_id.to_string(), mailbox.to_string());
        if let Some(watch) = self.idle_watches.get(&key) {
            return watch.is_virtual;
        }
        false
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
    let mut interval = tokio::time::interval(Duration::from_secs(5 * 60)); // 5 minutes

    loop {
        interval.tick().await;

        let mut pool = CONNECTION_POOL.lock().await;
        pool.rebalance().await;
    }
}

async fn cleanup_worker() {
    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60)); // 1 hour

    loop {
        interval.tick().await;

        let mut pool = CONNECTION_POOL.lock().await;
        pool.cleanup_dedup();
    }
}

/// Initialize the connection pool and start workers
pub async fn init_pool() {
    let mut pool = CONNECTION_POOL.lock().await;
    pool.start_workers();
    tracing::info!(target: "postail", "[Pool] Connection pool initialized");
}

/// Stop all pool workers
pub async fn shutdown_pool() {
    let mut pool = CONNECTION_POOL.lock().await;
    pool.stop_workers().await;
    tracing::info!(target: "postail", "[Pool] Connection pool shutdown");
}
