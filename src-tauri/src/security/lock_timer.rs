use crate::security::lock::{lock, should_lock};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

static TIMER_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn start_lock_timer(app_handle: AppHandle) {
    if TIMER_RUNNING.swap(true, Ordering::SeqCst) {
        return; // already running
    }

    let mut interval = interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        if !TIMER_RUNNING.load(Ordering::SeqCst) {
            break;
        }

        if should_lock() {
            lock();
            let _ = app_handle.emit("app:locked", ());
        }
    }
}

pub fn stop_lock_timer() {
    TIMER_RUNNING.store(false, Ordering::SeqCst);
}
