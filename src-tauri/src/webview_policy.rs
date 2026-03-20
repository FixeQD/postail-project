/// Block all outbound http/https requests from the main WebView.
use tauri::{Runtime, WebviewWindow};
use tracing::{info, warn};

pub fn install_network_block<R: Runtime>(window: &WebviewWindow<R>) {
    #[cfg(target_os = "linux")]
    install_linux(window);

    #[cfg(target_os = "windows")]
    install_windows(window);

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        warn!("webview network block: not implemented for this platform, relying on CSP only");
    }
}

#[cfg(target_os = "linux")]
fn install_linux<R: Runtime>(window: &WebviewWindow<R>) {
    use webkit2gtk::glib::Cast;
    use webkit2gtk::{
        NavigationAction, NavigationPolicyDecision, NavigationPolicyDecisionExt, PolicyDecision,
        PolicyDecisionExt, PolicyDecisionType, URIRequest, URIRequestExt, WebViewExt,
    };

    let result = window.with_webview(|webview| {
        let wk = webview.inner();

        wk.connect_decide_policy(|_view, decision: &PolicyDecision, decision_type| {
            match decision_type {
                PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction => {
                    if let Some(nav) = decision.downcast_ref::<NavigationPolicyDecision>() {
                        if let Some(action) = nav.navigation_action() {
                            let action: NavigationAction = action;
                            if let Some(req) = action.request() {
                                let req: URIRequest = req;
                                let uri = req.uri().unwrap_or_default();
                                let uri_str = uri.as_str();
                                if uri_str.starts_with("http://") || uri_str.starts_with("https://")
                                {
                                    if uri_str.starts_with("http://localhost")
                                        || uri_str.starts_with("http://127.0.0.1")
                                        || uri_str.starts_with("https://localhost")
                                        || uri_str.starts_with("https://127.0.0.1")
                                        || uri_str.contains(".localhost")
                                    {
                                        return false;
                                    }

                                    nav.ignore();
                                    tracing::warn!(
                                        "webview network block: blocked navigation to {uri_str}"
                                    );
                                    return true;
                                }
                            }
                        }
                    }
                    false
                }
                _ => false,
            }
        });

        info!("webview network block: installed on Linux (webkit2gtk decide-policy)");
    });

    if let Err(e) = result {
        warn!("webview network block: failed to install on Linux: {e}");
    }
}

#[cfg(target_os = "windows")]
fn install_windows<R: Runtime>(window: &WebviewWindow<R>) {
    use tracing::warn;

    // WebView2 exposes AddWebResourceRequestedFilter via ICoreWebView2_2.
    let result = window.with_webview(|webview| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_2, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        };
        use windows::core::Interface;

        let core: ICoreWebView2_2 = unsafe {
            webview
                .controller()
                .CoreWebView2()
                .expect("CoreWebView2")
                .cast()
                .expect("ICoreWebView2_2")
        };

        // Register wildcard filters for http and https
        unsafe {
            core.AddWebResourceRequestedFilter(
                windows::core::w!("http://*"),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            )
            .ok();
            core.AddWebResourceRequestedFilter(
                windows::core::w!("https://*"),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            )
            .ok();
        }

        // WebResourceRequested handler — cancel unless it's localhost/asset.localhost
        let token = unsafe {
            core.add_WebResourceRequested(&webview2_com::WebResourceRequestedEventHandler::create(
                Box::new(|_sender, args| {
                    if let Some(args) = args {
                        if let Ok(req) = args.Request() {
                            if let Ok(uri) = req.Uri() {
                                let uri_str = uri.to_string();
                                if (uri_str.starts_with("http://")
                                    || uri_str.starts_with("https://"))
                                    && !uri_str.starts_with("http://localhost")
                                    && !uri_str.starts_with("https://localhost")
                                    && !uri_str.contains(".localhost")
                                {
                                    tracing::warn!(
                                        "webview network block: blocked request to {uri_str}"
                                    );
                                    // Set an empty response to cancel
                                    // There's no direct cancel — we set a 403 response instead
                                    if let Ok(env) = args.Environment() {
                                        if let Ok(response) = env.CreateWebResourceResponse(
                                            None,
                                            403,
                                            windows::core::w!("Blocked"),
                                            windows::core::w!(""),
                                        ) {
                                            args.SetResponse(response).ok();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                }),
            ))
        };

        match token {
            Ok(_) => info!("webview network block: installed on Windows (WebResourceRequested)"),
            Err(e) => warn!("webview network block: failed to install handler on Windows: {e:?}"),
        }
    });

    if let Err(e) = result {
        warn!("webview network block: failed to install on Windows: {e}");
    }
}
