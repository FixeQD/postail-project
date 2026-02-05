use std::collections::HashMap;
use crate::db::settings;

#[tauri::command]
pub async fn get_all_settings() -> Result<HashMap<String, String>, String> {
    settings::get_all_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting(key: String) -> Result<Option<String>, String> {
    settings::get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(key: String, value: String) -> Result<(), String> {
    if key == "data-path" {
        crate::utils::config::set_data_dir_override(&value).map_err(|e| e.to_string())?;
    }
    settings::set_setting(&key, &value).map_err(|e| e.to_string())
}
