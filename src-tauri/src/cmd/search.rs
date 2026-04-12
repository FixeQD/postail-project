use crate::db::saved_searches::{self, SavedSearch};
use crate::globals::get_db_pool;
use chrono::Utc;
use tauri::command;
use uuid::Uuid;

#[command]
pub async fn get_saved_searches(account_id: String) -> Result<Vec<SavedSearch>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    saved_searches::get_saved_searches(&conn, &account_id).map_err(|e| e.to_string())
}

#[command]
pub async fn create_saved_search(
    account_id: String,
    name: String,
    query_json: String,
    icon: String,
) -> Result<SavedSearch, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();

    saved_searches::create_saved_search(&conn, &id, &account_id, &name, &query_json, &icon, created_at)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_saved_search(id: String, account_id: String) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    saved_searches::delete_saved_search(&conn, &id, &account_id).map_err(|e| e.to_string())
}
