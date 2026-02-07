use crate::db::SyncStatusEnum;
use crate::globals::IMAP_MANAGER;
use crate::imap::sync_status::{
    mark_sync_complete, mark_sync_error, update_sync_status, SYNC_STATUS_MANAGER,
};
use tauri::command;

#[command]
pub async fn start_sync(account_id: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] start_sync called for {}", account_id);
    let res = {
        let imap = IMAP_MANAGER.lock().await;
        imap.start_sync(&account_id).await
    };
    match res {
        Ok(()) => {
            tracing::info!(target: "postail", "[UI] start_sync succeeded");
            Ok(())
        }
        Err(e) => {
            tracing::error!(target: "postail", "[UI] start_sync failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[command]
pub async fn stop_sync(account_id: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] stop_sync called for {}", account_id);
    SYNC_STATUS_MANAGER.request_stop(&account_id).await;
    tracing::info!(target: "postail", "[UI] stop_sync requested for {}", account_id);
    Ok(())
}

#[command]
pub async fn get_sync_status(account_id: String) -> Result<SyncStatusEnum, String> {
    Ok(SYNC_STATUS_MANAGER.get_status(&account_id).await)
}

/// Fetches the mailbox/folder list from IMAP and saves to DB
#[command]
pub async fn sync_mailbox_list(account_id: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] sync_mailbox_list called for {}", account_id);
    let imap = IMAP_MANAGER.lock().await;
    imap.fetch_mailboxes(&account_id).await?;
    tracing::info!(target: "postail", "[UI] sync_mailbox_list done for {}", account_id);
    Ok(())
}

/// Syncs messages for a single mailbox
#[command]
pub async fn sync_single_mailbox(account_id: String, mailbox: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] sync_single_mailbox called for {}@{}", mailbox, account_id);

    let account_email = {
        let imap = IMAP_MANAGER.lock().await;
        let conn_guard = imap.get_conn();
        let conn_guard = conn_guard.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        crate::db::accounts::get_account_email(conn, &account_id)
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| account_id.clone())
    };

    // Register for status tracking so frontend gets events
    SYNC_STATUS_MANAGER
        .register_account(&account_id, &account_email)
        .await;

    SYNC_STATUS_MANAGER
        .set_status(&account_id, SyncStatusEnum::Syncing)
        .await;

    update_sync_status(&account_id, &mailbox, 0, 0).await;

    let result = {
        let imap = IMAP_MANAGER.lock().await;
        imap.sync_single_mailbox_messages(&account_id, &mailbox)
            .await
    };

    match result {
        Ok(()) => {
            tracing::info!(target: "postail", "[UI] sync_single_mailbox done for {}@{}", mailbox, account_id);
            mark_sync_complete(&account_id).await;
            Ok(())
        }
        Err(e) => {
            tracing::error!(target: "postail", "[UI] sync_single_mailbox failed for {}@{}: {}", mailbox, account_id, e);
            mark_sync_error(&account_id, &e.to_string()).await;
            Err(e.to_string())
        }
    }
}

/// Start IDLE/poll watch for the currently open mailbox.
/// Stops any previous watch for this account
#[command]
pub async fn watch_mailbox(account_id: String, mailbox: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] watch_mailbox called for {}@{}", mailbox, account_id);
    let imap = IMAP_MANAGER.lock().await.clone();
    imap.start_watch_mailbox(&account_id, &mailbox)
        .await
        .map_err(|e| e.to_string())
}

/// Stop the active IDLE/poll watch for an account
#[command]
pub async fn unwatch_mailbox(account_id: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] unwatch_mailbox called for {}", account_id);
    let imap = IMAP_MANAGER.lock().await.clone();
    imap.stop_watch_mailbox(&account_id).await;
    Ok(())
}
