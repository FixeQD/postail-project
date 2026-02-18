use ammonia;
use tauri::{AppHandle, Emitter};
use tiny_http::Response;
use tracing;

use crate::oauth::flow::validate_and_take_state;

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
                let mut has_error = false;
                let mut error_message = String::new();

                if let Ok(url) = url::Url::parse(&url_str) {
                    let query_pairs: std::collections::HashMap<_, _> =
                        url.query_pairs().into_owned().collect();

                    if let Some(error) = query_pairs.get("error") {
                        tracing::error!(target: "postail", "OAuth error: {}", error);
                        has_error = true;
                        let desc = query_pairs
                            .get("error_description")
                            .map(|s| s.as_str())
                            .unwrap_or("");
                        error_message = if error == "access_denied" {
                            "Access was denied by the provider.".to_string()
                        } else {
                            format!("Error: {}", error)
                        };
                        if !desc.is_empty() {
                            error_message.push_str(&format!(" ({})", desc));
                        }

                        let _ = handle.emit(
                            "oauth_error",
                            serde_json::json!({
                                "error": error.to_string(),
                                "error_description": query_pairs.get("error_description").map(|s| s.to_string())
                            }),
                        );
                    } else if let (Some(code), Some(state)) =
                        (query_pairs.get("code"), query_pairs.get("state"))
                    {
                        // Atomically validate and take state to prevent TOCTOU race condition
                        match validate_and_take_state(state) {
                            Some((provider, pkce)) => {
                                let _ = handle.emit(
                                    "oauth_callback",
                                    serde_json::json!({
                                        "code": code.to_string(),
                                        "state": state.to_string(),
                                        "code_verifier": pkce.code_verifier,
                                        "provider_type": provider.kind.as_str()
                                    }),
                                );
                            }
                            None => {
                                has_error = true;
                                tracing::error!(target: "postail", "Invalid or expired OAuth state: {}", state);
                                error_message =
                                    "Invalid or expired authentication request. Please try again."
                                        .to_string();
                                let _ = handle.emit(
                                    "oauth_error",
                                    serde_json::json!({"error": "invalid_state"}),
                                );
                            }
                        }
                    } else {
                        has_error = true;
                        tracing::error!(target: "postail", "Missing code or state in OAuth callback");
                        error_message = "Missing required parameters in the request.".to_string();
                    }
                }

                let mut response_html = include_str!("oauth_status.html").to_string();
                if has_error {
                    response_html =
                        response_html.replace("{{error_message}}", &ammonia::clean(&error_message));
                    response_html = response_html.replace(
                        "</head>",
                        "<script>window.history.replaceState({}, '', '?error=true');</script></head>",
                    );
                }

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
