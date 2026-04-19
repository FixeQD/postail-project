use crate::db::DraftAttachment;
use tauri::command;

#[command]
pub async fn add_attachment(path: String) -> Result<DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_attachment(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn add_attachment_bytes(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_attachment_bytes(bytes, filename, content_type)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn add_inline_attachment_path(path: String) -> Result<DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_inline_attachment_from_path(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn remove_attachment(id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::remove_attachment(&id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn add_inline_attachment(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<DraftAttachment, String> {
    tokio::task::spawn_blocking(move || {
        crate::db::attachments::add_inline_attachment_bytes(bytes, filename, content_type)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}