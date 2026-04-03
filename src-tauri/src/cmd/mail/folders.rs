use crate::globals::IMAP_MANAGER;
use tauri::command;

fn validate_folder_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Folder name cannot be empty".to_string());
    }
    if name.len() > 255 {
        return Err("Folder name is too long".to_string());
    }
    // IMAP forbids these in folder names
    for ch in ['\0', '\r', '\n'] {
        if name.contains(ch) {
            return Err(format!("Folder name contains illegal character"));
        }
    }
    Ok(())
}

#[command]
pub async fn create_folder(account_id: String, name: String) -> Result<(), String> {
    validate_folder_name(&name)?;

    let imap = IMAP_MANAGER.lock().await.clone();
    imap.create_folder(&account_id, &name)
        .await
        .map_err(|e| e.to_string())
}
