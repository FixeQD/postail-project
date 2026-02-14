use crate::db::settings;
use crate::utils::config::{get_default_data_dir as get_default_path, ThemeConfig};
use std::collections::HashMap;

#[tauri::command]
pub async fn get_default_data_dir() -> Result<String, String> {
    Ok(get_default_path().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_all_settings() -> Result<HashMap<String, String>, String> {
    let mut all = settings::get_all_settings()
        .await
        .map_err(|e| e.to_string())?;

    let data_path = crate::utils::config::get_data_dir();
    all.insert(
        "data-path".to_string(),
        data_path.to_string_lossy().to_string(),
    );

    Ok(all)
}

#[tauri::command]
pub async fn get_setting(key: String) -> Result<Option<String>, String> {
    settings::get_setting(&key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(key: String, value: String) -> Result<(), String> {
    settings::set_setting(&key, &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn migrate_data_path(
    app_handle: tauri::AppHandle,
    new_path: String,
) -> Result<(), String> {
    crate::utils::migration::perform_migration(&new_path).await?;
    app_handle.restart();
}

#[tauri::command]
pub async fn get_theme_config() -> Result<ThemeConfig, String> {
    Ok(crate::utils::config::load_theme_config())
}

#[tauri::command]
pub async fn set_theme_config(
    accent_color: String,
    background: String,
    animations_enabled: bool,
) -> Result<(), String> {
    let theme = ThemeConfig {
        accent_color,
        background,
        animations_enabled,
    };
    crate::utils::config::save_theme_config(&theme)
}
