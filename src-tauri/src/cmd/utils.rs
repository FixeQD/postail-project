use crate::db::eml_cache;
use crate::email_view::EmailViewState;
use crate::globals::SECURITY;
use crate::utils::sanitizer::{
    auto_fix_email_html as sanitizer_fix, sanitize_email_html_with_details, SanitizeResult,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use tauri::{command, State};
use tauri_plugin_notification::NotificationExt;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineImageInfo {
    pub cid: String,
    pub cached_path: String,
    pub mime_type: String,
}

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
pub fn process_email_content(html: String) -> SanitizeResult {
    sanitize_email_html_with_details(&html)
}

#[command]
pub fn auto_fix_email_html(html: String) -> String {
    sanitizer_fix(&html)
}

#[command]
pub fn show_notification(app: tauri::AppHandle, title: String, body: String) {
    let _ = app
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show();
}

#[command]
pub async fn set_email_view_content(
    state: State<'_, EmailViewState>,
    html: String,
    inline_images: Vec<InlineImageInfo>,
    allow_external: bool,
) -> Result<(), String> {
    let mut processed = html;

    if !inline_images.is_empty() {
        let security = SECURITY.lock().await;

        for img in &inline_images {
            if img.cached_path.is_empty() || img.cid.is_empty() {
                continue;
            }

            let raw_cid = img.cid.trim_matches(|c| c == '<' || c == '>');

            let data = match std::fs::read(&img.cached_path) {
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
            processed = processed.replace(&cid_pattern, &data_url);
        }
    }

    *state.html.lock().map_err(|e| e.to_string())? = Some(processed);
    *state.allow_external.lock().map_err(|e| e.to_string())? = allow_external;

    Ok(())
}
