use tauri::{Runtime, WebviewWindow};
use tracing::{info, warn};

pub fn install_network_block<R: Runtime>(window: &WebviewWindow<R>, proxy_port: u16) {
    #[cfg(target_os = "linux")]
    install_linux(window, proxy_port);

    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        let _ = proxy_port;
    }
}

#[cfg(target_os = "linux")]
fn install_linux<R: Runtime>(window: &WebviewWindow<R>, proxy_port: u16) {
    use webkit2gtk::{NetworkProxyMode, NetworkProxySettings, WebViewExt, WebsiteDataManagerExt};

    let result = window.with_webview(move |webview| {
        let wk = webview.inner();

        let proxy_uri = format!("http://127.0.0.1:{proxy_port}");
        let mut proxy = NetworkProxySettings::new(
            Some(proxy_uri.as_str()),
            &["localhost", "127.0.0.1", "::1", "*.localhost"],
        );

        if let Some(manager) = wk.website_data_manager() {
            manager.set_network_proxy_settings(NetworkProxyMode::Custom, Some(&mut proxy));
            info!("webview network block: null proxy installed on port {proxy_port}");
        } else {
            warn!("webview network block: could not get WebsiteDataManager");
        }
    });

    if let Err(e) = result {
        warn!("webview network block: failed to install: {e}");
    }
}
