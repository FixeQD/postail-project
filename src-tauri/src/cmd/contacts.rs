use crate::db::{Contact, MailHeader};
use crate::globals::get_db_pool;
use tauri::command;

#[command]
pub async fn list_contacts() -> Result<Vec<Contact>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::list_contacts(&conn).map_err(|e| e.to_string())
}

#[command]
pub async fn search_contacts_full(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<Contact>, String> {
    let limit = limit.unwrap_or(50);
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::search_contacts(&conn, &query, limit).map_err(|e| e.to_string())
}

#[command]
pub async fn get_contact_messages(
    account_id: String,
    email: String,
    limit: Option<u32>,
) -> Result<Vec<MailHeader>, String> {
    let limit = limit.unwrap_or(50);
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::get_contact_messages(&conn, &account_id, &email, limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_contact(
    id: i64,
    name: Option<String>,
    email: String,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
    avatar_url: Option<String>,
    birthday: Option<i64>,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::update_contact(
        &conn,
        id,
        name.as_deref(),
        &email,
        phone.as_deref(),
        company.as_deref(),
        notes.as_deref(),
        avatar_url.as_deref(),
        birthday,
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn create_contact(
    name: Option<String>,
    email: String,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
    avatar_url: Option<String>,
    birthday: Option<i64>,
) -> Result<i64, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::create_contact(
        &conn,
        name.as_deref(),
        &email,
        phone.as_deref(),
        company.as_deref(),
        notes.as_deref(),
        avatar_url.as_deref(),
        birthday,
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_contact(id: i64) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::delete_contact(&conn, id).map_err(|e| e.to_string())
}
