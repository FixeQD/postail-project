use std::collections::HashMap;
use crate::db::settings;
use crate::utils::config::get_default_data_dir as get_default_path;

#[tauri::command]
pub async fn get_default_data_dir() -> Result<String, String> {
    Ok(get_default_path().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_all_settings() -> Result<HashMap<String, String>, String> {
    let mut all = settings::get_all_settings().map_err(|e| e.to_string())?;
    
    let data_path = crate::utils::config::get_data_dir();
    all.insert("data-path".to_string(), data_path.to_string_lossy().to_string());
    
    Ok(all)
}

#[tauri::command]
pub async fn get_setting(key: String) -> Result<Option<String>, String> {
    settings::get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(key: String, value: String) -> Result<(), String> {
    settings::set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn migrate_data_path(app_handle: tauri::AppHandle, new_path: String) -> Result<(), String> {
    crate::utils::migration::perform_migration(&new_path).await?;
    app_handle.restart();
}
