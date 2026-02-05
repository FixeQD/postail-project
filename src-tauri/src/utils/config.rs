use std::path::PathBuf;
use std::fs;

pub fn get_data_dir() -> PathBuf {
    // Check for override in ~/.config/postail/data_path
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

    // Default
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
}

pub fn set_data_dir_override(path: &str) -> std::io::Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail");
    
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("data_path"), path.trim())?;
    Ok(())
}
