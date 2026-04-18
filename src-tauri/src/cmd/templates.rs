use crate::db::templates::{self, Template};
use crate::error::DBError;
use crate::globals::get_db_pool;
use chrono::Utc;
use tauri::command;
use uuid::Uuid;

#[command]
pub async fn list_templates(account_id: String) -> Result<Vec<Template>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    templates::list_templates(&conn, &account_id).map_err(|e: DBError| e.to_string())
}

#[command]
pub async fn save_template(mut tmpl: Template) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    if tmpl.id.is_empty() {
        tmpl.id = Uuid::new_v4().to_string();
    }

    let now = Utc::now().timestamp();
    tmpl.created_at = now;
    tmpl.updated_at = now;

    templates::save_template(&conn, &tmpl).map_err(|e: DBError| e.to_string())
}

#[command]
pub async fn delete_template(id: String, account_id: String) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    templates::delete_template(&conn, &id, &account_id).map_err(|e: DBError| e.to_string())
}
