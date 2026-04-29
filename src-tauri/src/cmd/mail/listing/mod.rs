pub mod fetching;
pub mod tags;
pub mod attachments;

pub use fetching::*;
pub use tags::*;
pub use attachments::*;

use crate::globals::{get_db_pool, IMAP_MANAGER};
use crate::db::MailHeader;
use crate::db::Mailbox;
use tauri::command;

#[command]
pub async fn fetch_mailboxes(account_id: String) -> Result<Vec<Mailbox>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::mail::mailbox::fetch_mailboxes(&conn, &account_id).map_err(|e| e.to_string())
}

#[command]
pub async fn fetch_headers(
    account_id: String,
    mailbox: String,
    anchor: Option<u32>,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    let imap = IMAP_MANAGER.lock().await;
    imap.fetch_headers_hybrid(&account_id, &mailbox, anchor, limit)
        .await
}

#[command]
pub async fn fetch_starred_headers(
    account_id: String,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::mail::messages::fetch_starred_headers(&conn, &account_id, limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_mailbox_role(
    account_id: String,
    name: String,
    role: String,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE mailboxes SET role = ?, role_customized = 1 WHERE account_id = ? AND name = ?",
        rusqlite::params![role, account_id, name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
