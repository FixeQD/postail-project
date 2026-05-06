use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, command};
use tracing::info;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct SendWidget<T>(pub T);
unsafe impl<T> Send for SendWidget<T> {}
unsafe impl<T> Sync for SendWidget<T> {}

#[derive(Clone, Default)]
pub struct EmbeddedEmailState {
    #[cfg(target_os = "linux")]
    pub bounds: Arc<Mutex<(i32, i32, i32, i32)>>,
    #[cfg(target_os = "linux")]
    pub overlay: Arc<Mutex<Option<SendWidget<gtk::Overlay>>>>,
    #[cfg(target_os = "linux")]
    pub email_wv: Arc<Mutex<Option<SendWidget<webkit2gtk::WebView>>>>,
}

#[cfg(target_os = "linux")]
fn find_new_renderer_pid(pids_before: &std::collections::HashSet<sysinfo::Pid>) -> Option<u32> {
    info!("LoadEvent::Started triggered, diffing processes...");
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let my_pid = sysinfo::Pid::from_u32(std::process::id());

    for (pid, process) in sys.processes() {
        if !pids_before.contains(pid) {
            let mut current = *pid;
            let mut is_child = false;
            while let Some(proc) = sys.process(current) {
                if let Some(parent) = proc.parent() {
                    if parent == my_pid {
                        is_child = true;
                        break;
                    }
                    current = parent;
                } else {
                    break;
                }
            }

            let name = format!("{:?}", process.name()).to_lowercase();
            let pid_u32 = pid.as_u32();
            info!(pid = pid_u32, name = ?name, is_child, "new process detected in diff");

            if is_child
                && (name.contains("webkit")
                    || name.contains("webprocess")
                    || name.contains("bwrap"))
            {
                info!(pid = pid_u32, name = ?name, "matched target process!");
                return Some(pid_u32);
            }
        }
    }

    info!("failed to find matching process in diff");
    None
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[command]
pub fn create_email_webview(
    app: AppHandle,
    state: tauri::State<'_, EmbeddedEmailState>,
    watchdog: tauri::State<'_, crate::cmd::watchdog::WatchdogState>,
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

// ---------------------------------------------------------------------------
// Linux — GTK overlay
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub fn create(
        app: AppHandle,
        state: EmbeddedEmailState,
        watchdog: crate::cmd::watchdog::WatchdogState,
    ) -> Result<(), String> {
        if state.email_wv.lock().unwrap().is_some() {
            let s = state.clone();
            gtk::glib::MainContext::default().invoke(move || {
                let wv = s.email_wv.lock().unwrap();
                let ov = s.overlay.lock().unwrap();
                if let (Some(wv), Some(ov)) = (wv.as_ref(), ov.as_ref()) {
                    use gtk::prelude::WidgetExt;
                    use webkit2gtk::WebViewExt;
                    wv.0.load_uri("postail://localhost/message/current");
                    wv.0.show();
                    ov.0.queue_resize();
                }
            });
            return Ok(());
        }

        let bounds_arc = state.bounds.clone();
        let overlay_arc = state.overlay.clone();
        let wv_arc = state.email_wv.clone();
        let watchdog_arc = watchdog.clone();

        let main_window = app
            .get_webview_window("main")
            .ok_or("main window not found")?;

        main_window
            .with_webview(move |main_wv| {
                use gtk::prelude::*;
                use webkit2gtk::WebViewExt;
                use webkit2gtk::glib::Cast;

                let main_gtk_wv: &webkit2gtk::WebView = &main_wv.inner();
                let widget = main_gtk_wv.upcast_ref::<gtk::Widget>();

                let toplevel = widget
                    .toplevel()
                    .and_then(|t| t.downcast::<gtk::Window>().ok())
                    .expect("GtkWindow toplevel not found");

                let original_child = toplevel.child().expect("window has no child widget");
                toplevel.remove(&original_child);

                let overlay = gtk::Overlay::new();
                overlay.add(&original_child);
                toplevel.add(&overlay);

                let ctx = main_gtk_wv
                    .web_context()
                    .expect("no WebContext on main webview");

                let mut sys = sysinfo::System::new_all();
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                let pids_before: std::collections::HashSet<_> =
                    sys.processes().keys().copied().collect();

                let email_wv = webkit2gtk::WebView::with_context(&ctx);

                if let Some(settings) = WebViewExt::settings(&email_wv) {
                    use webkit2gtk::SettingsExt;
                    settings.set_javascript_can_open_windows_automatically(false);
                    settings.set_allow_modal_dialogs(false);
                }

                overlay.add_overlay(&email_wv);
                overlay.set_overlay_pass_through(&email_wv, false);

                let bounds_for_signal = bounds_arc.clone();
                overlay.connect_get_child_position(move |_ov, _widget| {
                    let (x, y, w, h) = *bounds_for_signal.lock().unwrap();
                    Some(gdk::Rectangle::new(x, y, w.max(1), h.max(1)))
                });

                let watchdog_for_signal = watchdog_arc.clone();
                email_wv.connect_load_changed(move |_wv, event| {
                    if event == webkit2gtk::LoadEvent::Started {
                        if let Some(pid) = find_new_renderer_pid(&pids_before) {
                            watchdog_for_signal.0.lock().unwrap().pid = Some(pid);
                            info!(pid, "email renderer PID captured via sysinfo diff");
                        }
                    }
                });

                overlay.show_all();
                email_wv.hide();
                email_wv.load_uri("postail://localhost/message/current");

                *overlay_arc.lock().unwrap() = Some(SendWidget(overlay));
                *wv_arc.lock().unwrap() = Some(SendWidget(email_wv));

                info!("embedded email WebView created inside GtkOverlay");
            })
            .map_err(|e| format!("with_webview failed: {e}"))?;

        Ok(())
    }

    pub fn update_bounds(
        state: EmbeddedEmailState,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), String> {
        *state.bounds.lock().unwrap() = (x as i32, y as i32, width as i32, height as i32);
        gtk::glib::MainContext::default().invoke(move || {
            let wv = state.email_wv.lock().unwrap();
            let ov = state.overlay.lock().unwrap();
            if let (Some(wv), Some(ov)) = (wv.as_ref(), ov.as_ref()) {
                use gtk::prelude::WidgetExt;
                wv.0.show();
                ov.0.queue_resize();
            }
        });
        Ok(())
    }

    pub fn destroy(state: EmbeddedEmailState) {
        gtk::glib::MainContext::default().invoke(move || {
            if let Some(wv) = state.email_wv.lock().unwrap().as_ref() {
                use gtk::prelude::WidgetExt;
                use webkit2gtk::WebViewExt;
                wv.0.hide();
                wv.0.terminate_web_process();
                info!("embedded email WebView hidden and renderer process terminated");
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Windows — Tauri native child webview (WebView2)
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::time::Duration;

    pub fn create(
        app: AppHandle,
        watchdog: crate::cmd::watchdog::WatchdogState,
    ) -> Result<(), String> {
        // Close any existing child webview first
        if let Some(wv) = app.get_webview("email-webview") {
            let _ = wv.close();
        }

        let window = app
            .get_webview_window("main")
            .ok_or("main window not found")?;

        tauri::WebviewBuilder::new(
            "email-webview",
            tauri::WebviewUrl::Custom(
                "postail://localhost/message/current"
                    .parse()
                    .map_err(|e: url::ParseError| e.to_string())?,
            ),
        )
        .build(&window)
        .map_err(|e| e.to_string())?;

        let our_pid = std::process::id();
        let watchdog_clone = watchdog.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(600)).await;
            if let Some(pid) = find_webview2_renderer_pid(our_pid) {
                watchdog_clone.0.lock().unwrap().pid = Some(pid);
                info!(pid, "email renderer PID captured (WebView2)");
            } else {
                tracing::warn!("could not resolve WebView2 renderer PID");
            }
        });

        Ok(())
    }

    pub fn update_bounds(
        app: AppHandle,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<(), String> {
        if let Some(wv) = app.get_webview("email-webview") {
            wv.set_bounds(tauri::dpi::Rect {
                position: tauri::dpi::Position::Logical(tauri::dpi::LogicalPosition::new(x, y)),
                size: tauri::dpi::Size::Logical(tauri::dpi::LogicalSize::new(width, height)),
            })
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn destroy(app: AppHandle) {
        if let Some(wv) = app.get_webview("email-webview") {
            let _ = wv.close();
            info!("email WebView closed (Windows)");
        }
    }

    /// Scans child processes of `parent_pid` for an msedgewebview2.exe renderer.
    fn find_webview2_renderer_pid(parent_pid: u32) -> Option<u32> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        };

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut found = None;

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32ParentProcessID == parent_pid {
                        let name = String::from_utf16_lossy(
                            &entry.szExeFile
                                [..entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260)],
                        );
                        if name.to_lowercase().contains("msedgewebview2") {
                            found = Some(entry.th32ProcessID);
                            break;
                        }
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
            found
        }
    }
}
