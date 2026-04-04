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

/// Shared move logic: enqueue IMAP MOVE + optimistic local DELETE.
/// Spawns flag queue processing in background.
async fn do_move(
    account_id: &str,
    source_mailbox: &str,
    target_mailbox: &str,
    uids: &[u32],
) -> Result<(), String> {
    {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        for &uid in uids {
            crate::db::mail::flag_queue::enqueue_move_operation(
                &conn,
                account_id,
                source_mailbox,
                target_mailbox,
                uid,
            )
            .map_err(|e| e.to_string())?;
        }

        let placeholders = uids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "DELETE FROM messages WHERE account_id = ? AND mailbox = ? AND uid IN ({})",
            placeholders
        );
        let mut params: Vec<rusqlite::types::Value> = vec![
            account_id.to_string().into(),
            source_mailbox.to_string().into(),
        ];
        for &uid in uids {
            params.push(uid.into());
        }
        conn.execute(&query, rusqlite::params_from_iter(params))
            .map_err(|e| e.to_string())?;
    }

    let account_id_owned = account_id.to_string();
    tokio::spawn(async move {
        if let Err(e) = crate::cmd::mail::actions::process_flag_queue(&account_id_owned).await {
            tracing::error!(target: "postail", "[Move] Failed to process queue: {}", e);
        }
    });

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

#[command]
pub async fn delete_folder(account_id: String, name: String) -> Result<(), String> {
    {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        if is_system_folder(&conn, &account_id, &name) {
            return Err("System folders cannot be deleted".to_string());
        }
    }

    let imap = IMAP_MANAGER.lock().await.clone();
    imap.delete_folder(&account_id, &name)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn move_messages(
    account_id: String,
    source_mailbox: String,
    target_mailbox: String,
    uids: Vec<u64>,
) -> Result<(), String> {
    if source_mailbox == target_mailbox || uids.is_empty() {
        return Ok(());
    }

    let uids_u32: Vec<u32> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect::<Result<_, _>>()?;

    // Verify target exists
    {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM mailboxes WHERE account_id = ? AND name = ?",
                rusqlite::params![&account_id, &target_mailbox],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return Err(format!("Target folder '{}' not found", target_mailbox));
        }
    }

    do_move(&account_id, &source_mailbox, &target_mailbox, &uids_u32).await
}

#[command]
pub async fn archive_messages(
    account_id: String,
    source_mailbox: String,
    uids: Vec<u64>,
) -> Result<(), String> {
    if uids.is_empty() {
        return Ok(());
    }

    let uids_u32: Vec<u32> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect::<Result<_, _>>()?;

    let archive = {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        crate::db::mail::mailbox::get_mailbox_by_role(&conn, &account_id, "archive")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No archive folder configured for this account".to_string())?
    };

    if source_mailbox == archive {
        return Ok(());
    }

    do_move(&account_id, &source_mailbox, &archive, &uids_u32).await
}

#[command]
pub async fn subscribe_folder(account_id: String, name: String) -> Result<(), String> {
    let imap = IMAP_MANAGER.lock().await.clone();
    imap.subscribe_folder(&account_id, &name)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn unsubscribe_folder(account_id: String, name: String) -> Result<(), String> {
    let imap = IMAP_MANAGER.lock().await.clone();
    imap.unsubscribe_folder(&account_id, &name)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn set_folder_hidden(
    account_id: String,
    name: String,
    hidden: bool,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Block hiding system folders
    if is_system_folder(&conn, &account_id, &name) {
        return Err("System folders cannot be hidden".to_string());
    }

    conn.execute(
        "UPDATE mailboxes SET hidden = ? WHERE account_id = ? AND name = ?",
        rusqlite::params![hidden as i64, account_id, name],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
