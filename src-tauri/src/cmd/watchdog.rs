use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Emitter;

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

    /// Timestamp of when the webview was last resumed
    pub resumed_at: Option<Instant>,
}

impl Default for WatchdogData {
    fn default() -> Self {
        Self {
            last_heartbeat: Instant::now(),
            pid: None,
            token: String::new(),
            is_frozen: false,
            created_at: Instant::now(),
            resumed_at: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct WatchdogState(pub Arc<Mutex<WatchdogData>>);

// ---------------------------------------------------------------------------
// Heartbeat logic
// ---------------------------------------------------------------------------

const RATE_LIMIT: Duration = Duration::from_millis(50);

pub fn email_heartbeat(state: &WatchdogState, token: &str) -> Result<String, ()> {
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
    pub memory_bytes: u64,
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

        let threshold = if let Some(resumed_at) = data.resumed_at {
            let since_resume = now.duration_since(resumed_at);
            if since_resume < Duration::from_secs(10) {
                // 10s cooldown after unfreeze
                Duration::from_secs(10)
            } else if since_resume < Duration::from_secs(15) {
                // 5s grace period at 3s after the cooldown ends
                Duration::from_millis(3000)
            } else {
                Duration::from_millis(500)
            }
        } else {
            // 3 s grace period for the first 5 s, then tighten to 500 ms (27.12).
            if now.duration_since(data.created_at) < Duration::from_secs(5) {
                Duration::from_millis(3000)
            } else {
                Duration::from_millis(500)
            }
        };

        let silent_for = now.duration_since(data.last_heartbeat);
        if silent_for < threshold {
            continue;
        }

        data.is_frozen = true;
        let stats = FreezeStats {
            silent_for_ms: silent_for.as_millis() as u64,
            pid: data.pid,
            memory_bytes: data.pid.map(read_renderer_memory).unwrap_or(0),
        };

        drop(data);

        if let Some(pid) = stats.pid {
            #[cfg(target_os = "linux")]
            {
                let nix_pid = nix::unistd::Pid::from_raw(pid as i32);
                if let Err(e) = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGSTOP) {
                    tracing::error!(pid, err = %e, "SIGSTOP failed");
                } else {
                    tracing::warn!(pid, silent_ms = stats.silent_for_ms, "email webview frozen");
                }
            }
            #[cfg(target_os = "windows")]
            {
                match suspend_process_windows(pid) {
                    Ok(()) => {
                        tracing::warn!(pid, silent_ms = stats.silent_for_ms, "email webview frozen")
                    }
                    Err(e) => tracing::error!(pid, err = %e, "SuspendThread failed"),
                }
            }
        }

        // 27.16 — notify the frontend.
        if let Err(e) = app.emit("email_webview_frozen", &stats) {
            tracing::error!("failed to emit email_webview_frozen: {e}");
        }
    }
}
#[cfg(target_os = "windows")]
fn suspend_process_windows(pid: u32) -> Result<(), String> {
    use windows::Win32::Foundation::{CloseHandle, FALSE};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows::Win32::System::Threading::{OpenThread, SuspendThread, THREAD_SUSPEND_RESUME};

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).map_err(|e| e.to_string())?;
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if Thread32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32OwnerProcessID == pid {
                    if let Ok(thread) =
                        OpenThread(THREAD_SUSPEND_RESUME, FALSE.into(), entry.th32ThreadID)
                    {
                        SuspendThread(thread);
                        let _ = CloseHandle(thread);
                    }
                }
                if Thread32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Resume command
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn resume_email_webview(
    app: tauri::AppHandle,
    state: tauri::State<'_, WatchdogState>,
) -> Result<(), String> {
    let mut data = state.0.lock().unwrap();

    if !data.is_frozen {
        return Ok(());
    }

    if let Some(pid) = data.pid {
        #[cfg(target_os = "linux")]
        {
            let nix_pid = nix::unistd::Pid::from_raw(pid as i32);
            if let Err(e) = nix::sys::signal::kill(nix_pid, nix::sys::signal::Signal::SIGCONT) {
                tracing::error!(pid, err = %e, "SIGCONT failed");
                return Err(format!("SIGCONT failed: {}", e));
            } else {
                tracing::info!(pid, "email webview unfrozen");
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = resume_process_windows(pid) {
                tracing::error!(pid, err = %e, "ResumeThread failed");
                return Err(e);
            } else {
                tracing::info!(pid, "email webview unfrozen");
            }
        }
    }

    data.is_frozen = false;
    data.last_heartbeat = Instant::now();
    data.resumed_at = Some(Instant::now());
    drop(data);

    if let Err(e) = app.emit("email_webview_resumed", ()) {
        tracing::error!("failed to emit email_webview_resumed: {e}");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn resume_process_windows(pid: u32) -> Result<(), String> {
    use windows::Win32::Foundation::{CloseHandle, FALSE};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).map_err(|e| e.to_string())?;
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if Thread32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32OwnerProcessID == pid {
                    if let Ok(thread) =
                        OpenThread(THREAD_SUSPEND_RESUME, FALSE.into(), entry.th32ThreadID)
                    {
                        ResumeThread(thread);
                        let _ = CloseHandle(thread);
                    }
                }
                if Thread32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Memory stats
// ---------------------------------------------------------------------------

/// Returns the RSS of the given process in bytes
/// Returns 0 on any failure so callers can treat it as optional
fn read_renderer_memory(pid: u32) -> u64 {
    #[cfg(target_os = "linux")]
    {
        // /proc/{pid}/statm: size  rss  shared  text  lib  data  dt
        let path = format!("/proc/{pid}/statm");
        let Ok(content) = std::fs::read_to_string(&path) else {
            return 0;
        };
        let rss_pages: u64 = content
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        rss_pages * 4096
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        };

        unsafe {
            let Ok(handle) = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
            else {
                return 0;
            };

            let mut pmc = PROCESS_MEMORY_COUNTERS::default();
            let ok = GetProcessMemoryInfo(
                handle,
                &mut pmc,
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            );
            let _ = CloseHandle(handle);

            if ok.is_ok() {
                pmc.WorkingSetSize as u64
            } else {
                0
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        0
    }
}
