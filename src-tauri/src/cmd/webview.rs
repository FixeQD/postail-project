use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, command};
use tracing::info;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct EmbeddedEmailState {
    pub bounds: Arc<Mutex<(i32, i32, i32, i32)>>,
    pub overlay: Arc<Mutex<Option<SendWidget<gtk::Overlay>>>>,
    pub email_wv: Arc<Mutex<Option<SendWidget<webkit2gtk::WebView>>>>,
}

pub struct SendWidget<T>(pub T);
unsafe impl<T> Send for SendWidget<T> {}
unsafe impl<T> Sync for SendWidget<T> {}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[command]
pub fn create_email_webview(
    app: AppHandle,
    state: tauri::State<'_, EmbeddedEmailState>,
    watchdog: tauri::State<'_, crate::cmd::watchdog::WatchdogState>,
) -> Result<(), String> {
    // Already set up — just reload & show
    if state.email_wv.lock().unwrap().is_some() {
        let state_clone = state.inner().clone();
        gtk::glib::MainContext::default().invoke(move || {
            let wv = state_clone.email_wv.lock().unwrap();
            let ov = state_clone.overlay.lock().unwrap();
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
    let watchdog_arc = watchdog.inner().clone();

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

            // Share WebContext so postail:// protocol handler is available
            let ctx = main_gtk_wv
                .web_context()
                .expect("no WebContext on main webview");

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
            email_wv.connect_load_changed(move |wv, event| {
                if event == webkit2gtk::LoadEvent::Started {
                    use webkit2gtk::glib::ObjectExt;
                    let pid: u32 = wv.property("web-process-identifier");
                    if pid != 0 {
                        watchdog_for_signal.0.lock().unwrap().pid = Some(pid);
                        info!(pid, "email renderer PID captured");
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

#[command]
pub fn update_email_webview_bounds(
    state: tauri::State<'_, EmbeddedEmailState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    *state.bounds.lock().unwrap() = (x as i32, y as i32, width as i32, height as i32);

    let state_clone = state.inner().clone();

    gtk::glib::MainContext::default().invoke(move || {
        let wv = state_clone.email_wv.lock().unwrap();
        let ov = state_clone.overlay.lock().unwrap();
        if let (Some(wv), Some(ov)) = (wv.as_ref(), ov.as_ref()) {
            use gtk::prelude::WidgetExt;
            wv.0.show();
            ov.0.queue_resize();
        }
    });

    Ok(())
}

#[command]
pub fn destroy_email_webview(state: tauri::State<'_, EmbeddedEmailState>) {
    let state_clone = state.inner().clone();
    gtk::glib::MainContext::default().invoke(move || {
        if let Some(wv) = state_clone.email_wv.lock().unwrap().as_ref() {
            use gtk::prelude::WidgetExt;
            wv.0.hide();
            info!("embedded email WebView hidden");
        }
    });
}
