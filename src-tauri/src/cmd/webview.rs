use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, command};
use tracing::info;

// ---------------------------------------------------------------------------
// Send/Sync wrapper for non-Send handles (GTK widgets, COM objects)
// ---------------------------------------------------------------------------

pub struct SendWidget<T>(pub T);
unsafe impl<T> Send for SendWidget<T> {}
unsafe impl<T> Sync for SendWidget<T> {}

// ---------------------------------------------------------------------------
// Windows COM state
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub struct WinViewInner {
    pub comp_ctrl:
        Option<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2CompositionController>,
    pub controller: Option<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller>,
    pub dcomp_device: Option<windows_wv::Win32::Graphics::DirectComposition::IDCompositionDevice>,
    pub dcomp_target: Option<windows_wv::Win32::Graphics::DirectComposition::IDCompositionTarget>,
    pub dcomp_visual: Option<windows_wv::Win32::Graphics::DirectComposition::IDCompositionVisual>,
    pub main_hwnd: isize,
}

#[cfg(target_os = "windows")]
unsafe impl Send for WinViewInner {}
#[cfg(target_os = "windows")]
unsafe impl Sync for WinViewInner {}

#[cfg(target_os = "windows")]
impl Default for WinViewInner {
    fn default() -> Self {
        Self {
            comp_ctrl: None,
            controller: None,
            dcomp_device: None,
            dcomp_target: None,
            dcomp_visual: None,
            main_hwnd: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared Tauri state
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct EmbeddedEmailState {
    #[cfg(target_os = "linux")]
    pub bounds: Arc<Mutex<(i32, i32, i32, i32)>>,
    #[cfg(target_os = "linux")]
    pub overlay: Arc<Mutex<Option<SendWidget<gtk::Overlay>>>>,
    #[cfg(target_os = "linux")]
    pub email_wv: Arc<Mutex<Option<SendWidget<webkit2gtk::WebView>>>>,

    #[cfg(target_os = "windows")]
    pub win: Arc<Mutex<WinViewInner>>,
}

// ---------------------------------------------------------------------------
// Linux helper
// ---------------------------------------------------------------------------

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
        return win::create(app, watchdog.inner().clone());
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
        return win::update_bounds(app, x, y, width, height);
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
        win::destroy(app);
    }
}

#[command]
pub fn reload_email_webview(
    app: AppHandle,
    state: tauri::State<'_, EmbeddedEmailState>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    return linux::reload(state.inner().clone());

    #[cfg(target_os = "windows")]
    {
        let _ = state;
        return win::reload(app);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = app;
        let _ = state;
        Ok(())
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

    pub fn reload(state: EmbeddedEmailState) -> Result<(), String> {
        gtk::glib::MainContext::default().invoke(move || {
            if let Some(wv) = state.email_wv.lock().unwrap().as_ref() {
                use webkit2gtk::WebViewExt;
                wv.0.load_uri("postail://localhost/message/current");
                info!("email WebView reloaded");
            }
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Windows — ICoreWebView2CompositionController + DirectComposition
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod win {
    use super::*;

    use windows_core::Interface;
    use windows_wv::Win32::Foundation::{BOOL, HWND, RECT};
    use windows_wv::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
    use windows_wv::Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice,
    };
    use windows_wv::Win32::Graphics::DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
    };
    use windows_wv::Win32::Graphics::Dxgi::IDXGIDevice; // cast() lives here, shared across windows 0.61/0.62

    use webview2_com::Microsoft::Web::WebView2::Win32::{
        CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2,
        ICoreWebView2CompositionController, ICoreWebView2Controller, ICoreWebView2Environment3,
    };
    use webview2_com::{
        CreateCoreWebView2CompositionControllerCompletedHandler,
        CreateCoreWebView2EnvironmentCompletedHandler,
    };

    // -----------------------------------------------------------------------
    // Public entry points
    // -----------------------------------------------------------------------

    pub fn create(
        app: AppHandle,
        watchdog: crate::cmd::watchdog::WatchdogState,
    ) -> Result<(), String> {
        let app2 = app.clone();
        app.run_on_main_thread(move || create_on_main(app2, watchdog))
            .map_err(|e| format!("run_on_main_thread: {e}"))
    }

    pub fn update_bounds(app: AppHandle, x: f64, y: f64, w: f64, h: f64) -> Result<(), String> {
        let app2 = app.clone();
        app.run_on_main_thread(move || update_bounds_on_main(&app2, x, y, w, h))
            .map_err(|e| format!("run_on_main_thread: {e}"))
    }

    pub fn destroy(app: AppHandle) {
        let app2 = app.clone();
        let _ = app.run_on_main_thread(move || destroy_on_main(&app2));
    }

    pub fn reload(app: AppHandle) -> Result<(), String> {
        let app2 = app.clone();
        app.run_on_main_thread(move || reload_on_main(&app2))
            .map_err(|e| format!("run_on_main_thread: {e}"))
    }

    // -----------------------------------------------------------------------
    // Main-thread implementations
    // -----------------------------------------------------------------------

    fn create_on_main(app: AppHandle, watchdog: crate::cmd::watchdog::WatchdogState) {
        let state = app.state::<super::EmbeddedEmailState>();

        if state.win.lock().unwrap().comp_ctrl.is_some() {
            reload_on_main(&app);
            return;
        }

        let window = match app.get_webview_window("main") {
            Some(w) => w,
            None => {
                tracing::error!("[webview/win] main window not found");
                return;
            }
        };

        let hwnd_isize: isize = match window.hwnd() {
            Ok(h) => h.0 as isize,
            Err(e) => {
                tracing::error!("[webview/win] hwnd() failed: {e}");
                return;
            }
        };
        state.win.lock().unwrap().main_hwnd = hwnd_isize;

        let win_arc = state.win.clone();
        let wd = watchdog.clone();

        let env_handler =
            CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |hr, env| {
                if hr.is_err() {
                    tracing::error!("[webview/win] env creation failed: {hr:?}");
                    return Ok(());
                }
                let env = match env {
                    Some(e) => e,
                    None => {
                        tracing::error!("[webview/win] env is None");
                        return Ok(());
                    }
                };

                // Reconstruct windows_wv HWND from stored isize
                let hwnd = HWND(win_arc.lock().unwrap().main_hwnd as *mut _);

                let (dev, tgt, vis) = match setup_dcomp(hwnd) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("[webview/win] DComp setup: {e}");
                        return Ok(());
                    }
                };
                {
                    let mut g = win_arc.lock().unwrap();
                    g.dcomp_device = Some(dev);
                    g.dcomp_target = Some(tgt);
                    g.dcomp_visual = Some(vis);
                }

                let env3: ICoreWebView2Environment3 = match env.cast() {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("[webview/win] env→env3: {e:?}");
                        return Ok(());
                    }
                };

                let win_for_ctrl = win_arc.clone();
                let wd_for_ctrl = wd.clone();

                let ctrl_handler = CreateCoreWebView2CompositionControllerCompletedHandler::create(
                    Box::new(move |hr, comp_ctrl| {
                        on_ctrl_created(hr, comp_ctrl, &win_for_ctrl, &wd_for_ctrl);
                        Ok(())
                    }),
                );

                unsafe {
                    if let Err(e) =
                        env3.CreateCoreWebView2CompositionController(hwnd, &ctrl_handler)
                    {
                        tracing::error!("[webview/win] CreateCompositionController: {e:?}");
                    }
                }
                Ok(())
            }));

        unsafe {
            if let Err(e) = CreateCoreWebView2EnvironmentWithOptions(
                windows_core::PCWSTR::null(),
                windows_core::PCWSTR::null(),
                None,
                &env_handler,
            ) {
                tracing::error!("[webview/win] CreateEnvironmentWithOptions: {e:?}");
            }
        }
    }

    fn on_ctrl_created(
        hr: windows_core::HRESULT,
        comp_ctrl: Option<ICoreWebView2CompositionController>,
        win_arc: &Arc<Mutex<WinViewInner>>,
        watchdog: &crate::cmd::watchdog::WatchdogState,
    ) {
        if hr.is_err() {
            tracing::error!("[webview/win] CompositionController failed: {hr:?}");
            return;
        }
        let comp_ctrl = match comp_ctrl {
            Some(c) => c,
            None => {
                tracing::error!("[webview/win] CompositionController is None");
                return;
            }
        };

        unsafe {
            // Bind WebView2 rendering to our DComp visual
            if let Some(vis) = win_arc.lock().unwrap().dcomp_visual.clone() {
                let unknown: windows_core::IUnknown = match vis.cast() {
                    Ok(u) => u,
                    Err(e) => {
                        tracing::error!("[webview/win] visual→IUnknown: {e:?}");
                        return;
                    }
                };
                // webview2-com 0.38: setter is SetRootVisualTarget, getter is RootVisualTarget()
                if let Err(e) = comp_ctrl.SetRootVisualTarget(&unknown) {
                    tracing::error!("[webview/win] SetRootVisualTarget: {e:?}");
                    return;
                }
            }

            // QI to ICoreWebView2Controller for Bounds / IsVisible / Close
            let controller: ICoreWebView2Controller = match comp_ctrl.cast() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("[webview/win] comp_ctrl→controller: {e:?}");
                    return;
                }
            };

            // Start hidden, 1×1
            let _ = controller.Bounds(RECT {
                left: 0,
                top: 0,
                right: 1,
                bottom: 1,
            });
            let _ = controller.IsVisible(BOOL(0));

            if let Ok(wv) = controller.CoreWebView2() {
                navigate_to_email(&wv);
            }

            if let Some(dev) = win_arc.lock().unwrap().dcomp_device.clone() {
                let _ = dev.Commit();
            }

            let wd = watchdog.clone();
            let our_pid = std::process::id();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                if let Some(pid) = find_webview2_renderer_pid(our_pid) {
                    wd.0.lock().unwrap().pid = Some(pid);
                    info!(pid, "email renderer PID captured (WebView2/DComp)");
                } else {
                    tracing::warn!("[webview/win] could not resolve WebView2 renderer PID");
                }
            });

            let mut g = win_arc.lock().unwrap();
            g.controller = Some(controller);
            g.comp_ctrl = Some(comp_ctrl);
        }

        info!("[webview/win] ICoreWebView2CompositionController ready");
    }

    fn update_bounds_on_main(app: &AppHandle, x: f64, y: f64, w: f64, h: f64) {
        let state = app.state::<super::EmbeddedEmailState>();
        let guard = state.win.lock().unwrap();
        let (Some(ctrl), Some(vis), Some(dev)) = (
            guard.controller.as_ref(),
            guard.dcomp_visual.as_ref(),
            guard.dcomp_device.as_ref(),
        ) else {
            return;
        };

        unsafe {
            let _ = ctrl.Bounds(RECT {
                left: 0,
                top: 0,
                right: w as i32,
                bottom: h as i32,
            });
            let _ = ctrl.IsVisible(BOOL(if w > 0.0 && h > 0.0 { 1 } else { 0 }));
            let _ = vis.SetOffsetX(x as f32);
            let _ = vis.SetOffsetY(y as f32);
            let _ = dev.Commit();
        }
    }

    fn destroy_on_main(app: &AppHandle) {
        let state = app.state::<super::EmbeddedEmailState>();
        let mut guard = state.win.lock().unwrap();
        unsafe {
            if let Some(ctrl) = guard.controller.as_ref() {
                let _ = ctrl.IsVisible(BOOL(0));
                let _ = ctrl.Close();
            }
            if let Some(dev) = guard.dcomp_device.as_ref() {
                let _ = dev.Commit();
            }
        }
        guard.comp_ctrl = None;
        guard.controller = None;
        guard.dcomp_device = None;
        guard.dcomp_target = None;
        guard.dcomp_visual = None;
        info!("[webview/win] email WebView destroyed (DComp)");
    }

    fn reload_on_main(app: &AppHandle) {
        let state = app.state::<super::EmbeddedEmailState>();
        let guard = state.win.lock().unwrap();
        if let Some(ctrl) = guard.controller.as_ref() {
            unsafe {
                if let Ok(wv) = ctrl.CoreWebView2() {
                    navigate_to_email(&wv);
                    info!("[webview/win] email WebView reloaded");
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // DirectComposition setup
    // -----------------------------------------------------------------------

    fn setup_dcomp(
        hwnd: HWND,
    ) -> Result<
        (
            IDCompositionDevice,
            IDCompositionTarget,
            IDCompositionVisual,
        ),
        String,
    > {
        unsafe {
            let mut d3d_device = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )
            .map_err(|e| format!("D3D11CreateDevice: {e:?}"))?;

            let d3d = d3d_device.ok_or("D3D11 device is None")?;
            let dxgi: IDXGIDevice = d3d.cast().map_err(|e| format!("IDXGIDevice cast: {e:?}"))?;

            let dev: IDCompositionDevice = DCompositionCreateDevice(Some(&dxgi))
                .map_err(|e| format!("DCompositionCreateDevice: {e:?}"))?;

            // topmost=true → composites above all HWND children, including Tauri's main WebView2
            let target: IDCompositionTarget = dev
                .CreateTargetForHwnd(hwnd, true)
                .map_err(|e| format!("CreateTargetForHwnd: {e:?}"))?;

            let root: IDCompositionVisual = dev
                .CreateVisual()
                .map_err(|e| format!("CreateVisual (root): {e:?}"))?;
            let email_vis: IDCompositionVisual = dev
                .CreateVisual()
                .map_err(|e| format!("CreateVisual (email): {e:?}"))?;

            root.AddVisual(&email_vis, false, None)
                .map_err(|e| format!("AddVisual: {e:?}"))?;
            target
                .SetRoot(&root)
                .map_err(|e| format!("SetRoot: {e:?}"))?;
            dev.Commit().map_err(|e| format!("initial Commit: {e:?}"))?;

            Ok((dev, target, email_vis))
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    #[inline]
    unsafe fn navigate_to_email(wv: &ICoreWebView2) {
        let url: Vec<u16> = "http://postail.localhost/message/current\0"
            .encode_utf16()
            .collect();
        let _ = wv.Navigate(windows_core::PCWSTR(url.as_ptr()));
    }

    /// Walk ToolHelp snapshot for an msedgewebview2.exe child of `parent_pid`.
    fn find_webview2_renderer_pid(parent_pid: u32) -> Option<u32> {
        use ::windows::Win32::Foundation::CloseHandle;
        use ::windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        };
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut found = None;
            if Process32FirstW(snap, &mut entry).is_ok() {
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
                    if Process32NextW(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
            found
        }
    }
}
