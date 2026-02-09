use crate::db;
use crate::globals::{DB_CONN, IMAP_MANAGER};
use tauri::command;

#[command]
pub fn search_messages(
    account_id: Option<String>,
    mailbox: Option<String>,
    query: String,
    limit: u32,
) -> Result<Vec<db::search::SearchResult>, String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    db::search_messages(
        conn,
        account_id.as_deref(),
        mailbox.as_deref(),
        &query,
        limit,
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn mark_read(
    account_id: String,
    mailbox: String,
    uids: Vec<u64>,
    read: bool,
) -> Result<(), String> {
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;

    {
        let conn_guard = DB_CONN.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        db::mark_read(conn, &account_id, &mailbox, &uids, read).map_err(|e| e.to_string())?;

        let operation = if read { "add" } else { "remove" };
        for uid in &uids {
            db::enqueue_flag_change(
                conn,
                &account_id,
                &mailbox,
                *uid,
                operation,
                &[String::from("\\Seen")],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let account_id_clone = account_id.clone();
    tokio::spawn(async move {
        if let Err(e) = process_flag_queue(&account_id_clone).await {
            tracing::error!(target: "postail", "Failed to process flag queue: {}", e);
        }
    });

    Ok(())
}

async fn process_flag_queue(account_id: &str) -> Result<(), String> {
    let ops = {
        let conn_guard = DB_CONN.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        db::get_pending_flag_operations(conn, account_id, 5).map_err(|e| e.to_string())?
    };

    let imap_manager = IMAP_MANAGER.lock().await;

    for op in ops {
        let result = imap_manager
            .set_flags_remote(
                &op.account_id,
                &op.mailbox,
                &[op.uid],
                &op.operation,
                &op.flags,
            )
            .await;

        let conn_guard = DB_CONN.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

        match result {
            Ok(_) => {
                db::mark_flag_operation_success(conn, op.id).map_err(|e| e.to_string())?;
                tracing::debug!(target: "postail",
                    "Flag sync success: {}@{} uid={}", op.mailbox, op.account_id, op.uid
                );
            }
            Err(e) => {
                let error_msg = e.to_string();
                db::mark_flag_operation_failed(conn, op.id, &error_msg)
                    .map_err(|e| e.to_string())?;
                tracing::warn!(target: "postail",
                    "Flag sync failed: {}@{} uid={} - {}", op.mailbox, op.account_id, op.uid, error_msg
                );
            }
        }
    }

    Ok(())
}

#[command]
pub fn move_to_trash(account_id: String, mailbox: String, uids: Vec<u64>) -> Result<(), String> {
    let conn_guard = DB_CONN.lock().unwrap();
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;
    db::move_to_trash(conn, &account_id, &mailbox, &uids).map_err(|e| e.to_string())?;
    Ok(())
}
