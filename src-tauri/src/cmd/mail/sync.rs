use crate::db::SyncStatusEnum;
use crate::globals::IMAP_MANAGER;
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

    use crate::imap::sync_status::SYNC_STATUS_MANAGER;
    SYNC_STATUS_MANAGER.request_stop(&account_id).await;

    tracing::info!(target: "postail", "[UI] stop_sync requested for {}", account_id);
    Ok(())
}

#[command]
pub async fn get_sync_status(account_id: String) -> Result<SyncStatusEnum, String> {
    use crate::imap::sync_status::SYNC_STATUS_MANAGER;
    Ok(SYNC_STATUS_MANAGER.get_status(&account_id).await)
}
