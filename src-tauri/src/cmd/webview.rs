use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewBuilder, WebviewUrl, Window, command,
};
use tracing::{info, warn};

#[command]
pub async fn create_email_webview(app: AppHandle, window: Window) -> Result<(), String> {
    if let Some(existing) = app.get_webview("email-webview") {
        info!("Email webview already exists, closing it first.");
        let _ = existing.close();
        // Small yield so the close propagates before we recreate
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }

    info!("Creating new child webview for email content");

    let url = "postail://localhost/message/current"
        .parse()
        .map_err(|e| format!("Failed to parse webview URL: {}", e))?;

    let builder = WebviewBuilder::new("email-webview", WebviewUrl::External(url));

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

#[command]
pub fn update_email_webview_bounds(
    app: AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let Some(webview) = app.get_webview("email-webview") else {
        // Bounds arrived before the webview was ready — silently ignore.
        return Ok(());
    };

    webview
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| format!("set_position failed: {e}"))?;

    webview
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| format!("set_size failed: {e}"))?;

    Ok(())
}

#[command]
pub fn destroy_email_webview(app: AppHandle) {
    if let Some(webview) = app.get_webview("email-webview") {
        info!("Destroying email webview");
        let _ = webview.close();
    }
}
