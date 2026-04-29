use tauri::command;
use crate::globals::get_db_pool;
use crate::db::MessageFull;
use rusqlite::OptionalExtension;

#[command]
pub async fn fetch_message_full(
    account_id: String,
    mailbox: String,
    uid: u32,
) -> Result<MessageFull, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut message = crate::db::mail::messages::fetch_message_full(&conn, &account_id, &mailbox, uid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Message not found in database".to_string())?;

    // Load body content from disk
    if let Ok(Some(message_table_id)) =
        crate::db::mail::messages::get_message_table_id(&conn, &account_id, &mailbox, uid)
    {
        if let Ok((html, plain)) =
            crate::db::mail::message_bodies::load_message_body(&conn, message_table_id)
        {
            message.body_html_safe = html.unwrap_or_default();
            message.body_plain = plain;
        }
    }

    Ok(message)
}

fn fetch_thread_uids(conn: &rusqlite::Connection, account_id: &str, mailbox: &str, uid: u32)
    -> Result<Vec<u32>, crate::error::DBError>
{
    // Look up the message_id for this uid to find the thread
    let message_id: Option<String> = conn.query_row(
        "SELECT message_id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
        rusqlite::params![account_id, mailbox, uid],
        |row| row.get(0),
    ).optional()?;

    let Some(message_id) = message_id else {
        return Ok(vec![]);
    };

    let mut stmt = conn.prepare(
        "SELECT uid FROM messages WHERE account_id = ? AND mailbox = ? AND message_id = ? ORDER BY uid ASC"
    )?;
    let uids_iter = stmt.query_map(
        rusqlite::params![account_id, mailbox, message_id],
        |row| row.get(0),
    )?;
    let uids: Result<Vec<u32>, _> = uids_iter.collect();
    Ok(uids?)
}

#[command]
pub async fn fetch_thread(
    account_id: String,
    mailbox: String,
    uid: u32,
) -> Result<Vec<MessageFull>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let thread_uids = fetch_thread_uids(&conn, &account_id, &mailbox, uid)
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for t_uid in thread_uids {
        if let Ok(Some(mut msg)) =
            crate::db::mail::messages::fetch_message_full(&conn, &account_id, &mailbox, t_uid)
        {
            if let Ok(Some(message_table_id)) =
                crate::db::mail::messages::get_message_table_id(&conn, &account_id, &mailbox, t_uid)
            {
                if let Ok((html, plain)) =
                    crate::db::mail::message_bodies::load_message_body(&conn, message_table_id)
                {
                    msg.body_html_safe = html.unwrap_or_default();
                    msg.body_plain = plain;
                }
            }
            messages.push(msg);
        }
    }

    Ok(messages)
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
