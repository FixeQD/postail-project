use crate::email_view::EmailViewState;
use crate::utils::sanitizer::{
    auto_fix_email_html as sanitizer_fix, sanitize_email_html_with_details, SanitizeResult,
};
use tauri::{command, State};
use tauri_plugin_notification::NotificationExt;

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
pub fn set_email_view_content(
    state: State<'_, EmailViewState>,
    html: String,
    allow_external: bool,
) -> Result<(), String> {
    *state.html.lock().map_err(|e| e.to_string())? = Some(html);
    *state.allow_external.lock().map_err(|e| e.to_string())? = allow_external;
    Ok(())
}
