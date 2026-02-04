use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use tauri::{AppHandle, Emitter};
use serde::Serialize;

use crate::db::SyncStatusEnum;

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum SyncEvent {
    #[serde(rename = "started")]
    Started { 
        #[serde(rename = "accountId")]
        account_id: String, 
        #[serde(rename = "accountEmail")]
        account_email: String 
    },
    #[serde(rename = "progress")]
    Progress { 
        #[serde(rename = "accountId")]
        account_id: String, 
        mailbox: String, 
        current: u32, 
        total: u32,
        #[serde(rename = "mailboxProgress")]
        mailbox_progress: Option<MailboxProgress>,
    },
    #[serde(rename = "completed")]
    Completed { 
        #[serde(rename = "accountId")]
        account_id: String, 
        timestamp: u64 
    },
    #[serde(rename = "error")]
    Error { 
        #[serde(rename = "accountId")]
        account_id: String, 
        error: String 
    },
}

#[derive(Clone, Serialize)]
pub struct MailboxProgress {
    #[serde(rename = "currentMailbox")]
    pub current_mailbox: u32,
    #[serde(rename = "totalMailboxes")]
    pub total_mailboxes: u32,
}

pub struct SyncStatusManager {
    statuses: AsyncMutex<Vec<SyncStatus>>,
    stop_flags: AsyncMutex<std::collections::HashMap<String, Arc<AtomicBool>>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

struct SyncStatus {
    account_id: String,
    account_email: String,
    status: Arc<AsyncMutex<SyncStatusEnum>>,
    progress: Arc<AtomicU32>,
    total: Arc<AtomicU32>,
    current_mailbox: Arc<AsyncMutex<Option<String>>>,
    last_error: Arc<AsyncMutex<Option<String>>>,
    last_sync: Arc<AsyncMutex<Option<u64>>>,
    mailbox_count: Arc<AtomicU32>,
    total_mailboxes: Arc<AtomicU32>,
}

impl Default for SyncStatusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncStatusManager {
    pub fn new() -> Self {
        Self {
            statuses: AsyncMutex::new(Vec::new()),
            stop_flags: AsyncMutex::new(std::collections::HashMap::new()),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut guard = self.app_handle.lock().unwrap();
        *guard = Some(handle);
    }

    fn emit_event(&self, event: SyncEvent) {
        let guard = self.app_handle.lock().unwrap();
        if let Some(ref handle) = *guard {
            let event_name = match &event {
                SyncEvent::Started { .. } => "sync:started",
                SyncEvent::Progress { .. } => "sync:progress",
                SyncEvent::Completed { .. } => "sync:completed",
                SyncEvent::Error { .. } => "sync:error",
            };
            let _ = handle.emit::<SyncEvent>(event_name, event);
        }
    }

    pub async fn register_account(&self, account_id: &str, account_email: &str) {
        let mut statuses = self.statuses.lock().await;
        // Check if already registered
        if statuses.iter().any(|s| s.account_id == account_id) {
            return;
        }
        let status = SyncStatus {
            account_id: account_id.to_string(),
            account_email: account_email.to_string(),
            status: Arc::new(AsyncMutex::new(SyncStatusEnum::Idle)),
            progress: Arc::new(AtomicU32::new(0)),
            total: Arc::new(AtomicU32::new(0)),
            current_mailbox: Arc::new(AsyncMutex::new(None)),
            last_error: Arc::new(AsyncMutex::new(None)),
            last_sync: Arc::new(AsyncMutex::new(None)),
            mailbox_count: Arc::new(AtomicU32::new(0)),
            total_mailboxes: Arc::new(AtomicU32::new(0)),
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

    pub async fn request_stop(&self, account_id: &str) {
        let stop_flags = self.stop_flags.lock().await;
        if let Some(flag) = stop_flags.get(account_id) {
            flag.store(true, Ordering::SeqCst);
        }
    }

    pub async fn is_stop_requested(&self, account_id: &str) -> bool {
        let stop_flags = self.stop_flags.lock().await;
        if let Some(flag) = stop_flags.get(account_id) {
            flag.load(Ordering::SeqCst)
        } else {
            false
        }
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

    pub async fn set_mailbox_counters(&self, account_id: &str, current: u32, total: u32) {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                s.mailbox_count.store(current, Ordering::SeqCst);
                s.total_mailboxes.store(total, Ordering::SeqCst);
                return;
            }
        }
    }

    pub async fn get_mailbox_counters(&self, account_id: &str) -> (u32, u32) {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                return (
                    s.mailbox_count.load(Ordering::SeqCst),
                    s.total_mailboxes.load(Ordering::SeqCst),
                );
            }
        }
        (0, 0)
    }

    pub async fn get_account_email(&self, account_id: &str) -> Option<String> {
        let statuses = self.statuses.lock().await;
        for s in statuses.iter() {
            if s.account_id == account_id {
                return Some(s.account_email.clone());
            }
        }
        None
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
    // Decode IMAP UTF-7 mailbox name
    let decoded_mailbox = utf7_imap::decode_utf7_imap(mailbox.to_string());
    
    SYNC_STATUS_MANAGER
        .set_status(account_id, SyncStatusEnum::Syncing)
        .await;
    SYNC_STATUS_MANAGER
        .set_mailbox(account_id, Some(&decoded_mailbox))
        .await;
    SYNC_STATUS_MANAGER
        .set_progress(account_id, current_uid, total_uids)
        .await;
    
    // Get mailbox counters
    let (current_mailbox, total_mailboxes) = SYNC_STATUS_MANAGER
        .get_mailbox_counters(account_id)
        .await;
    
    // Emit progress event
    SYNC_STATUS_MANAGER.emit_event(SyncEvent::Progress {
        account_id: account_id.to_string(),
        mailbox: decoded_mailbox,
        current: current_uid,
        total: total_uids,
        mailbox_progress: if total_mailboxes > 0 {
            Some(MailboxProgress {
                current_mailbox,
                total_mailboxes,
            })
        } else {
            None
        },
    });
}

pub async fn mark_sync_error(account_id: &str, error: &str) {
    SYNC_STATUS_MANAGER
        .set_status(account_id, SyncStatusEnum::Error(error.to_string()))
        .await;
    SYNC_STATUS_MANAGER.set_error(account_id, Some(error)).await;
    
    // Emit error event
    SYNC_STATUS_MANAGER.emit_event(SyncEvent::Error {
        account_id: account_id.to_string(),
        error: error.to_string(),
    });
}

pub async fn mark_sync_complete(account_id: &str) {
    SYNC_STATUS_MANAGER
        .set_status(account_id, SyncStatusEnum::Idle)
        .await;
    SYNC_STATUS_MANAGER.set_mailbox(account_id, None).await;
    SYNC_STATUS_MANAGER.update_last_sync(account_id).await;
    
    // Emit completed event
    let timestamp = chrono::Utc::now().timestamp() as u64;
    SYNC_STATUS_MANAGER.emit_event(SyncEvent::Completed {
        account_id: account_id.to_string(),
        timestamp,
    });
}

pub async fn start_sync_status_tracking(account_id: &str, account_email: &str) {
    SYNC_STATUS_MANAGER.register_account(account_id, account_email).await;
    
    // Emit started event
    SYNC_STATUS_MANAGER.emit_event(SyncEvent::Started {
        account_id: account_id.to_string(),
        account_email: account_email.to_string(),
    });
}

pub async fn stop_sync_status_tracking(account_id: &str) {
    SYNC_STATUS_MANAGER.unregister_account(account_id).await;
}

pub fn set_sync_status_app_handle(handle: AppHandle) {
    SYNC_STATUS_MANAGER.set_app_handle(handle);
}
