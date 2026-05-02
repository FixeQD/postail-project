use crate::db::mail::eml_cache;
use crate::db::{MessageFull, ThreadMessage, ThreadView};
use crate::globals::{get_crypto, get_db_pool};
use rusqlite::OptionalExtension;
use tauri::command;

#[command]
pub async fn fetch_message_full(
    account_id: String,
    mailbox: String,
    uid: u32,
) -> Result<MessageFull, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut message =
        crate::db::mail::messages::fetch_message_full(&conn, &account_id, &mailbox, uid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Message not found in database".to_string())?;

    // Body lives in encrypted file cache — fetch from IMAP and cache if missing
    let crypto = get_crypto().await?;
    match eml_cache::load_body(&crypto, &account_id, &mailbox, uid) {
        Ok(Some(cached)) => {
            message.body_html_safe = cached.body_html;
            message.body_plain = cached.body_plain;
            message.read_receipt_to = cached.read_receipt_to;
        }
        _ => {
            // Cache miss — pull from IMAP, parse, and populate cache
            let imap = crate::globals::IMAP_MANAGER.lock().await;
            if let Ok(Some(full)) = imap
                .fetch_and_cache_message(&account_id, &mailbox, uid)
                .await
            {
                message.body_html_safe = full.body_html_safe;
                message.body_plain = full.body_plain;
                message.read_receipt_to = full.read_receipt_to;
            }
        }
    }

    Ok(message)
}

fn fetch_thread_uids(
    conn: &rusqlite::Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Vec<u32>, crate::error::DBError> {
    // Look up the message_id for this uid to find the thread
    let message_id: Option<String> = conn
        .query_row(
            "SELECT message_id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
            rusqlite::params![account_id, mailbox, uid],
            |row| row.get(0),
        )
        .optional()?;

    let Some(message_id) = message_id else {
        return Ok(vec![]);
    };

    let mut stmt = conn.prepare(
        "SELECT uid FROM messages WHERE account_id = ? AND mailbox = ? AND message_id = ? ORDER BY uid ASC"
    )?;
    let uids_iter = stmt.query_map(rusqlite::params![account_id, mailbox, message_id], |row| {
        row.get(0)
    })?;
    let uids: Result<Vec<u32>, _> = uids_iter.collect();
    Ok(uids?)
}

#[command]
pub async fn fetch_thread(
    account_id: String,
    mailbox: String,
    uid: u32,
) -> Result<ThreadView, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let thread_uids =
        fetch_thread_uids(&conn, &account_id, &mailbox, uid).map_err(|e| e.to_string())?;

    let crypto = get_crypto().await?;
    let mut messages: Vec<ThreadMessage> = Vec::new();
    for t_uid in thread_uids {
        if let Ok(Some(mut msg)) =
            crate::db::mail::messages::fetch_message_full(&conn, &account_id, &mailbox, t_uid)
        {
            if let Ok(Some(cached)) = eml_cache::load_body(&crypto, &account_id, &mailbox, t_uid) {
                msg.body_html_safe = cached.body_html;
                msg.body_plain = cached.body_plain;
            }
            messages.push(ThreadMessage {
                header: msg.header,
                body_html_safe: msg.body_html_safe,
                body_plain: msg.body_plain,
                is_current: t_uid == uid,
            });
        }
    }

    Ok(ThreadView { messages })
}

#[command]
pub async fn fetch_raw_eml_text(
    account_id: String,
    mailbox: String,
    uid: u32,
) -> Result<String, String> {
    let imap = crate::globals::IMAP_MANAGER.lock().await;
    let mut session = imap.connect_imap(&account_id).await?;
    session.select(&mailbox).await.map_err(|e| e.to_string())?;

    let result = {
        let mut fetches = session
            .uid_fetch(uid.to_string(), "RFC822")
            .await
            .map_err(|e| e.to_string())?;

        if let Some(fetch) = futures::StreamExt::next(&mut fetches).await {
            let fetch = fetch.map_err(|e| e.to_string())?;
            let bytes = fetch.body().ok_or("No body in fetch")?;
            Ok(String::from_utf8_lossy(bytes).into_owned())
        } else {
            Err("Message not found on server".to_string())
        }
    };

    let _ = session.logout().await;
    result
}
