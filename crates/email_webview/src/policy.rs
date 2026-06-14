use tauri::{Runtime, WebviewWindow};

#[cfg(target_os = "linux")]
use tracing::{info, warn};

#[cfg(target_os = "windows")]
use std::borrow::Cow;
#[cfg(target_os = "windows")]
use std::sync::{Arc, OnceLock};
#[cfg(target_os = "windows")]
use tauri::http::Response;

#[cfg(target_os = "windows")]
type RouteHandler = Arc<dyn Fn(&str, &str) -> Option<Response<Cow<'static, [u8]>>> + Send + Sync>;

#[cfg(target_os = "windows")]
static ROUTE_HANDLER: OnceLock<RouteHandler> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn set_route_handler(handler: RouteHandler) {
    let _ = ROUTE_HANDLER.set(handler);
}

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

#[cfg(target_os = "windows")]
pub fn register_webview2_resource_handler(
    wv: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2,
    env: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
    _app: tauri::AppHandle,
) {
    use crate::state::SendWidget;
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2WebResourceRequestedEventArgs, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
    };
    use webview2_com::WebResourceRequestedEventHandler;
    use windows_wv::core::PWSTR;
    use windows_wv::Win32::Foundation::HGLOBAL;
    use windows_wv::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
    use windows_wv::Win32::System::Com::{CoTaskMemFree, IStream, STREAM_SEEK_SET};

    let Some(route_handler) = ROUTE_HANDLER.get() else {
        tracing::error!("[webview/win] route handler not set");
        return;
    };
    let route_handler = route_handler.clone();

    let filter: Vec<u16> = "http://postail.localhost/*\0".encode_utf16().collect();
    if let Err(e) = unsafe {
        wv.AddWebResourceRequestedFilter(
            windows_wv::core::PCWSTR(filter.as_ptr()),
            COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        )
    } {
        tracing::error!("[webview/win] AddWebResourceRequestedFilter: {e:?}");
        return;
    }

    fn stream_from_bytes(body: &[u8]) -> Option<IStream> {
        unsafe {
            let stream = CreateStreamOnHGlobal(HGLOBAL::default(), true).ok()?;
            if !body.is_empty() {
                let mut written = 0u32;
                if stream
                    .Write(body.as_ptr() as _, body.len() as u32, Some(&mut written))
                    .is_err()
                {
                    return None;
                }
                let _ = stream.Seek(0, STREAM_SEEK_SET, None);
            }
            Some(stream)
        }
    }

    let env = SendWidget(env.clone());
    let handler = WebResourceRequestedEventHandler::create(Box::new(
        move |_wv, args: Option<ICoreWebView2WebResourceRequestedEventArgs>| {
            let args = match args {
                Some(a) => a,
                None => return Ok(()),
            };
            let request = unsafe { args.Request()? };
            let mut uri = PWSTR::null();
            unsafe {
                request.Uri(&mut uri)?;
            }
            if uri.is_null() {
                return Ok(());
            }
            let uri_str = unsafe { uri.to_string() };
            unsafe {
                CoTaskMemFree(Some(uri.0 as _));
            }
            let uri_str = uri_str?;

            let path_and_query = uri_str
                .strip_prefix("http://postail.localhost")
                .unwrap_or("/");
            let (path, query) = path_and_query
                .split_once('?')
                .unwrap_or((path_and_query, ""));

            let response = route_handler(path, query);

            let response = match response {
                Some(resp) => resp,
                None => {
                    let body = b"Not Found".to_vec();
                    if let Some(stream) = stream_from_bytes(&body) {
                        let headers_w: Vec<u16> =
                            "Content-Type: text/plain\r\n\0".encode_utf16().collect();
                        let reason_w: Vec<u16> = "Not Found\0".encode_utf16().collect();
                        if let Ok(resp) = unsafe {
                            env.0.CreateWebResourceResponse(
                                Some(&stream),
                                404,
                                windows_wv::core::PCWSTR(reason_w.as_ptr()),
                                windows_wv::core::PCWSTR(headers_w.as_ptr()),
                            )
                        } {
                            let _ = unsafe { args.SetResponse(&resp) };
                        }
                    }
                    return Ok(());
                }
            };

            let status = response.status().as_u16() as i32;
            let body = response.body().as_ref().to_vec();

            let mut headers = String::new();
            for (name, value) in response.headers().iter() {
                headers.push_str(name.as_str());
                headers.push_str(": ");
                headers.push_str(value.to_str().unwrap_or_default());
                headers.push_str("\r\n");
            }
            headers.push('\0');
            let headers_w: Vec<u16> = headers.encode_utf16().collect();

            let reason = response.status().canonical_reason().unwrap_or("OK");
            let reason_w: Vec<u16> = format!("{reason}\0").encode_utf16().collect();

            let stream: Option<IStream> = stream_from_bytes(&body);

            match unsafe {
                env.0.CreateWebResourceResponse(
                    stream.as_ref(),
                    status,
                    windows_wv::core::PCWSTR(reason_w.as_ptr()),
                    windows_wv::core::PCWSTR(headers_w.as_ptr()),
                )
            } {
                Ok(response) => {
                    let _ = unsafe { args.SetResponse(&response) };
                }
                Err(e) => {
                    tracing::error!("[webview/win] CreateWebResourceResponse: {e:?}");
                }
            }

            Ok(())
        },
    ));

    let mut token = Default::default();
    if let Err(e) = unsafe { wv.add_WebResourceRequested(&handler, &mut token) } {
        tracing::error!("[webview/win] add_WebResourceRequested: {e:?}");
    }
}
