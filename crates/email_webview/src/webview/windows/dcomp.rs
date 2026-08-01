use windows_wv::core::Interface;
use windows_wv::Win32::Foundation::HMODULE;
use windows_wv::Win32::Foundation::HWND;
use windows_wv::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows_wv::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
};
use windows_wv::Win32::Graphics::DirectComposition::{
    DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
};
use windows_wv::Win32::Graphics::Dxgi::IDXGIDevice;

use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;

// ── Helpers ──────────────────────────────────────────────────────────────────

pub(super) fn setup_dcomp(
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
        let dxgi: IDXGIDevice = d3d.cast().map_err(|e| format!("IDXGIDevice: {e:?}"))?;
        let dev: IDCompositionDevice = DCompositionCreateDevice(Some(&dxgi))
            .map_err(|e| format!("DCompositionCreateDevice: {e:?}"))?;

        let target: IDCompositionTarget = dev
            .CreateTargetForHwnd(hwnd, true)
            .map_err(|e| format!("CreateTargetForHwnd: {e:?}"))?;

        let root: IDCompositionVisual = dev
            .CreateVisual()
            .map_err(|e| format!("CreateVisual(root): {e:?}"))?;
        let email_vis: IDCompositionVisual = dev
            .CreateVisual()
            .map_err(|e| format!("CreateVisual(email): {e:?}"))?;

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
pub(super) unsafe fn navigate_to_email(wv: &ICoreWebView2) {
    let url: Vec<u16> = "http://postail.localhost/message/current\0"
        .encode_utf16()
        .collect();
    let _ = wv.Navigate(windows_wv::core::PCWSTR(url.as_ptr()));
}
