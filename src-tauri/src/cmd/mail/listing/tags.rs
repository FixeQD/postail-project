use std::collections::HashMap;
use tauri::command;
use crate::globals::get_db_pool;
use crate::db::MailHeader;

#[command]
pub async fn add_message_tag(
    account_id: String,
    mailbox: String,
    uid: u32,
    tag: String,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    crate::db::mail::messages::add_tag(&conn, &account_id, &mailbox, uid, &tag)
        .map_err(|e| e.to_string())?;

    // also sync to IMAP if possible
    let imap = crate::globals::IMAP_MANAGER.lock().await;
    tauri::async_runtime::spawn(async move {
        if let Ok(mut session) = imap.connect_imap(&account_id).await {
            if session.select(&mailbox).await.is_ok() {
                let _ = session
                    .uid_store(uid.to_string(), format!("+FLAGS ({})", tag))
                    .await;
            }
            let _ = session.logout().await;
        }
    });

    Ok(())
}

#[command]
pub async fn remove_message_tag(
    account_id: String,
    mailbox: String,
    uid: u32,
    tag: String,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    crate::db::mail::messages::remove_tag(&conn, &account_id, &mailbox, uid, &tag)
        .map_err(|e| e.to_string())?;

    let imap = crate::globals::IMAP_MANAGER.lock().await;
    tauri::async_runtime::spawn(async move {
        if let Ok(mut session) = imap.connect_imap(&account_id).await {
            if session.select(&mailbox).await.is_ok() {
                let _ = session
                    .uid_store(uid.to_string(), format!("-FLAGS ({})", tag))
                    .await;
            }
            let _ = session.logout().await;
        }
    });

    Ok(())
}

#[command]
pub async fn fetch_tag_headers(
    account_id: String,
    tag: String,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    crate::db::mail::messages::fetch_tag_headers(&conn, &account_id, &tag, limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_account_tags(account_id: String) -> Result<Vec<String>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT mt.tag
             FROM message_tags mt
             JOIN messages m ON m.id = mt.message_id
             WHERE m.account_id = ?
             ORDER BY mt.tag ASC",
        )
        .map_err(|e| e.to_string())?;

    let tags: Vec<String> = stmt
        .query_map([&account_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    tracing::info!(target: "postail", "[DB] get_account_tags for account_id={} returned {} tags: {:?}", account_id, tags.len(), tags);

    Ok(tags)
}

/// Get hue (0-359) for all tags on an account. Returns {tag: hue} map.
#[command]
pub async fn get_tag_colors(
    account_id: String,
) -> Result<HashMap<String, i64>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT mt.tag, COALESCE(tc.hue, 200) as hue
             FROM message_tags mt
             JOIN messages m ON m.id = mt.message_id
             LEFT JOIN tag_colors tc ON tc.tag = mt.tag
             WHERE m.account_id = ?
             ORDER BY mt.tag ASC",
        )
        .map_err(|e| e.to_string())?;

    let map: HashMap<String, i64> = stmt
        .query_map([&account_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

/// Set hue (0-359) for a tag.
#[command]
pub async fn set_tag_color(tag: String, hue: i64) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO tag_colors (tag, hue) VALUES (?, ?) ON CONFLICT(tag) DO UPDATE SET hue = excluded.hue",
        rusqlite::params![tag, hue],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Rename a tag across all messages.
#[command]
pub async fn rename_tag(
    old_tag: String,
    new_tag: String,
    account_id: String,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let new_tag = new_tag.trim().to_string();
    if new_tag.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }

    conn.execute(
        "UPDATE message_tags SET tag = ?
         WHERE tag = ? AND message_id IN (SELECT id FROM messages WHERE account_id = ?)",
        rusqlite::params![new_tag, old_tag, account_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO tag_colors (tag, hue)
         SELECT ?, hue FROM tag_colors WHERE tag = ?
         ON CONFLICT(tag) DO NOTHING",
        rusqlite::params![new_tag, old_tag],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM tag_colors WHERE tag = ?",
        rusqlite::params![old_tag],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete a tag from all messages on an account.
#[command]
pub async fn delete_tag(tag: String, account_id: String) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM message_tags WHERE tag = ?
         AND message_id IN (SELECT id FROM messages WHERE account_id = ?)",
        rusqlite::params![tag, account_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM tag_colors WHERE tag = ?",
        rusqlite::params![tag],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
