use crate::email_view::EmailViewState;
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

    if path == "/message/current" {
        return serve_email(&context);
    }

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

fn serve_email<R: Runtime>(context: &UriSchemeContext<R>) -> Response<Cow<'static, [u8]>> {
    let state = context.app_handle().state::<EmailViewState>();

    let html = state
        .html
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_default();

    let allow_external = state
        .allow_external
        .lock()
        .ok()
        .map(|g| *g)
        .unwrap_or(false);

    let csp = if allow_external {
        "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; \
         img-src * data: cid: asset:; font-src * data: asset:; connect-src 'none';"
    } else {
        "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; \
         img-src data: cid: asset:; font-src data: asset:; connect-src 'none';"
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Content-Security-Policy", csp)
        .header("Cache-Control", "no-store")
        .body(Cow::Owned(html.into_bytes()))
        .expect("Failed to create email response")
}
