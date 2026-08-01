use std::sync::{Arc, Mutex};

use crate::state::{EmbeddedEmailState, WinViewInner};
use crate::watchdog::WatchdogState;
use tauri::{AppHandle, Manager};

use windows_wv::core::Interface;
use windows_wv::Win32::Foundation::{HWND, RECT};
use windows_wv::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows_wv::Win32::System::Threading::GetCurrentThreadId;
use windows_wv::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DispatchMessageW, GetMessageW, TranslateMessage, MSG, WINDOW_EX_STYLE,
    WS_CHILD, WS_VISIBLE,
};

use webview2_com::Microsoft::Web::WebView2::Win32::{
    CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2CompositionController,
    ICoreWebView2Controller, ICoreWebView2Environment, ICoreWebView2Environment3,
};
use webview2_com::{
    CreateCoreWebView2CompositionControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler,
};

use super::dcomp::{navigate_to_email, setup_dcomp};
use super::{dispatch_to_wv_thread, WV_DISPATCH};

/// Runs on the main thread only to guard shared state and spawn the WebView2 thread
pub(super) fn create_on_main(app: AppHandle, watchdog: WatchdogState) {
    let state = app.state::<EmbeddedEmailState>();

    {
        let mut g = state.win.lock().unwrap();
        if g.comp_ctrl.is_some() {
            let tid = g.webview2_thread_id;
            drop(g);
            let win_arc = state.win.clone();
            dispatch_to_wv_thread(tid, move || super::ops::reload_on_wv_thread(&win_arc));
            return;
        }
        if g.is_creating {
            tracing::warn!("[webview/win] creation already in progress, ignoring duplicate call");
            return;
        }
        g.is_creating = true;
    }

    let main_hwnd_isize = match app.get_webview_window("main") {
        Some(w) => match w.hwnd() {
            Ok(h) => h.0 as isize,
            Err(e) => {
                tracing::error!("[webview/win] hwnd(): {e}");
                state.win.lock().unwrap().is_creating = false;
                return;
            }
        },
        None => {
            tracing::error!("[webview/win] main window not found");
            state.win.lock().unwrap().is_creating = false;
            return;
        }
    };
    state.win.lock().unwrap().main_hwnd = main_hwnd_isize;

    let win_arc = state.win.clone();
    let app2 = app.clone();

    if let Err(e) = std::thread::Builder::new()
        .name("webview2".into())
        .spawn(move || webview2_thread(main_hwnd_isize, win_arc, watchdog, app2))
    {
        tracing::error!("[webview/win] thread spawn failed: {e}");
        state.win.lock().unwrap().is_creating = false;
    }
}

/// Long-lived dedicated thread that owns all WebView2 COM objects and runs the message loop WebView2 needs to dispatch its async callbacks
fn webview2_thread(
    main_hwnd_isize: isize,
    win_arc: Arc<Mutex<WinViewInner>>,
    watchdog: WatchdogState,
    app: AppHandle,
) {
    unsafe {
        // WebView2 and DirectComposition both require COM STA
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        // Store our thread ID FIRST so destroy() can always reach us, even if it races with our startup
        let tid = GetCurrentThreadId();
        {
            let mut g = win_arc.lock().unwrap();
            if !g.is_creating {
                // destroy() was called before we got going; exit cleanly.
                tracing::warn!("[webview/win] WebView2 thread cancelled before start");
                CoUninitialize();
                return;
            }
            g.webview2_thread_id = tid;
        }

        // Create child_hwnd ON THIS THREAD so its message queue lives here
        let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let window_name: Vec<u16> = "\0".encode_utf16().collect();
        let child_hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            windows_wv::core::PCWSTR(class_name.as_ptr()),
            windows_wv::core::PCWSTR(window_name.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            1,
            1,
            Some(HWND(main_hwnd_isize as *mut _)),
            None,
            None,
            None,
        ) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("[webview/win] CreateWindowExW: {e}");
                let mut g = win_arc.lock().unwrap();
                g.is_creating = false;
                g.webview2_thread_id = 0;
                CoUninitialize();
                return;
            }
        };
        let child_hwnd_isize = child_hwnd.0 as isize;
        win_arc.lock().unwrap().child_hwnd = child_hwnd_isize;

        // --- Kick off async WebView2 initialisation ---
        let win_for_env = win_arc.clone();
        let wd_for_env = watchdog.clone();
        let app_for_env = app.clone();

        let env_handler =
            CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |hr, env| {
                if hr.is_err() {
                    tracing::error!("[webview/win] env creation failed: {hr:?}");
                    win_for_env.lock().unwrap().is_creating = false;
                    return Ok(());
                }
                let env = match env {
                    Some(e) => e,
                    None => {
                        tracing::error!("[webview/win] env is None");
                        win_for_env.lock().unwrap().is_creating = false;
                        return Ok(());
                    }
                };

                {
                    let g = win_for_env.lock().unwrap();
                    if !g.is_creating || g.child_hwnd != child_hwnd_isize {
                        tracing::warn!("[webview/win] env callback: creation superseded");
                        return Ok(());
                    }
                }

                let env3: ICoreWebView2Environment3 = match env.cast() {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("[webview/win] env→env3: {e:?}");
                        win_for_env.lock().unwrap().is_creating = false;
                        return Ok(());
                    }
                };

                let win_for_ctrl = win_for_env.clone();
                let wd_for_ctrl = wd_for_env.clone();
                let env_for_ctrl = crate::state::SendWidget(env.clone());
                let app_for_ctrl = app_for_env.clone();

                let ctrl_handler = CreateCoreWebView2CompositionControllerCompletedHandler::create(
                    Box::new(move |hr, comp_ctrl| {
                        on_ctrl_created(
                            hr,
                            comp_ctrl,
                            child_hwnd_isize,
                            &win_for_ctrl,
                            &wd_for_ctrl,
                            &env_for_ctrl.0,
                            &app_for_ctrl,
                        );
                        Ok(())
                    }),
                );

                if let Err(e) = env3.CreateCoreWebView2CompositionController(
                    HWND(child_hwnd_isize as *mut _),
                    &ctrl_handler,
                ) {
                    tracing::error!("[webview/win] CreateCompositionController call: {e:?}");
                    win_for_env.lock().unwrap().is_creating = false;
                }
                Ok(())
            }));

        if let Err(e) = CreateCoreWebView2EnvironmentWithOptions(
            windows_wv::core::PCWSTR::null(),
            windows_wv::core::PCWSTR::null(),
            None,
            &env_handler,
        ) {
            tracing::error!("[webview/win] CreateEnvironmentWithOptions: {e:?}");
            let mut g = win_arc.lock().unwrap();
            g.is_creating = false;
            g.webview2_thread_id = 0;
            CoUninitialize();
            return;
        }

        // --- Message loop ---
        let mut msg = MSG::default();
        loop {
            match GetMessageW(&mut msg, None, 0, 0).0 {
                -1 => {
                    tracing::error!("[webview/win] GetMessageW returned -1");
                    break;
                }
                0 => break, // WM_QUIT
                _ => {
                    if msg.hwnd.0.is_null() && msg.message == WV_DISPATCH {
                        // Execute a closure posted by the main thread.
                        let f = Box::from_raw(msg.lParam.0 as *mut Box<dyn FnOnce() + Send>);
                        f();
                    } else {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        }

        win_arc.lock().unwrap().webview2_thread_id = 0;
        CoUninitialize();
    }
}

fn on_ctrl_created(
    hr: windows_wv::core::Result<()>,
    comp_ctrl: Option<ICoreWebView2CompositionController>,
    child_hwnd_isize: isize,
    win_arc: &Arc<Mutex<WinViewInner>>,
    watchdog: &WatchdogState,
    env: &ICoreWebView2Environment,
    app: &AppHandle,
) {
    if hr.is_err() {
        tracing::error!("[webview/win] CompositionController failed: {hr:?}");
        win_arc.lock().unwrap().is_creating = false;
        return;
    }
    let comp_ctrl = match comp_ctrl {
        Some(c) => c,
        None => {
            tracing::error!("[webview/win] CompositionController is None");
            win_arc.lock().unwrap().is_creating = false;
            return;
        }
    };

    // Stale guard: destroy() may have fired between the env callback and here
    {
        let g = win_arc.lock().unwrap();
        if !g.is_creating || g.child_hwnd != child_hwnd_isize {
            tracing::warn!("[webview/win] ctrl callback: creation superseded, discarding");
            return;
        }
    }

    // Build DComp visual tree now that WebView2 has finished its own setup
    let child_hwnd = HWND(child_hwnd_isize as *mut _);
    let (dev, tgt, vis) = match setup_dcomp(child_hwnd) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("[webview/win] DComp setup: {e}");
            win_arc.lock().unwrap().is_creating = false;
            return;
        }
    };
    {
        let mut g = win_arc.lock().unwrap();
        g.dcomp_device = Some(dev);
        g.dcomp_target = Some(tgt);
        g.dcomp_visual = Some(vis);
    }

    unsafe {
        if let Some(vis) = win_arc.lock().unwrap().dcomp_visual.clone() {
            let unknown: windows_wv::core::IUnknown = match vis.cast() {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("[webview/win] visual→IUnknown: {e:?}");
                    win_arc.lock().unwrap().is_creating = false;
                    return;
                }
            };
            if let Err(e) = comp_ctrl.SetRootVisualTarget(&unknown) {
                tracing::error!("[webview/win] SetRootVisualTarget: {e:?}");
                win_arc.lock().unwrap().is_creating = false;
                return;
            }
        }

        let controller: ICoreWebView2Controller = match comp_ctrl.cast() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("[webview/win] comp_ctrl→controller: {e:?}");
                win_arc.lock().unwrap().is_creating = false;
                return;
            }
        };

        let _ = controller.SetBounds(RECT {
            left: 0,
            top: 0,
            right: 1,
            bottom: 1,
        });
        let _ = controller.SetIsVisible(false);

        if let Ok(wv) = controller.CoreWebView2() {
            crate::policy::register_webview2_resource_handler(&wv, env, app.clone());
            navigate_to_email(&wv);

            let mut pid: u32 = 0;
            if wv.BrowserProcessId(&mut pid as *mut u32).is_ok() && pid != 0 {
                watchdog.0.lock().unwrap().pid = Some(pid);
                tracing::info!(pid, "email renderer PID captured");
            } else {
                tracing::warn!("[webview/win] BrowserProcessId failed or returned 0");
            }
        }

        if let Some(dev) = win_arc.lock().unwrap().dcomp_device.clone() {
            let _ = dev.Commit();
        }

        let mut g = win_arc.lock().unwrap();
        g.controller = Some(controller);
        g.comp_ctrl = Some(comp_ctrl);
        g.is_creating = false;

        if let Some((x, y, w, h)) = g.pending_bounds.take() {
            if let (Some(ctrl), Some(vis), Some(dev)) = (
                g.controller.as_ref(),
                g.dcomp_visual.as_ref(),
                g.dcomp_device.as_ref(),
            ) {
                let _ = ctrl.SetBounds(RECT {
                    left: 0,
                    top: 0,
                    right: w as i32,
                    bottom: h as i32,
                });
                let _ = ctrl.SetIsVisible(w > 0.0 && h > 0.0);
                let _ = vis.SetOffsetX2(x as f32);
                let _ = vis.SetOffsetY2(y as f32);
                let _ = dev.Commit();
            }
        }
    }

    tracing::info!("[webview/win] ICoreWebView2CompositionController ready");
}
