use crate::db;
use crate::globals::{IMAP_MANAGER, get_db_pool};
use tauri::command;

#[command]
pub async fn search_messages(
    account_id: Option<String>,
    mailbox: Option<String>,
    query: String,
    limit: u32,
) -> Result<Vec<db::search::SearchResult>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    db::search_messages(
        &conn,
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
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        db::mark_read(&conn, &account_id, &mailbox, &uids, read).map_err(|e| e.to_string())?;

        let operation = if read { "add" } else { "remove" };
        for uid in &uids {
            db::enqueue_flag_change(
                &conn,
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

pub async fn process_flag_queue(account_id: &str) -> Result<(), String> {
    // Prevent concurrent queue processing per account
    static QUEUE_LOCKS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<tokio::sync::Mutex<()>>>>,
    > = std::sync::OnceLock::new();
    let lock_map =
        QUEUE_LOCKS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let account_lock = {
        let mut map = lock_map.lock().unwrap();
        map.entry(account_id.to_string())
            .or_insert_with(|| std::sync::Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    };
    let _guard = account_lock.lock().await;

    let ops = {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        db::get_pending_flag_operations(&conn, account_id, 5).map_err(|e| e.to_string())?
    };

    let imap_manager = IMAP_MANAGER.lock().await;

    for op in ops {
        let result = match op.operation_type.as_str() {
            "move" => {
                if let Some(target) = &op.target_mailbox {
                    imap_manager
                        .move_messages_remote(&op.account_id, &op.mailbox, target, &[op.uid])
                        .await
                } else {
                    Err(crate::error::AppError::from(
                        "Move operation missing target mailbox",
                    ))
                }
            }
            _ => {
                imap_manager
                    .set_flags_remote(
                        &op.account_id,
                        &op.mailbox,
                        &[op.uid],
                        &op.operation,
                        &op.flags,
                    )
                    .await
            }
        };

        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        match result {
            Ok(_) => {
                db::mark_flag_operation_success(&conn, op.id).map_err(|e| e.to_string())?;
                tracing::debug!(target: "postail",
                    "{} sync success: {}@{} uid={}",
                    op.operation_type, op.mailbox, op.account_id, op.uid
                );
            }
            Err(e) => {
                let error_msg = e.to_string();
                db::mark_flag_operation_failed(&conn, op.id, &error_msg)
                    .map_err(|e| e.to_string())?;
                tracing::warn!(target: "postail",
                    "{} sync failed: {}@{} uid={} - {}",
                    op.operation_type, op.mailbox, op.account_id, op.uid, error_msg
                );
            }
        }
    }

    Ok(())
}

#[command]
pub async fn delete_messages(
    account_id: String,
    mailbox: String,
    uids: Vec<u64>,
) -> Result<(), String> {
    let uids: Result<Vec<u32>, String> = uids
        .into_iter()
        .map(|u| u.try_into().map_err(|_| format!("UID too large: {}", u)))
        .collect();
    let uids = uids?;

    {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        db::move_to_trash(&conn, &account_id, &mailbox, &uids).map_err(|e| e.to_string())?;
    }

    let account_id_clone = account_id.clone();
    tokio::spawn(async move {
        if let Err(e) = process_flag_queue(&account_id_clone).await {
            tracing::error!(target: "postail", "Failed to process move queue: {}", e);
        }
    });

    Ok(())
}

/// Toggle starred flag for a message. Returns the new starred state
#[command]
pub async fn toggle_starred(account_id: String, mailbox: String, uid: u64) -> Result<bool, String> {
    let uid_u32: u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    let new_starred = {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        let starred =
            db::toggle_starred(&conn, &account_id, &mailbox, uid_u32).map_err(|e| e.to_string())?;

        let imap_operation = if starred { "add" } else { "remove" };
        db::enqueue_flag_change(
            &conn,
            &account_id,
            &mailbox,
            uid_u32,
            imap_operation,
            &[String::from("\\Flagged")],
        )
        .map_err(|e| e.to_string())?;

        starred
    };

    let account_id_clone = account_id.clone();
    tokio::spawn(async move {
        if let Err(e) = process_flag_queue(&account_id_clone).await {
            tracing::error!(target: "postail", "[Star] Failed to sync \\Flagged to server: {}", e);
        }
    });

    Ok(new_starred)
}
