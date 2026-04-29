use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

pub const RFC_IDLE_TIMEOUT_SECS: u64 = 29 * 60;
pub const POLL_INTERVAL_SECS: u64 = 60;

// ── Stop / interrupt maps ─────────────────────────────────────────────────────

pub static STOP_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Interrupt handles for active IDLE waits (dropping StopSource cancels the IDLE).
pub static IDLE_INTERRUPTS: LazyLock<Mutex<HashMap<String, stop_token::StopSource>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Notify handles for active poll-loop sleeps.
pub static POLL_INTERRUPTS: LazyLock<Mutex<HashMap<String, Arc<tokio::sync::Notify>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static WATCH_STOP_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Interrupt any active IDLE wait for `account_id`.
pub async fn interrupt_idle(account_id: &str) {
    let mut interrupts = IDLE_INTERRUPTS.lock().await;
    if let Some(interrupt) = interrupts.remove(account_id) {
        tracing::info!(target: "postail", "[IMAP] Interrupting IDLE for {}", account_id);
        drop(interrupt);
    }
}

/// Wake any active poll-loop sleep for `account_id`.
pub async fn interrupt_poll(account_id: &str) {
    let interrupts = POLL_INTERRUPTS.lock().await;
    if let Some(notify) = interrupts.get(account_id) {
        notify.notify_one();
    }
}
