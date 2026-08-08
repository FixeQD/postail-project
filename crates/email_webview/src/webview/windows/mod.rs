use crate::state::EmbeddedEmailState;
use crate::watchdog::WatchdogState;
use tauri::{AppHandle, Manager};

use windows_wv::Win32::Foundation::{LPARAM, WPARAM};
use windows_wv::Win32::System::Threading::GetCurrentThreadId;
use windows_wv::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_APP, WM_QUIT};

mod dcomp;
mod ops;
mod thread;

use ops::{destroy_on_wv_thread, reload_on_wv_thread, update_bounds_on_wv_thread};
use thread::create_on_main;

/// WM_APP is used to post closures to the WebView2 thread's message queue
/// Thread messages (hwnd == NULL) with this ID carry a boxed FnOnce pointer
pub(super) const WV_DISPATCH: u32 = WM_APP;

// ── Public API ───────────────────────────────────────────────────────────────

pub fn create(app: AppHandle, watchdog: WatchdogState) -> Result<(), String> {
    tracing::info!("[webview/win] create: scheduling create_on_main");
    let app2 = app.clone();
    app.run_on_main_thread(move || create_on_main(app2, watchdog))
        .map_err(|e| format!("run_on_main_thread: {e}"))
}

pub fn update_bounds(app: AppHandle, x: f64, y: f64, w: f64, h: f64) -> Result<(), String> {
    let state = app.state::<EmbeddedEmailState>();

    // Stash the latest desired bounds here unconditionally, before even trying to reach the WebView2 thread
    state.win.lock().unwrap().pending_bounds = Some((x, y, w, h));

    let tid = state.win.lock().unwrap().webview2_thread_id;
    tracing::info!(x, y, w, h, tid, "[webview/win] update_bounds requested");
    let win_arc = state.win.clone();
    dispatch_to_wv_thread(tid, move || {
        update_bounds_on_wv_thread(&win_arc, x, y, w, h)
    });
    Ok(())
}

pub fn destroy(app: AppHandle) {
    let state = app.state::<EmbeddedEmailState>();
    let mut g = state.win.lock().unwrap();
    let tid = g.webview2_thread_id;
    // Signal cancellation immediately so any in-flight creation callback bails
    g.is_creating = false;
    drop(g);

    let win_arc = state.win.clone();
    dispatch_to_wv_thread(tid, move || {
        destroy_on_wv_thread(&win_arc);
        // Exit the WebView2 thread's message loop
        unsafe {
            let _ = PostThreadMessageW(GetCurrentThreadId(), WM_QUIT, WPARAM(0), LPARAM(0));
        }
    });
}

pub fn reload(app: AppHandle) -> Result<(), String> {
    let state = app.state::<EmbeddedEmailState>();
    let tid = state.win.lock().unwrap().webview2_thread_id;
    let win_arc = state.win.clone();
    dispatch_to_wv_thread(tid, move || reload_on_wv_thread(&win_arc));
    Ok(())
}

// ── Internal ─────────────────────────────────────────────────────────────────

/// Post a closure to run on the dedicated WebView2 thread
pub(super) fn dispatch_to_wv_thread(tid: u32, f: impl FnOnce() + Send + 'static) {
    if tid == 0 {
        tracing::info!(
            "[webview/win] dispatch_to_wv_thread: no thread yet (tid=0), dropping this dispatch"
        );
        return;
    }
    // Double-box to get a thin pointer we can fit into LPARAM
    let raw = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce() + Send>));
    unsafe {
        let _ = PostThreadMessageW(tid, WV_DISPATCH, WPARAM(0), LPARAM(raw as isize));
    }
}
