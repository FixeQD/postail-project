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

const RATE_LIMIT: Duration = Duration::from_millis(50);

/// Called by the injected script every ~100 ms.
#[tauri::command]
pub async fn email_heartbeat(
    state: tauri::State<'_, WatchdogState>,
    token: String,
) -> Result<String, ()> {
    let mut data = state.0.lock().unwrap();

    if data.token.is_empty() {
        return Err(());
    }

    if data.token != token {
        return Err(());
    }

    // Rate-limit: drop floods without rotating so the caller can retry on the next tick.
    let now = Instant::now();
    if now.duration_since(data.last_heartbeat) < RATE_LIMIT {
        return Err(());
    }

    data.last_heartbeat = now;

    let next = uuid::Uuid::new_v4().to_string();
    data.token = next.clone();

    Ok(next)
}

// ---------------------------------------------------------------------------
// Watchdog loop
// ---------------------------------------------------------------------------

#[derive(Clone, serde::Serialize)]
pub struct FreezeStats {
    pub silent_for_ms: u64,
    pub pid: Option<u32>,
}

/// Ticks every 200 ms. Freezes the webview if heartbeat goes silent too long.
pub async fn run_watchdog_loop(app: tauri::AppHandle, state: WatchdogState) {
    loop {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut data = state.0.lock().unwrap();

        if data.token.is_empty() || data.is_frozen {
            continue;
        }

        let now = Instant::now();

        // 3 s grace period for the first 5 s, then tighten to 500 ms (27.12).
        let threshold = if now.duration_since(data.created_at) < Duration::from_secs(5) {
            Duration::from_millis(3000)
        } else {
            Duration::from_millis(500)
        };

        let silent_for = now.duration_since(data.last_heartbeat);
        if silent_for < threshold {
            continue;
        }

        data.is_frozen = true;
        let stats = FreezeStats {
            silent_for_ms: silent_for.as_millis() as u64,
            pid: data.pid,
        };

        drop(data);

        // 27.13 — SIGSTOP / SuspendThread on `stats.pid` goes here.
        tracing::warn!(pid = ?stats.pid, silent_ms = stats.silent_for_ms, "email webview frozen");

        // 27.16 — notify the frontend.
        if let Err(e) = app.emit("email_webview_frozen", &stats) {
            tracing::error!("failed to emit email_webview_frozen: {e}");
        }
    }
}
