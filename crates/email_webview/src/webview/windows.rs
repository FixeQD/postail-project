use std::sync::{Arc, Mutex};

use crate::state::{EmbeddedEmailState, WinViewInner};
use crate::watchdog::WatchdogState;
use tauri::{AppHandle, Manager};

use windows_wv::core::Interface;
use windows_wv::Win32::Foundation::{HMODULE, HWND, RECT};
use windows_wv::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows_wv::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
};
use windows_wv::Win32::Graphics::DirectComposition::{
    DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
};
use windows_wv::Win32::Graphics::Dxgi::IDXGIDevice;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2, ICoreWebView2CompositionController,
    ICoreWebView2Controller, ICoreWebView2Environment, ICoreWebView2Environment3,
};
use webview2_com::{
    CreateCoreWebView2CompositionControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler,
};

use windows_wv::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WINDOW_STYLE, WS_CHILD, WS_VISIBLE,
};

pub fn create(app: AppHandle, watchdog: WatchdogState) -> Result<(), String> {
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

fn create_on_main(app: AppHandle, watchdog: WatchdogState) {
    let state = app.state::<EmbeddedEmailState>();
    {
        let mut g = state.win.lock().unwrap();
        if g.comp_ctrl.is_some() {
            drop(g);
            reload_on_main(&app);
            return;
        }
        if g.is_creating {
            tracing::warn!("[webview/win] creation already in progress, ignoring duplicate call");
            return;
        }
        g.is_creating = true;
    }

    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            tracing::error!("[webview/win] main window not found");
            state.win.lock().unwrap().is_creating = false;
            return;
        }
    };

    let hwnd_isize: isize = match window.hwnd() {
        Ok(h) => h.0 as isize,
        Err(e) => {
            tracing::error!("[webview/win] hwnd() failed: {e}");
            state.win.lock().unwrap().is_creating = false;
            return;
        }
    };
    state.win.lock().unwrap().main_hwnd = hwnd_isize;

    let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
    let window_name: Vec<u16> = "\0".encode_utf16().collect();

    let child_hwnd = match unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            windows_wv::core::PCWSTR(class_name.as_ptr()),
            windows_wv::core::PCWSTR(window_name.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            1,
            1,
            Some(HWND(hwnd_isize as *mut _)),
            None,
            None,
            None,
        )
    } {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("[webview/win] CreateWindowExW failed: {e}");
            state.win.lock().unwrap().is_creating = false;
            return;
        }
    };

    let child_hwnd_isize = child_hwnd.0 as isize;
    state.win.lock().unwrap().child_hwnd = child_hwnd_isize;

    let win_arc = state.win.clone();
    let wd = watchdog.clone();
    let app_for_env = app.clone();

    let env_handler =
        CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |hr, env| {
            if hr.is_err() {
                tracing::error!("[webview/win] env creation failed: {hr:?}");
                win_arc.lock().unwrap().is_creating = false;
                return Ok(());
            }
            let env = match env {
                Some(e) => e,
                None => {
                    tracing::error!("[webview/win] env is None");
                    win_arc.lock().unwrap().is_creating = false;
                    return Ok(());
                }
            };

            // Use the captured-at-creation-time HWND
            let child_hwnd = HWND(child_hwnd_isize as *mut _);

            let (dev, tgt, vis) = match setup_dcomp(child_hwnd) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("[webview/win] DComp setup: {e}");
                    win_arc.lock().unwrap().is_creating = false;
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
                    win_arc.lock().unwrap().is_creating = false;
                    return Ok(());
                }
            };

            let win_for_ctrl = win_arc.clone();
            let wd_for_ctrl = wd.clone();
            let env_for_ctrl = crate::state::SendWidget(env.clone());
            let app_for_ctrl = app_for_env.clone();

            let ctrl_handler = CreateCoreWebView2CompositionControllerCompletedHandler::create(
                Box::new(move |hr, comp_ctrl| {
                    on_ctrl_created(
                        hr,
                        comp_ctrl,
                        &win_for_ctrl,
                        &wd_for_ctrl,
                        &env_for_ctrl.0,
                        &app_for_ctrl,
                    );
                    Ok(())
                }),
            );

            unsafe {
                if let Err(e) =
                    env3.CreateCoreWebView2CompositionController(child_hwnd, &ctrl_handler)
                {
                    tracing::error!("[webview/win] CreateCompositionController: {e:?}");
                    win_arc.lock().unwrap().is_creating = false;
                }
            }
            Ok(())
        }));

    unsafe {
        if let Err(e) = CreateCoreWebView2EnvironmentWithOptions(
            windows_wv::core::PCWSTR::null(),
            windows_wv::core::PCWSTR::null(),
            None,
            &env_handler,
        ) {
            tracing::error!("[webview/win] CreateEnvironmentWithOptions: {e:?}");
            state.win.lock().unwrap().is_creating = false;
        }
    }
}

fn on_ctrl_created(
    hr: windows_wv::core::Result<()>,
    comp_ctrl: Option<ICoreWebView2CompositionController>,
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
                tracing::info!(pid, "email renderer PID captured via BrowserProcessId");
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
        g.is_creating = false; // creation complete

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

fn update_bounds_on_main(app: &AppHandle, x: f64, y: f64, w: f64, h: f64) {
    let state = app.state::<EmbeddedEmailState>();
    let mut guard = state.win.lock().unwrap();
    let (Some(ctrl), Some(vis), Some(dev)) = (
        guard.controller.as_ref(),
        guard.dcomp_visual.as_ref(),
        guard.dcomp_device.as_ref(),
    ) else {
        guard.pending_bounds = Some((x, y, w, h));
        return;
    };

    unsafe {
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

fn destroy_on_main(app: &AppHandle) {
    let state = app.state::<EmbeddedEmailState>();
    let mut guard = state.win.lock().unwrap();
    unsafe {
        if let Some(ctrl) = guard.controller.as_ref() {
            let _ = ctrl.SetIsVisible(false);
            let _ = ctrl.Close();
        }
        if let Some(dev) = guard.dcomp_device.as_ref() {
            let _ = dev.Commit();
        }
        if guard.child_hwnd != 0 {
            let _ = DestroyWindow(HWND(guard.child_hwnd as *mut _));
            guard.child_hwnd = 0;
        }
    }
    guard.comp_ctrl = None;
    guard.controller = None;
    guard.dcomp_device = None;
    guard.dcomp_target = None;
    guard.dcomp_visual = None;
    guard.is_creating = false;
    tracing::info!("[webview/win] email WebView destroyed (DComp)");
}

fn reload_on_main(app: &AppHandle) {
    let state = app.state::<EmbeddedEmailState>();
    let guard = state.win.lock().unwrap();
    if let Some(ctrl) = guard.controller.as_ref() {
        unsafe {
            if let Ok(wv) = ctrl.CoreWebView2() {
                navigate_to_email(&wv);
                tracing::info!("[webview/win] email WebView reloaded");
            }
        }
    }
}

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
            HMODULE::default(),
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

#[inline]
unsafe fn navigate_to_email(wv: &ICoreWebView2) {
    let url: Vec<u16> = "http://postail.localhost/message/current\0"
        .encode_utf16()
        .collect();
    let _ = wv.Navigate(windows_wv::core::PCWSTR(url.as_ptr()));
}
