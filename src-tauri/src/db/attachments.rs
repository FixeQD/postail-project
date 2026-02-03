use crate::error::DBError;
use dirs;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Ensure the application's attachments directory exists and return its path.
///
/// This uses the platform data directory (from `dirs::data_dir()`), falling back
/// to the current working directory if unavailable, and appends `postail/attachments`.
/// The directory is created recursively if it does not already exist; IO errors
/// are mapped to `DBError::Io`.
///
/// # Returns
///
/// The full path to the attachments directory.
///
/// # Examples
///
/// ```
/// let dir = crate::db::attachments::get_attachments_dir().unwrap();
/// assert!(dir.to_string_lossy().ends_with("postail/attachments"));
/// ```
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

/// Adds a file from the given filesystem path to the attachments store.
///
/// The function reads the file at `source_path`, infers its MIME type from the file
/// extension, saves the file into the attachments directory, and returns a
/// `DraftAttachment` containing metadata (id, filename, content type, size, hash,
/// path, and inline flag).
///
/// # Examples
///
/// ```
/// use std::fs::File;
/// use std::io::Write;
/// // create a temporary file to add
/// let mut tmp = tempfile::NamedTempFile::new().unwrap();
/// write!(tmp, "hello").unwrap();
/// let path = tmp.path().to_string_lossy().to_string();
/// let attachment = crate::db::attachments::add_attachment(&path).unwrap();
/// assert_eq!(attachment.filename, tmp.path().file_name().unwrap().to_string_lossy());
/// ```
///
/// # Returns
///
/// `Ok(DraftAttachment)` with metadata for the saved attachment, or `Err(DBError)` if the file
/// could not be read or saved.
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

/// Save a byte buffer as a non-inline attachment and return its attachment metadata.
///
/// `content_type` is the attachment's MIME type (for example, `"image/png"`).
///
/// # Returns
///
/// `DraftAttachment` containing the saved attachment's id, filename, content type, size, SHA-256 hash, filesystem path, optional CID, and inline flag.
///
/// # Examples
///
/// ```no_run
/// use crate::db::attachments::add_attachment_bytes;
///
/// let attachment = add_attachment_bytes(
///     vec![0u8, 1, 2, 3],
///     "example.bin".into(),
///     "application/octet-stream".into(),
/// ).unwrap();
///
/// assert!(!attachment.id.is_empty());
/// ```
pub fn add_attachment_bytes(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<crate::db::DraftAttachment, DBError> {
    save_attachment_data(&bytes, &filename, &content_type, false)
}

/// Creates and stores an inline attachment from raw bytes.
///
/// The saved attachment will be marked as inline and receive a generated content ID.
///
/// # Examples
///
/// ```
/// let bytes = b"hello".to_vec();
/// let filename = "greeting.txt".to_string();
/// let content_type = "text/plain".to_string();
/// let attachment = add_inline_attachment_bytes(bytes, filename, content_type).unwrap();
/// assert!(attachment.inline);
/// assert!(attachment.cid.is_some());
/// ```
pub fn add_inline_attachment_bytes(
    bytes: Vec<u8>,
    filename: String,
    content_type: String,
) -> Result<crate::db::DraftAttachment, DBError> {
    save_attachment_data(&bytes, &filename, &content_type, true)
}

/// Creates a new message Content-ID (CID) suitable for inline attachments.
///
/// # Returns
///
/// A CID string in the format `<uuid>@postail.local`.
///
/// # Examples
///
/// ```
/// let cid = generate_cid();
/// assert!(cid.ends_with("@postail.local"));
/// ```
fn generate_cid() -> String {
    format!("{}@postail.local", Uuid::new_v4())
}

/// Saves raw attachment bytes to the attachments directory and returns metadata for the saved file.
///
/// The function generates a new UUID for the attachment filename, writes the provided bytes to disk,
/// computes the SHA-256 hash and byte size, and, if `inline` is true, generates a CID for the attachment.
/// The returned `DraftAttachment` contains the attachment `id`, original `filename`, `content_type`, `size`,
/// `hash`, filesystem `path`, optional `cid`, and the `inline` flag.
///
/// # Examples
///
/// ```
/// let att = save_attachment_data(b"hello world", "hello.txt", "text/plain", false).unwrap();
/// assert_eq!(att.filename, "hello.txt");
/// assert_eq!(att.content_type, "text/plain");
/// assert_eq!(att.inline, false);
/// assert!(att.size > 0);
/// ```
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

/// Map a file extension to a MIME type string.
///
/// Returns the MIME type for common image and document extensions; returns `"application/octet-stream"` for unknown extensions.
///
/// # Examples
///
/// ```
/// assert_eq!(infer_mime("jpg"), "image/jpeg");
/// assert_eq!(infer_mime("TXT"), "text/plain");
/// assert_eq!(infer_mime("unknown"), "application/octet-stream");
/// ```
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

/// Removes the stored attachment file identified by `id` from the attachments directory.
///
/// If the file for `id` exists, it is removed; if it does not exist, the function succeeds silently.
///
/// # Errors
///
/// Returns a `DBError::Io` if creating or accessing the attachments directory or removing the file fails.
///
/// # Examples
///
/// ```
/// // Remove an attachment by its id (filename used when saved)
/// remove_attachment("550e8400-e29b-41d4-a716-446655440000").unwrap();
/// ```
pub fn remove_attachment(id: &str) -> Result<(), DBError> {
    let target_dir = get_attachments_dir()?;
    let target_path = target_dir.join(id);

    if target_path.exists() {
        fs::remove_file(target_path).map_err(DBError::Io)?;
    }

    Ok(())
}