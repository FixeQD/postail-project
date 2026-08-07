use tauri::{command, AppHandle};

#[command]
pub async fn open_system_calendar(app: AppHandle) -> Result<(), String> {
    crate::system::open_system_calendar(&app).await
}
