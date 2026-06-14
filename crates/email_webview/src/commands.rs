use crate::state::EmbeddedEmailState;
use crate::watchdog::WatchdogState;
#[cfg(target_os = "linux")]
use crate::webview::linux;
#[cfg(target_os = "windows")]
use crate::webview::windows;
use tauri::{command, AppHandle, Emitter, State};

#[command]
pub fn create_email_webview(
    app: AppHandle,
    state: State<'_, EmbeddedEmailState>,
    watchdog: State<'_, WatchdogState>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    return linux::create(app, state.inner().clone(), watchdog.inner().clone());

    #[cfg(target_os = "windows")]
    {
        let _ = state;
        return windows::create(app, watchdog.inner().clone());
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Err("platform not supported".into())
}

#[command]
pub fn update_email_webview_bounds(
    app: AppHandle,
    state: State<'_, EmbeddedEmailState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        return linux::update_bounds(state.inner().clone(), x, y, width, height);
    }

    #[cfg(target_os = "windows")]
    {
        let _ = state;
        return windows::update_bounds(app, x, y, width, height);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Ok(())
}

#[command]
pub fn destroy_email_webview(app: AppHandle, state: State<'_, EmbeddedEmailState>) {
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        linux::destroy(state.inner().clone());
    }

    #[cfg(target_os = "windows")]
    {
        let _ = state;
        windows::destroy(app);
    }
}

#[command]
pub fn reload_email_webview(
    app: AppHandle,
    state: State<'_, EmbeddedEmailState>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let _ = app;
        return linux::reload(state.inner().clone());
    }

    #[cfg(target_os = "windows")]
    {
        let _ = state;
        return windows::reload(app);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = app;
        let _ = state;
        Ok(())
    }
}

#[command]
pub async fn resume_email_webview(
    app: AppHandle,
    state: State<'_, WatchdogState>,
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
            if let Err(e) = crate::watchdog::resume_process_windows(pid) {
                tracing::error!(pid, err = %e, "ResumeThread failed");
                return Err(e);
            } else {
                tracing::info!(pid, "email webview unfrozen");
            }
        }
    }

    data.is_frozen = false;
    data.last_heartbeat = std::time::Instant::now();
    data.resumed_at = Some(std::time::Instant::now());
    drop(data);

    if let Err(e) = app.emit("email_webview_resumed", ()) {
        tracing::error!("failed to emit email_webview_resumed: {e}");
    }

    Ok(())
}
