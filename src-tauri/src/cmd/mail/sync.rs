use crate::db::SyncStatusEnum;
use crate::globals::IMAP_MANAGER;
use tauri::command;

#[command]
pub fn start_sync(account_id: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] start_sync called for {}", account_id);
    let imap = IMAP_MANAGER.lock().unwrap();
    match imap.start_sync(&account_id) {
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
pub fn stop_sync(account_id: String) -> Result<(), String> {
    tracing::info!(target: "postail", "[UI] stop_sync called for {}", account_id);
    
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    
    rt.block_on(async {
        use crate::imap::sync_status::SYNC_STATUS_MANAGER;
        SYNC_STATUS_MANAGER.request_stop(&account_id).await;
    });
    
    tracing::info!(target: "postail", "[UI] stop_sync requested for {}", account_id);
    Ok(())
}

#[command]
pub fn get_sync_status(account_id: String) -> Result<SyncStatusEnum, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(rt.block_on(async {
        use crate::imap::sync_status::SYNC_STATUS_MANAGER;
        SYNC_STATUS_MANAGER.get_status(&account_id).await
    }))
}
