use crate::db::mail::eml_cache;
use crate::globals::{IMAP_MANAGER, SECURITY, get_crypto, get_db_pool};
use crate::network::rewriter::rewrite_external_resources;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{State, command};

pub struct EmailViewState {
    pub html: Mutex<Option<String>>,
    pub allow_external: Mutex<bool>,
}

impl Default for EmailViewState {
    fn default() -> Self {
        Self {
            html: Mutex::new(None),
            allow_external: Mutex::new(false),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareEmailViewResult {
    pub has_external_resources: bool,
    pub is_plain_only: bool,
}

/// Builds the full HTML document that the webview will load
fn build_email_html(
    body_html_safe: &str,
    body_plain: &str,
    accent_color: &str,
    view_mode: &str,
) -> (String, bool) {
    let is_plain_only = body_html_safe.trim().is_empty() || view_mode == "plain";

    if is_plain_only {
        let escaped = escape_html(body_plain);
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0; padding: 20px 24px;
      font-family: ui-monospace, "Cascadia Code", "Source Code Pro", Menlo, Consolas, monospace;
      font-size: 13px; line-height: 1.7;
      color: #e2e8f0; background: transparent;
      word-wrap: break-word; overflow-x: hidden;
    }}
    pre {{
      margin: 0; white-space: pre-wrap; word-wrap: break-word;
      border: 1px solid rgba(255,255,255,0.08);
      background: rgba(255,255,255,0.03);
      border-radius: 10px; padding: 20px;
      font-family: inherit; font-size: inherit; color: inherit;
    }}
    ::-webkit-scrollbar {{ width: 8px; height: 8px; }}
    ::-webkit-scrollbar-track {{ background: transparent; }}
    ::-webkit-scrollbar-thumb {{ background: rgba(255,255,255,0.1); border-radius: 4px; }}
  </style>
</head>
<body>
  <pre>{escaped}</pre>
</body>
</html>"#,
            escaped = escaped
        );
        return (html, true);
    }

    let has_dark = body_html_safe.contains("prefers-color-scheme: dark")
        || body_html_safe.contains("data-ogsc")
        || body_html_safe.contains("data-ogsb");

    let iframe_bg = if has_dark { "transparent" } else { "#ffffff" };
    let iframe_text = if has_dark { "inherit" } else { "#1a1a1a" };
    let color_scheme = if has_dark { "dark light" } else { "light" };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0; padding: 24px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      color: {iframe_text}; background: {iframe_bg};
      font-size: 14px; line-height: 1.6; word-wrap: break-word;
      color-scheme: {color_scheme};
      overflow-x: hidden;
    }}
    a {{ color: {accent_color}; text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}
    img, table, td, th {{ max-width: 100% !important; height: auto !important; }}
    pre {{ overflow-x: auto; max-width: 100%; white-space: pre-wrap; }}
    ::-webkit-scrollbar {{ width: 8px; height: 8px; }}
    ::-webkit-scrollbar-track {{ background: transparent; }}
    ::-webkit-scrollbar-thumb {{ background: rgba(0,0,0,0.1); border-radius: 4px; }}
  </style>
</head>
<body>
  <div id="email-wrapper">{body_html_safe}</div>
  <script>
    document.addEventListener('click', (e) => {{
      const a = e.target.closest('a');
      if (!a || !a.href) return;
      e.preventDefault();
      fetch('/message/link?url=' + encodeURIComponent(a.href)).catch(function() {{}});
    }});
  </script>
</body>
</html>"#,
        iframe_text = iframe_text,
        iframe_bg = iframe_bg,
        color_scheme = color_scheme,
        accent_color = accent_color,
        body_html_safe = body_html_safe,
    );

    (html, false)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Fetches the email body from cache/IMAP, builds the full HTML document, processes inline images and external resources, and stores the result in EmailViewState for the protocol handler to serve
#[command]
pub async fn prepare_email_view(
    state: State<'_, EmailViewState>,
    account_id: String,
    mailbox: String,
    uid: u32,
    accent_color: String,
    allow_external: bool,
    view_mode: String,
) -> Result<PrepareEmailViewResult, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Load message metadata from DB to get inline image attachment info
    let msg_meta = crate::db::mail::messages::fetch_message_full(&conn, &account_id, &mailbox, uid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Message not found".to_string())?;

    // Fetch body from encrypted file cache, fall back to IMAP fetch+cache
    let crypto = get_crypto().await?;
    let (body_html, body_plain) = match eml_cache::load_body(&crypto, &account_id, &mailbox, uid) {
        Ok(Some(cached)) => (cached.body_html, cached.body_plain),
        _ => {
            let imap = IMAP_MANAGER.lock().await;
            if let Ok(Some(full)) = imap
                .fetch_and_cache_message(&account_id, &mailbox, uid)
                .await
            {
                (full.body_html_safe, full.body_plain)
            } else {
                (String::new(), String::new())
            }
        }
    };

    // Build the full HTML document — Rust owns the email content, not the frontend
    let (mut html, is_plain_only) =
        build_email_html(&body_html, &body_plain, &accent_color, &view_mode);

    // Resolve inline images: replace cid: references with encrypted-and-decrypted data: URLs
    let inline_images = msg_meta.inline_images;
    if !inline_images.is_empty() {
        let security = SECURITY.lock().await;
        for img in &inline_images {
            let cid = match &img.cid {
                Some(c) if !c.is_empty() => c,
                _ => continue,
            };
            let cached_path = match &img.cached_path {
                Some(p) if !p.is_empty() => p,
                _ => continue,
            };

            let raw_cid = cid.trim_matches(|c| c == '<' || c == '>');

            let data = match std::fs::read(cached_path) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            let decrypted = match security.decrypt(&data) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            let b64 = STANDARD.encode(&decrypted);
            let data_url = format!("data:{};base64,{}", img.mime_type, b64);
            let cid_pattern = format!("cid:{}", raw_cid);
            html = html.replace(&cid_pattern, &data_url);
        }
    }

    // Rewrite or detect external resources
    let rewrite = rewrite_external_resources(&html, allow_external).await;
    html = rewrite.html;

    // Store in state — protocol.rs will serve this on the next webview load
    *state.html.lock().map_err(|e| e.to_string())? = Some(html);
    *state.allow_external.lock().map_err(|e| e.to_string())? = allow_external;

    Ok(PrepareEmailViewResult {
        has_external_resources: rewrite.has_external,
        is_plain_only,
    })
}
