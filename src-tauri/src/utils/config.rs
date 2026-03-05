use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use dunce::canonicalize;
#[cfg(not(target_os = "windows"))]
use std::fs::canonicalize;

pub fn get_default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
}

pub fn get_data_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail");

    let mut data_dir = get_default_data_dir();

    let override_file = config_dir.join("data_path");
    if override_file.exists() {
        if let Ok(content) = fs::read_to_string(&override_file) {
            let path = PathBuf::from(content.trim());
            if path.is_absolute() {
                data_dir = path;
            } else {
                // Handle relative paths from the config dir
                data_dir = config_dir.join(path);
            }
        }
    }

    if let Ok(canonical) = canonicalize(&data_dir) {
        canonical
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(&data_dir))
            .unwrap_or(data_dir)
    }
}

pub fn set_data_dir_override(path: &str) -> std::io::Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail");

    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("data_path"), path.trim())?;
    Ok(())
}

// ─── Theme config ───

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeConfig {
    pub accent_color: String,
    pub background: String,
    #[serde(default = "default_true")]
    pub animations_enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            accent_color: "#f97316".to_string(),
            background: "slate".to_string(),
            animations_enabled: true,
        }
    }
}

// ─── App config (persisted as JSON next to the DB) ───

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub security_method: String,
    #[serde(default)]
    pub theme: Option<ThemeConfig>,
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

// ─── Theme-specific helpers (work before DB unlock) ───

pub fn load_theme_config() -> ThemeConfig {
    load_config().and_then(|c| c.theme).unwrap_or_default()
}

pub fn save_theme_config(theme: &ThemeConfig) -> Result<(), String> {
    let mut config = load_config().unwrap_or_else(|| AppConfig {
        security_method: String::new(),
        theme: None,
    });
    config.theme = Some(theme.clone());
    save_config(&config)
}
