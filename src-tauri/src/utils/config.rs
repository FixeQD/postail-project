use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub fn get_default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
}

pub fn get_data_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail");

    let override_file = config_dir.join("data_path");
    if override_file.exists() {
        if let Ok(content) = fs::read_to_string(&override_file) {
            let path = PathBuf::from(content.trim());
            if path.is_absolute() {
                return path;
            }
        }
    }

    get_default_data_dir()
}

pub fn set_data_dir_override(path: &str) -> std::io::Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail");

    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("data_path"), path.trim())?;
    Ok(())
}

// ─── App config (persisted as JSON next to the DB) ───

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub security_method: String,
}

pub fn get_config_path() -> PathBuf {
    get_data_dir().join("config.json")
}

pub fn load_config() -> Option<AppConfig> {
    let path = get_config_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}
