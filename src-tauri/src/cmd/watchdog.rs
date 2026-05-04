use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Shared watchdog data - lives for the entire session of one email webview.
pub struct WatchdogData {
    /// Timestamp of the last accepted heartbeat
    pub last_heartbeat: Instant,

    /// OS-level PID of the email renderer process
    pub pid: Option<u32>,

    /// Session token generated when the webview is spawned
    /// An empty string means the watchdog is not yet armed
    pub token: String,

    /// Set to `true` after the process is suspended, cleared on resume
    pub is_frozen: bool,

    /// Timestamp of when the current webview session was created
    pub created_at: Instant,
}

impl Default for WatchdogData {
    fn default() -> Self {
        Self {
            last_heartbeat: Instant::now(),
            pid: None,
            token: String::new(),
            is_frozen: false,
            created_at: Instant::now(),
        }
    }
}

#[derive(Clone, Default)]
pub struct WatchdogState(pub Arc<Mutex<WatchdogData>>);

// ---------------------------------------------------------------------------
// Heartbeat command
// ---------------------------------------------------------------------------

/// Minimum interval between two accepted heartbeats.
/// Anything faster than this is treated as a flood and dropped silently.
const RATE_LIMIT: Duration = Duration::from_millis(50);

#[tauri::command]
pub async fn email_heartbeat(
    state: tauri::State<'_, WatchdogState>,
    token: String,
) -> Result<(), ()> {
    let mut data = state.0.lock().unwrap();

    // Watchdog not yet armed - webview hasn't been fully set up yet.
    if data.token.is_empty() {
        return Ok(());
    }

    // Token mismatch - either a stale webview or someone being clever.
    if data.token != token {
        return Ok(());
    }

    // Rate-limit
    let now = Instant::now();
    if now.duration_since(data.last_heartbeat) < RATE_LIMIT {
        return Ok(());
    }

    data.last_heartbeat = now;

    Ok(())
}
