use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewBuilder, WebviewUrl, Window, command,
};
use tracing::{info, warn};

#[command]
pub async fn create_email_webview(app: AppHandle, window: Window) -> Result<(), String> {
    if let Some(existing) = app.get_webview("email-webview") {
        info!("Email webview already exists, closing it first.");
        let _ = existing.close();
    }

    info!("Creating new child webview for email content");

    // We use External to point to our custom protocol
    let url = "postail://localhost/message/current"
        .parse()
        .map_err(|e| format!("Failed to parse webview URL: {}", e))?;

    let builder = WebviewBuilder::new("email-webview", WebviewUrl::External(url));

    // Build and attach as a child to the provided window
    window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(100.0, 100.0),
        )
        .map_err(|e| {
            warn!("Failed to build child webview: {}", e);
            format!("Failed to build child webview: {}", e)
        })?;

    Ok(())
}
