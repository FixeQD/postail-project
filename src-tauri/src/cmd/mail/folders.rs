use crate::globals::{IMAP_MANAGER, get_db_pool};
use tauri::command;

const SYSTEM_ROLES: &[&str] = &["inbox", "sent", "drafts", "trash", "archive", "junk"];

fn is_system_folder(conn: &rusqlite::Connection, account_id: &str, name: &str) -> bool {
    conn.query_row(
        "SELECT role FROM mailboxes WHERE account_id = ? AND name = ?",
        rusqlite::params![account_id, name],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .map(|role| SYSTEM_ROLES.contains(&role.as_str()))
    .unwrap_or(false)
}

fn validate_folder_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Folder name cannot be empty".to_string());
    }
    if name.len() > 255 {
        return Err("Folder name is too long".to_string());
    }
    for ch in ['\0', '\r', '\n'] {
        if name.contains(ch) {
            return Err("Folder name contains illegal character".to_string());
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

#[command]
pub async fn rename_folder(
    account_id: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    validate_folder_name(&new_name)?;

    if old_name == new_name {
        return Ok(());
    }

    {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        if is_system_folder(&conn, &account_id, &old_name) {
            return Err("System folders cannot be renamed".to_string());
        }
    }

    let imap = IMAP_MANAGER.lock().await.clone();
    imap.rename_folder(&account_id, &old_name, &new_name)
        .await
        .map_err(|e| e.to_string())
}
