use super::state::EmbeddedEmailState;
#[cfg(target_os = "linux")]
use super::linux;
#[cfg(target_os = "windows")]
use super::windows;
use crate::cmd::watchdog::WatchdogState;
use tauri::{AppHandle, command};

#[command]
pub fn create_email_webview(
    app: AppHandle,
    state: tauri::State<'_, EmbeddedEmailState>,
    watchdog: tauri::State<'_, WatchdogState>,
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
    state: tauri::State<'_, EmbeddedEmailState>,
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
pub fn destroy_email_webview(app: AppHandle, state: tauri::State<'_, EmbeddedEmailState>) {
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
    state: tauri::State<'_, EmbeddedEmailState>,
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