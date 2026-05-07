use crate::cmd::watchdog::WatchdogState;
use crate::email_view::EmailViewState;
use serde_json;
use std::borrow::Cow;
use std::time::Instant;
use tauri::{
    Emitter, Manager, Runtime, UriSchemeContext,
    http::{Request, Response},
};

pub fn handler<R: Runtime>(
    context: UriSchemeContext<R>,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path();

    if path == "/message/current" {
        return serve_email(&context);
    }

    if path == "/message/heartbeat" {
        return handle_heartbeat(&context, request);
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

fn handle_heartbeat<R: Runtime>(
    context: &UriSchemeContext<R>,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let app = context.app_handle();
    let state = app.state::<WatchdogState>();

    let query = request.uri().query().unwrap_or("");
    let mut token = "";
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "token" {
                token = v;
                break;
            }
        }
    }

    match crate::cmd::watchdog::email_heartbeat(&state, token) {
        Ok(next) => Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Cow::Owned(
                serde_json::json!({ "token": next })
                    .to_string()
                    .into_bytes(),
            ))
            .unwrap(),
        Err(_) => Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Cow::Borrowed(b"{}" as &[u8]))
            .unwrap(),
    }
}

fn serve_email<R: Runtime>(context: &UriSchemeContext<R>) -> Response<Cow<'static, [u8]>> {
    let app = context.app_handle();

    let mut html = app
        .state::<EmailViewState>()
        .html
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_default();

    // A new token is minted on every serve so stale heartbeats from a previous email load are automatically invalidated
    let token = uuid::Uuid::new_v4().to_string();

    {
        let watchdog = app.state::<WatchdogState>();
        let mut data = watchdog.0.lock().unwrap();
        data.token = token.clone();
        data.last_heartbeat = Instant::now();
        data.created_at = Instant::now();
        data.is_frozen = false;
    }

    // Inject the heartbeat loop as the very first script in the document
    let script = format!(
        r#"<script>
(function(){{
  var token = {token_json};
  setInterval(function(){{
    fetch('/message/heartbeat?token=' + encodeURIComponent(token))
    .then(function(res) {{ return res.json(); }})
    .then(function(data) {{ if (data && data.token) token = data.token; }})
    .catch(function(err){{ console.error('Heartbeat fetch failed:', err); }});
  }}, 100);
}})();
</script>"#,
        token_json = serde_json::to_string(&token).unwrap_or_default(),
    );

    // Inject before </head> when present; fall back to prepending to the document so it always executes even on bare / malformed HTML
    let html = if let Some(pos) = html.to_lowercase().find("</head>") {
        let mut out = String::with_capacity(html.len() + script.len());
        out.push_str(&html[..pos]);
        out.push_str(&script);
        out.push_str(&html[pos..]);
        out
    } else {
        script + &html
    };

    let csp = "default-src 'none'; \
               script-src 'unsafe-inline'; \
               style-src 'unsafe-inline' data:; \
               img-src data: cid: asset: postail: http://postail.localhost; \
               font-src data: asset:; \
               connect-src 'self' postail: http://postail.localhost;";

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Content-Security-Policy", csp)
        .header("Cache-Control", "no-store")
        .body(Cow::Owned(html.into_bytes()))
        .expect("Failed to create email response")
}
