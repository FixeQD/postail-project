use std::fs;
use std::path::{Path, PathBuf};

use crate::error::DBError;
use crate::security::crypto::Crypto;

pub fn get_eml_cache_dir() -> PathBuf {
    crate::utils::config::get_data_dir().join("eml_cache")
}

fn sanitize_path_component(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn message_dir(account_id: &str, mailbox: &str, uid: u32) -> PathBuf {
    get_eml_cache_dir()
        .join(sanitize_path_component(account_id))
        .join(sanitize_path_component(mailbox))
        .join(uid.to_string())
}

pub fn eml_cache_path(account_id: &str, mailbox: &str, uid: u32) -> PathBuf {
    message_dir(account_id, mailbox, uid).join("eml.enc")
}

pub fn body_cache_path(account_id: &str, mailbox: &str, uid: u32) -> PathBuf {
    message_dir(account_id, mailbox, uid).join("body.json.enc")
}

pub fn inline_image_path(account_id: &str, mailbox: &str, uid: u32, part_id: &str) -> PathBuf {
    message_dir(account_id, mailbox, uid)
        .join(format!("img_{}.enc", sanitize_path_component(part_id)))
}

pub fn has_cached_eml(account_id: &str, mailbox: &str, uid: u32) -> bool {
    eml_cache_path(account_id, mailbox, uid).exists()
}

pub fn has_cached_body_file(account_id: &str, mailbox: &str, uid: u32) -> bool {
    body_cache_path(account_id, mailbox, uid).exists()
}

// ── EML ────────────────────────────────────────────────────────────────────

pub fn save_eml(
    crypto: &Crypto,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    raw_eml: &[u8],
) -> Result<PathBuf, DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    ensure_parent(&path)?;

    let encrypted = encrypt(security, raw_eml)?;
    fs::write(&path, encrypted)
        .map_err(|e| DBError::EmlCache(format!("Failed to write EML file: {}", e)))?;

    tracing::info!(
        target: "postail",
        "[EmlCache] Saved encrypted EML: uid={} account={} mailbox={}",
        uid, account_id, mailbox
    );

    Ok(path)
}

pub fn load_eml(
    crypto: &Crypto,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<Vec<u8>>, DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    if !path.exists() {
        return Ok(None);
    }
    let encrypted = fs::read(&path)
        .map_err(|e| DBError::EmlCache(format!("Failed to read EML file: {}", e)))?;
    Ok(Some(decrypt(security, &encrypted)?))
}

pub fn delete_eml(account_id: &str, mailbox: &str, uid: u32) -> Result<(), DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| DBError::EmlCache(format!("Failed to delete EML file: {}", e)))?;
    }
    Ok(())
}

// ── Parsed body ────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CachedBody {
    pub body_html: String,
    pub body_plain: String,
    #[serde(default)]
    pub read_receipt_to: Option<String>,
}

pub fn save_body(
    crypto: &Crypto,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    body: &CachedBody,
) -> Result<(), DBError> {
    let path = body_cache_path(account_id, mailbox, uid);
    ensure_parent(&path)?;

    let json = serde_json::to_vec(body)
        .map_err(|e| DBError::BodyCache(format!("Body serialization failed: {}", e)))?;

    let encrypted = encrypt(security, &json)?;
    fs::write(&path, encrypted)
        .map_err(|e| DBError::BodyCache(format!("Failed to write body file: {}", e)))?;

    tracing::info!(
        target: "postail",
        "[BodyCache] Saved body file uid={} mailbox={} html_len={} plain_len={}",
        uid, mailbox, body.body_html.len(), body.body_plain.len()
    );

    Ok(())
}

pub fn load_body(
    crypto: &Crypto,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<CachedBody>, DBError> {
    let path = body_cache_path(account_id, mailbox, uid);
    if !path.exists() {
        return Ok(None);
    }

    let encrypted = fs::read(&path)
        .map_err(|e| DBError::BodyCache(format!("Failed to read body file: {}", e)))?;
    let json = decrypt(security, &encrypted)?;

    let body: CachedBody = serde_json::from_slice(&json)
        .map_err(|e| DBError::BodyCache(format!("Body deserialization failed: {}", e)))?;

    Ok(Some(body))
}

pub fn delete_body(account_id: &str, mailbox: &str, uid: u32) -> Result<(), DBError> {
    let path = body_cache_path(account_id, mailbox, uid);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| DBError::BodyCache(format!("Failed to delete body file: {}", e)))?;
    }
    Ok(())
}

// ── Inline images ──────────────────────────────────────────────────────────

/// Encrypts and saves inline image bytes. Returns the path for storing in DB.
pub fn save_inline_image(
    crypto: &Crypto,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    part_id: &str,
    data: &[u8],
) -> Result<PathBuf, DBError> {
    let path = inline_image_path(account_id, mailbox, uid, part_id);
    ensure_parent(&path)?;

    let encrypted = encrypt(security, data)?;
    fs::write(&path, encrypted)
        .map_err(|e| DBError::Cache(format!("Failed to write inline image: {}", e)))?;

    Ok(path)
}

/// Decrypts and returns inline image bytes, or None if not cached.
pub fn load_inline_image(
    crypto: &Crypto,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    part_id: &str,
) -> Result<Option<Vec<u8>>, DBError> {
    let path = inline_image_path(account_id, mailbox, uid, part_id);
    if !path.exists() {
        return Ok(None);
    }
    let encrypted = fs::read(&path)
        .map_err(|e| DBError::Cache(format!("Failed to read inline image: {}", e)))?;
    Ok(Some(decrypt(security, &encrypted)?))
}

// ── Cleanup ────────────────────────────────────────────────────────────────

/// Deletes all cached files for a single message (eml, body, inline images).
pub fn delete_message_cache(account_id: &str, mailbox: &str, uid: u32) -> Result<(), DBError> {
    let dir = message_dir(account_id, mailbox, uid);
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| DBError::Cache(format!("Failed to delete message cache dir: {}", e)))?;
    }
    Ok(())
}

/// Deletes all cached files for a given account.
pub fn delete_account_eml_cache(account_id: &str) -> Result<(), DBError> {
    let dir = get_eml_cache_dir().join(sanitize_path_component(account_id));
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| {
            DBError::Cache(format!("Failed to delete account cache directory: {}", e))
        })?;
    }
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn ensure_parent(path: &Path) -> Result<(), DBError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| DBError::Cache(format!("Failed to create parent directory: {}", e)))?;
    }
    Ok(())
}

fn encrypt(crypto: &Crypto, data: &[u8]) -> Result<Vec<u8>, DBError> {
    crypto
        .encrypt(data)
        .map_err(|e| DBError::Cache(format!("Encryption failed: {}", e)))
}

fn decrypt(crypto: &Crypto, data: &[u8]) -> Result<Vec<u8>, DBError> {
    crypto
        .decrypt(data)
        .map_err(|e| DBError::Cache(format!("Decryption failed: {}", e)))
}
