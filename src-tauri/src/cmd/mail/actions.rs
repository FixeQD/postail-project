use crate::db;
use crate::globals::DB_CONN;
use tauri::command;

#[command]
pub fn search_messages(
    account_id: Option<String>,
    mailbox: Option<String>,
    query: String,
    limit: u32,
) -> Result<Vec<db::search::SearchResult>, String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db::search_messages(
        conn,
        account_id.as_deref(),
        mailbox.as_deref(),
        &query,
        limit,
    )
    .map_err(|e| e.to_string())
}

#[command]
pub fn mark_read(
    account_id: String,
    mailbox: String,
    uids: Vec<u64>,
    read: bool,
) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;
    db::mark_read(conn, &account_id, &mailbox, &uids, read).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn move_to_trash(account_id: String, mailbox: String, uids: Vec<u64>) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;
    db::move_to_trash(conn, &account_id, &mailbox, &uids).map_err(|e| e.to_string())?;
    Ok(())
}
