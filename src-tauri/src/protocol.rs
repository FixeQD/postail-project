use serde_json;
use std::borrow::Cow;
use tauri::{
    http::{Request, Response},
    Emitter, Manager, Runtime, UriSchemeContext,
};

pub fn handler<R: Runtime>(
    context: UriSchemeContext<R>,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path();
    if path == "/window/maximize" {
        if let Some(window) = context.app_handle().get_webview_window("main") {
            let _ = window.maximize();
        }
    } else if path == "/oauth/callback" {
        if let Some(query) = request.uri().query() {
            let params: std::collections::HashMap<_, _> =
                url::form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect();
            if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
                let _ = context.app_handle().emit(
                    "oauth_callback",
                    serde_json::json!({
                        "code": code,
                        "state": state
                    }),
                );
            }
        }
    }

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(Cow::Borrowed(b"OK" as &[u8]))
        .expect("Failed to create response")
}
