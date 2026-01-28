use crate::error::DBError;
use dirs;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use std::io::{Read, BufReader};

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

    let id = Uuid::new_v4().to_string();
    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("");
    let content_type = match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    };

    let size = fs::metadata(source).map_err(DBError::Io)?.len();

    // Compute SHA-256 hash
    let file = fs::File::open(source).map_err(DBError::Io)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer).map_err(DBError::Io)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    let hash = format!("{:x}", hasher.finalize());

    let target_dir = get_attachments_dir()?;
    let target_path = target_dir.join(&id);

    fs::copy(source, &target_path).map_err(DBError::Io)?;

    Ok(crate::db::DraftAttachment {
        id,
        filename,
        content_type: content_type.to_string(),
        size,
        hash,
    })
}

pub fn remove_attachment(id: &str) -> Result<(), DBError> {
    let target_dir = get_attachments_dir()?;
    let target_path = target_dir.join(id);

    if target_path.exists() {
        fs::remove_file(target_path).map_err(DBError::Io)?;
    }

    Ok(())
}
