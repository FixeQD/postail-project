use crate::db::Draft;
use crate::globals::DB_CONN;
use std::sync::Arc;
use tauri::command;

#[command]
pub async fn save_draft(draft: Draft) -> Result<(), String> {
    let body_len = draft.body.as_ref().map(|b| b.len()).unwrap_or(0);
    tracing::info!(target: "postail", "[save_draft] Received draft from frontend - id={}, subject={:?}, body_len={}, to_count={}, cc_count={}, bcc_count={}",
        draft.id, draft.subject, body_len, draft.to.len(), draft.cc.len(), draft.bcc.len());

    let db_conn = Arc::clone(&DB_CONN);
    let _ = tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::save_draft(conn, &draft).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    tracing::info!(target: "postail", "[save_draft] Draft saved successfully");
    Ok(())
}

#[command]
pub async fn list_drafts(account_id: String) -> Result<Vec<Draft>, String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::list_drafts(conn, &account_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn delete_draft(id: String) -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.lock().unwrap();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::delete_draft(conn, &id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
