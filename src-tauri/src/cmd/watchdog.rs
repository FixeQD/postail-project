use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Shared watchdog data - lives for the entire session of one email webview.
pub struct WatchdogData {
    /// Timestamp of the last accepted heartbeat.
    pub last_heartbeat: Instant,

    /// OS-level PID of the email renderer process
    pub pid: Option<u32>,

    /// Rolling nonce: the token the *next* heartbeat call must present
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

/// Called by the injected script every ~100 ms.
#[tauri::command]
pub async fn email_heartbeat(
    state: tauri::State<'_, WatchdogState>,
    token: String,
) -> Result<String, ()> {
    let mut data = state.0.lock().unwrap();

    // Not yet armed.
    if data.token.is_empty() {
        return Err(());
    }

    // Wrong token - stale call or rogue script.
    if data.token != token {
        return Err(());
    }

    // Rate-limit: drop floods silently without rotating the token so the legitimate caller can retry on the next tick.
    let now = Instant::now();
    if now.duration_since(data.last_heartbeat) < RATE_LIMIT {
        return Err(());
    }

    data.last_heartbeat = now;

    // Rotate - mint a new nonce and hand it back to the caller.
    let next = uuid::Uuid::new_v4().to_string();
    data.token = next.clone();

    Ok(next)
}
