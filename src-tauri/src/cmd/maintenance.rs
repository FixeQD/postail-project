use crate::db::Contact;
use crate::globals::{DB_CONN, SECURITY};
use std::sync::Arc;
use tauri::command;

#[command]
pub async fn export_backup(passphrase: Option<String>) -> Result<String, String> {
    let db_conn = Arc::clone(&DB_CONN);
    let security = Arc::clone(&SECURITY);
    let passphrase_clone = passphrase;
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        let sec = security.blocking_lock();
        crate::db::export_backup(conn, &sec, passphrase_clone)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn import_backup(backup_path: String, passphrase: Option<String>) -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    let security = Arc::clone(&SECURITY);
    let path = std::path::PathBuf::from(backup_path);
    let passphrase_clone = passphrase;
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        let sec = security.blocking_lock();
        crate::db::import_backup(conn, &sec, &path, passphrase_clone).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn run_maintenance() -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::run_maintenance(conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn search_contacts(query: String, limit: u32) -> Result<Vec<Contact>, String> {
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    crate::db::search_contacts(conn, &query, limit).map_err(|e| e.to_string())
}
