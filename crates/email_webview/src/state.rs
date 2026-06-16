use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2CompositionController, ICoreWebView2Controller,
};
#[cfg(target_os = "windows")]
use windows_wv::Win32::Graphics::DirectComposition::{
    IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
};

pub struct SendWidget<T>(pub T);
unsafe impl<T> Send for SendWidget<T> {}
unsafe impl<T> Sync for SendWidget<T> {}

#[cfg(target_os = "windows")]
pub struct WinViewInner {
    pub comp_ctrl: Option<ICoreWebView2CompositionController>,
    pub controller: Option<ICoreWebView2Controller>,
    pub dcomp_device: Option<IDCompositionDevice>,
    pub dcomp_target: Option<IDCompositionTarget>,
    pub dcomp_visual: Option<IDCompositionVisual>,
    pub main_hwnd: isize,
    pub child_hwnd: isize,
    pub pending_bounds: Option<(f64, f64, f64, f64)>,
    pub is_creating: bool,
    pub webview2_thread_id: u32,
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
            child_hwnd: 0,
            pending_bounds: None,
            is_creating: false,
            webview2_thread_id: 0,
        }
    }
}

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
