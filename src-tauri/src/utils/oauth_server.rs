use crate::globals;
use serde_json;
use std::thread;
use tauri::{AppHandle, Emitter};

pub fn start(handle: AppHandle) {
    thread::spawn(move || {
        let server_addr = format!("127.0.0.1:{}", globals::get_oauth_port());
        let server = match tiny_http::Server::http(&server_addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to start http server for oauth: {}", e);
                return;
            }
        };

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

                let response_html = "<!DOCTYPE html><html><head><title>Postail</title><style>body{display:flex;justify-content:center;align-items:center;height:100vh;margin:0;font-family:sans-serif;background-color:#1a1a1a;color:white;}</style></head><body><div><h1>Authentication successful!</h1><p>You can now close this tab.</p></div></body></html>";
                let response = tiny_http::Response::from_string(response_html);

                let response = match "Content-Type: text/html".parse::<tiny_http::Header>() {
                    Ok(header) => response.with_header(header),
                    Err(err) => {
                        eprintln!("Failed to parse header: {:?}", err);
                        response
                    }
                };

                let _ = request.respond(response);
            } else {
                let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    });
}
