use std::sync::{Arc, Mutex};

use crate::state::WinViewInner;

use windows_wv::Win32::Foundation::{HWND, RECT};
use windows_wv::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
};

use super::dcomp::navigate_to_email;

// ── WebView2-thread-local operations ────────────────────────────────────────

pub(super) fn update_bounds_on_wv_thread(
    win_arc: &Arc<Mutex<WinViewInner>>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) {
    let mut guard = win_arc.lock().unwrap();
    let (Some(ctrl), Some(vis), Some(dev)) = (
        guard.controller.as_ref(),
        guard.dcomp_visual.as_ref(),
        guard.dcomp_device.as_ref(),
    ) else {
        tracing::info!(
            x, y, w, h,
            "[webview/win] update_bounds_on_wv_thread: no controller yet, resizing host hwnd anyway, stashed in pending_bounds"
        );
        // Resize/reposition the actual host window, even without a controller yet, instead of leaving it at the degenerate 1x1 it was created at for the entire async environment/controller setup
        let child_hwnd = guard.child_hwnd;
        if child_hwnd != 0 {
            unsafe {
                let _ = SetWindowPos(
                    HWND(child_hwnd as *mut _),
                    None,
                    x as i32,
                    y as i32,
                    w as i32,
                    h as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        guard.pending_bounds = Some((x, y, w, h));
        return;
    };
    tracing::info!(
        x,
        y,
        w,
        h,
        "[webview/win] update_bounds_on_wv_thread: applying directly"
    );
    let child_hwnd = guard.child_hwnd;
    unsafe {
        // See the comment in on_ctrl_created: SetBounds only sizes WebView2's content inside the DComp tree, it never resizes the host Win32 window itself
        if child_hwnd != 0 {
            let _ = SetWindowPos(
                HWND(child_hwnd as *mut _),
                None,
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        let _ = ctrl.SetBounds(RECT {
            left: 0,
            top: 0,
            right: w as i32,
            bottom: h as i32,
        });
        let _ = ctrl.SetIsVisible(w > 0.0 && h > 0.0);
        let _ = vis.SetOffsetX2(0.0);
        let _ = vis.SetOffsetY2(0.0);
        let _ = dev.Commit();
        // Separate from Bounds - tells WebView the host ancestor HWND moved/resized.
        let _ = ctrl.NotifyParentWindowPositionChanged();
    }
}

pub(super) fn destroy_on_wv_thread(win_arc: &Arc<Mutex<WinViewInner>>) {
    let mut guard = win_arc.lock().unwrap();
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
    guard.webview2_thread_id = 0;
    tracing::info!("[webview/win] email WebView destroyed (DComp)");
}

pub(super) fn reload_on_wv_thread(win_arc: &Arc<Mutex<WinViewInner>>) {
    let guard = win_arc.lock().unwrap();
    if let Some(ctrl) = guard.controller.as_ref() {
        unsafe {
            if let Ok(wv) = ctrl.CoreWebView2() {
                navigate_to_email(&wv);
                tracing::info!("[webview/win] email WebView reloaded");
            }
        }
    }
}
