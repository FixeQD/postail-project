use std::fs;
use std::path::{Path, PathBuf};

use crate::error::DBError;
use crate::security::SecurityManager;

/// Returns the base cache directory for encrypted EML/body files.
/// Structure: <data_dir>/eml_cache/<account_id>/<mailbox_safe>/
pub fn get_eml_cache_dir() -> PathBuf {
    crate::utils::config::get_data_dir().join("eml_cache")
}

/// Sanitizes a mailbox name for use as a directory component.
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

fn account_mailbox_dir(account_id: &str, mailbox: &str) -> PathBuf {
    get_eml_cache_dir()
        .join(sanitize_path_component(account_id))
        .join(sanitize_path_component(mailbox))
}

/// Returns the full path to the encrypted EML file for a given message.
pub fn eml_cache_path(account_id: &str, mailbox: &str, uid: u32) -> PathBuf {
    account_mailbox_dir(account_id, mailbox).join(format!("{}.eml.enc", uid))
}

/// Returns the full path to the encrypted parsed-body JSON file.
pub fn body_cache_path(account_id: &str, mailbox: &str, uid: u32) -> PathBuf {
    account_mailbox_dir(account_id, mailbox).join(format!("{}.body.json.enc", uid))
}

/// Returns true if an encrypted EML file exists for this message.
pub fn has_cached_eml(account_id: &str, mailbox: &str, uid: u32) -> bool {
    eml_cache_path(account_id, mailbox, uid).exists()
}

/// Returns true if a parsed body file exists for this message.
pub fn has_cached_body_file(account_id: &str, mailbox: &str, uid: u32) -> bool {
    body_cache_path(account_id, mailbox, uid).exists()
}

// ── EML ────────────────────────────────────────────────────────────────────

/// Encrypts and saves raw EML bytes to disk.
pub fn save_eml(
    security: &SecurityManager,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    raw_eml: &[u8],
) -> Result<PathBuf, DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    ensure_parent(&path)?;

    let encrypted = encrypt(security, raw_eml)?;
    fs::write(&path, encrypted).map_err(DBError::Io)?;

    tracing::info!(
        target: "postail",
        "[EmlCache] Saved encrypted EML: uid={} account={} mailbox={}",
        uid, account_id, mailbox
    );

    Ok(path)
}

/// Reads and decrypts an EML file from disk. Returns None if file doesn't exist.
pub fn load_eml(
    security: &SecurityManager,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<Vec<u8>>, DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    if !path.exists() {
        return Ok(None);
    }
    let encrypted = fs::read(&path).map_err(DBError::Io)?;
    Ok(Some(decrypt(security, &encrypted)?))
}

/// Deletes the cached EML file for a message.
pub fn delete_eml(account_id: &str, mailbox: &str, uid: u32) -> Result<(), DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    if path.exists() {
        fs::remove_file(&path).map_err(DBError::Io)?;
    }
    Ok(())
}

// ── Parsed body ────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CachedBody {
    pub body_html: String,
    pub body_plain: String,
}

/// Encrypts and saves parsed body (html + plain) as a JSON file on disk.
pub fn save_body(
    security: &SecurityManager,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    body: &CachedBody,
) -> Result<(), DBError> {
    let path = body_cache_path(account_id, mailbox, uid);
    ensure_parent(&path)?;

    let json = serde_json::to_vec(body).map_err(|e| {
        DBError::Io(std::io::Error::other(format!(
            "body serialization failed: {}",
            e
        )))
    })?;

    let encrypted = encrypt(security, &json)?;
    fs::write(&path, encrypted).map_err(DBError::Io)?;

    tracing::info!(
        target: "postail",
        "[BodyCache] Saved body file uid={} mailbox={} html_len={} plain_len={}",
        uid, mailbox, body.body_html.len(), body.body_plain.len()
    );

    Ok(())
}

/// Reads and decrypts the parsed body file. Returns None if it doesn't exist.
pub fn load_body(
    security: &SecurityManager,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<Option<CachedBody>, DBError> {
    let path = body_cache_path(account_id, mailbox, uid);
    if !path.exists() {
        return Ok(None);
    }

    let encrypted = fs::read(&path).map_err(DBError::Io)?;
    let json = decrypt(security, &encrypted)?;

    let body: CachedBody = serde_json::from_slice(&json).map_err(|e| {
        DBError::Io(std::io::Error::other(format!(
            "body deserialization failed: {}",
            e
        )))
    })?;

    Ok(Some(body))
}

/// Deletes the cached body file for a message.
pub fn delete_body(account_id: &str, mailbox: &str, uid: u32) -> Result<(), DBError> {
    let path = body_cache_path(account_id, mailbox, uid);
    if path.exists() {
        fs::remove_file(&path).map_err(DBError::Io)?;
    }
    Ok(())
}

// ── Account-level cleanup ──────────────────────────────────────────────────

/// Deletes all cached EML and body files for a given account.
pub fn delete_account_eml_cache(account_id: &str) -> Result<(), DBError> {
    let dir = get_eml_cache_dir().join(sanitize_path_component(account_id));
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(DBError::Io)?;
    }
    Ok(())
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn ensure_parent(path: &Path) -> Result<(), DBError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(DBError::Io)?;
    }
    Ok(())
}

fn encrypt(security: &SecurityManager, data: &[u8]) -> Result<Vec<u8>, DBError> {
    security
        .encrypt(data)
        .map_err(|e| DBError::Io(std::io::Error::other(format!("encryption failed: {}", e))))
}

fn decrypt(security: &SecurityManager, data: &[u8]) -> Result<Vec<u8>, DBError> {
    security
        .decrypt(data)
        .map_err(|e| DBError::Io(std::io::Error::other(format!("decryption failed: {}", e))))
}
