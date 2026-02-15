use tauri::{AppHandle, Emitter};
use tiny_http::Response;
use tracing;

pub fn start(handle: AppHandle) {
    let port = portpicker::pick_unused_port().unwrap_or(8765);
    crate::globals::set_oauth_port(port);

    std::thread::spawn(move || {
        let server_addr = format!("127.0.0.1:{}", port);
        let server = match tiny_http::Server::http(&server_addr) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(target: "postail", "Failed to start http server for oauth: {}", e);
                return;
            }
        };

        tracing::info!(target: "postail", "OAuth server started on {}", server_addr);

        for request in server.incoming_requests() {
            if request.url().starts_with("/oauth/callback") {
                let url_str = format!("http://localhost{}", request.url());
                if let Ok(url) = url::Url::parse(&url_str) {
                    let query_pairs: std::collections::HashMap<_, _> =
                        url.query_pairs().into_owned().collect();

                    if let (Some(code), Some(state)) =
                        (query_pairs.get("code"), query_pairs.get("state"))
                    {
                        let _ = handle.emit(
                            "oauth_callback",
                            serde_json::json!({ "code": code.to_string(), "state": state.to_string() }),
                        );
                    }
                }

                let response_html = include_str!("oauth_success.html");
                let response = Response::from_string(response_html);

                let response = match "Content-Type: text/html".parse::<tiny_http::Header>() {
                    Ok(header) => response.with_header(header),
                    Err(err) => {
                        tracing::warn!(target: "postail", "Failed to parse header: {:?}", err);
                        response
                    }
                };

                let _ = request.respond(response);
                break;
            } else {
                let response = Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    });
}
