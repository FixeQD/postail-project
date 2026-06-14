use crate::state::{EmbeddedEmailState, SendWidget};
use crate::watchdog::WatchdogState;
use gtk::prelude::*;
use tauri::{AppHandle, Manager};
use webkit2gtk::WebViewExt;

fn find_new_renderer_pid(pids_before: &std::collections::HashSet<sysinfo::Pid>) -> Option<u32> {
    tracing::info!("LoadEvent::Started triggered, diffing processes...");
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
            tracing::info!(pid = pid_u32, name = ?name, is_child, "new process detected in diff");
            if is_child
                && (name.contains("webkit")
                    || name.contains("webprocess")
                    || name.contains("bwrap"))
            {
                tracing::info!(pid = pid_u32, name = ?name, "matched target process!");
                return Some(pid_u32);
            }
        }
    }
    tracing::info!("failed to find matching process in diff");
    None
}

pub fn create(
    app: AppHandle,
    state: EmbeddedEmailState,
    watchdog: WatchdogState,
) -> Result<(), String> {
    if state.email_wv.lock().unwrap().is_some() {
        let s = state.clone();
        gtk::glib::MainContext::default().invoke(move || {
            let wv = s.email_wv.lock().unwrap();
            let ov = s.overlay.lock().unwrap();
            if let (Some(wv), Some(ov)) = (wv.as_ref(), ov.as_ref()) {
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
                        tracing::info!(pid, "email renderer PID captured via sysinfo diff");
                    }
                }
            });

            overlay.show_all();
            email_wv.hide();
            email_wv.load_uri("postail://localhost/message/current");

            *overlay_arc.lock().unwrap() = Some(SendWidget(overlay));
            *wv_arc.lock().unwrap() = Some(SendWidget(email_wv));

            tracing::info!("embedded email WebView created inside GtkOverlay");
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
            wv.0.show();
            ov.0.queue_resize();
        }
    });
    Ok(())
}

pub fn destroy(state: EmbeddedEmailState) {
    gtk::glib::MainContext::default().invoke(move || {
        if let Some(wv) = state.email_wv.lock().unwrap().as_ref() {
            wv.0.hide();
            wv.0.terminate_web_process();
            tracing::info!("embedded email WebView hidden and renderer process terminated");
        }
    });
}

pub fn reload(state: EmbeddedEmailState) -> Result<(), String> {
    gtk::glib::MainContext::default().invoke(move || {
        if let Some(wv) = state.email_wv.lock().unwrap().as_ref() {
            wv.0.load_uri("postail://localhost/message/current");
            tracing::info!("email WebView reloaded");
        }
    });
    Ok(())
}
