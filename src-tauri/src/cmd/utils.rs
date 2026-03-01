use crate::utils::sanitizer::{
    auto_fix_email_html as sanitizer_fix, sanitize_email_html_with_details, SanitizeResult,
};
use tauri::command;
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
