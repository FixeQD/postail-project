use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::SyncStatusEnum;

pub struct SyncStatusManager {
    statuses: Mutex<Vec<SyncStatus>>,
    stop_flags: Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>,
}

struct SyncStatus {
    account_id: String,
    status: Arc<Mutex<SyncStatusEnum>>,
    progress: Arc<AtomicU32>,
    total: Arc<AtomicU32>,
    current_mailbox: Arc<Mutex<Option<String>>>,
    last_error: Arc<Mutex<Option<String>>>,
    last_sync: Arc<Mutex<Option<u64>>>,
}

impl Default for SyncStatusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncStatusManager {
    pub fn new() -> Self {
        Self {
            statuses: Mutex::new(Vec::new()),
            stop_flags: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub async fn register_account(&self, account_id: &str) {
        let mut statuses = self.statuses.lock().await;
        let status = SyncStatus {
            account_id: account_id.to_string(),
            status: Arc::new(Mutex::new(SyncStatusEnum::Idle)),
            progress: Arc::new(AtomicU32::new(0)),
            total: Arc::new(AtomicU32::new(0)),
            current_mailbox: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            last_sync: Arc::new(Mutex::new(None)),
        };
        statuses.push(status);
    }

    pub async fn unregister_account(&self, account_id: &str) {
        let mut statuses = self.statuses.lock().await;
        statuses.retain(|s| s.account_id != account_id);
        let mut stop_flags = self.stop_flags.lock().await;
        stop_flags.remove(account_id);
    }

    pub async fn get_stop_flag(&self, account_id: &str) -> Arc<AtomicBool> {
        let mut stop_flags = self.stop_flags.lock().await;
        if !stop_flags.contains_key(account_id) {
            stop_flags.insert(account_id.to_string(), Arc::new(AtomicBool::new(false)));
        }
        stop_flags.get(account_id).unwrap().clone()
    }

    pub async fn set_status(&self, account_id: &str, status: SyncStatusEnum) {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                let mut current = s.status.lock().await;
                *current = status;
                return;
            }
        }
    }

    pub async fn get_status(&self, account_id: &str) -> SyncStatusEnum {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                let status = s.status.lock().await;
                return match *status {
                    SyncStatusEnum::Idle => SyncStatusEnum::Idle,
                    SyncStatusEnum::Syncing => SyncStatusEnum::Syncing,
                    SyncStatusEnum::Error(ref e) => SyncStatusEnum::Error(e.clone()),
                };
            }
        }
        SyncStatusEnum::Idle
    }

    pub async fn set_progress(&self, account_id: &str, current: u32, total: u32) {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                s.progress.store(current, Ordering::SeqCst);
                s.total.store(total, Ordering::SeqCst);
                return;
            }
        }
    }

    pub async fn set_mailbox(&self, account_id: &str, mailbox: Option<&str>) {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                let mut current = s.current_mailbox.lock().await;
                *current = mailbox.map(|m| m.to_string());
                return;
            }
        }
    }

    pub async fn set_error(&self, account_id: &str, error: Option<&str>) {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                let mut current = s.last_error.lock().await;
                *current = error.map(|e| e.to_string());
                return;
            }
        }
    }

    pub async fn update_last_sync(&self, account_id: &str) {
        let now = chrono::Utc::now().timestamp() as u64;
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                let mut last = s.last_sync.lock().await;
                *last = Some(now);
                return;
            }
        }
    }
}

lazy_static::lazy_static! {
    pub static ref SYNC_STATUS_MANAGER: SyncStatusManager = SyncStatusManager::new();
}

pub async fn update_sync_status(
    account_id: &str,
    mailbox: &str,
    current_uid: u32,
    total_uids: u32,
) {
    SYNC_STATUS_MANAGER
        .set_status(account_id, SyncStatusEnum::Syncing)
        .await;
    SYNC_STATUS_MANAGER
        .set_mailbox(account_id, Some(mailbox))
        .await;
    SYNC_STATUS_MANAGER
        .set_progress(account_id, current_uid, total_uids)
        .await;
}

pub async fn mark_sync_error(account_id: &str, error: &str) {
    SYNC_STATUS_MANAGER
        .set_status(account_id, SyncStatusEnum::Error(error.to_string()))
        .await;
    SYNC_STATUS_MANAGER.set_error(account_id, Some(error)).await;
}

pub async fn mark_sync_complete(account_id: &str) {
    SYNC_STATUS_MANAGER
        .set_status(account_id, SyncStatusEnum::Idle)
        .await;
    SYNC_STATUS_MANAGER.set_mailbox(account_id, None).await;
    SYNC_STATUS_MANAGER.update_last_sync(account_id).await;
}

pub async fn start_sync_status_tracking(account_id: &str) {
    SYNC_STATUS_MANAGER.register_account(account_id).await;
}

pub async fn stop_sync_status_tracking(account_id: &str) {
    SYNC_STATUS_MANAGER.unregister_account(account_id).await;
}
