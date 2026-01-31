use crate::error::DBError;
use dirs;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn get_attachments_dir() -> Result<PathBuf, DBError> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("attachments");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(DBError::Io)?;
    }

    Ok(data_dir)
}

pub fn add_attachment(source_path: &str) -> Result<crate::db::DraftAttachment, DBError> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(DBError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source file not found: {}", source_path),
        )));
    }

    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let bytes = fs::read(source).map_err(DBError::Io)?;
    let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("");
    let content_type = infer_mime(extension);

    save_attachment_data(&bytes, &filename, &content_type, false)
}

pub fn add_attachment_bytes(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<crate::db::DraftAttachment, DBError> {
    save_attachment_data(&bytes, &filename, &content_type, false)
}

pub fn add_inline_attachment_bytes(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<crate::db::DraftAttachment, DBError> {
    save_attachment_data(&bytes, &filename, &content_type, true)
}

fn generate_cid() -> String {
    format!("{}@postail.local", Uuid::new_v4())
}

fn save_attachment_data(
    bytes: &[u8],
    filename: &str,
    content_type: &str,
    inline: bool,
) -> Result<crate::db::DraftAttachment, DBError> {
    let id = Uuid::new_v4().to_string();
    let size = bytes.len() as u64;

    // Compute SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = format!("{:x}", hasher.finalize());

    let target_dir = get_attachments_dir()?;
    let target_path = target_dir.join(&id);

    fs::write(&target_path, bytes).map_err(DBError::Io)?;

    let cid = if inline { Some(generate_cid()) } else { None };

    Ok(crate::db::DraftAttachment {
        id,
        filename: filename.to_string(),
        content_type: content_type.to_string(),
        size,
        hash,
        path: target_path.to_string_lossy().to_string(),
        cid,
        inline,
    })
}

fn infer_mime(extension: &str) -> String {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
    .to_string()
}

pub fn remove_attachment(id: &str) -> Result<(), DBError> {
    let target_dir = get_attachments_dir()?;
    let target_path = target_dir.join(id);

    if target_path.exists() {
        fs::remove_file(target_path).map_err(DBError::Io)?;
    }

    Ok(())
}
