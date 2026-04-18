use crate::db::signatures::{self, Signature};
use crate::error::DBError;
use crate::globals::get_db_pool;
use chrono::Utc;
use rusqlite::OptionalExtension;
use tauri::command;
use uuid::Uuid;

#[command]
pub async fn list_signatures(account_id: String) -> Result<Vec<Signature>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    signatures::list_signatures(&conn, &account_id).map_err(|e: DBError| e.to_string())
}

#[command]
pub async fn save_signature(mut sig: Signature) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    if sig.id.is_empty() {
        sig.id = Uuid::new_v4().to_string();
    }

    let now = Utc::now().timestamp();
    sig.created_at = now;
    sig.updated_at = now;

    signatures::save_signature(&conn, &sig).map_err(|e: DBError| e.to_string())
}

#[command]
pub async fn delete_signature(id: String, account_id: String) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    signatures::delete_signature(&conn, &id, &account_id).map_err(|e: DBError| e.to_string())
}

#[command]
pub async fn get_default_signature(account_id: String) -> Result<Option<Signature>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, account_id, name, html_content, is_default, created_at, updated_at
         FROM signatures WHERE account_id = ? AND is_default = 1 LIMIT 1",
        [account_id],
        |row| {
            Ok(Signature {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                html_content: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(|e| e.to_string())
}
