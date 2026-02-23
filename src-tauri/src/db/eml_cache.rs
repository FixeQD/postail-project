use std::fs;
use std::path::PathBuf;

use crate::error::DBError;
use crate::security::SecurityManager;

/// Returns the base cache directory for encrypted EML files.
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

/// Returns the full path to the encrypted EML file for a given message.
pub fn eml_cache_path(account_id: &str, mailbox: &str, uid: u32) -> PathBuf {
    get_eml_cache_dir()
        .join(sanitize_path_component(account_id))
        .join(sanitize_path_component(mailbox))
        .join(format!("{}.eml.enc", uid))
}

/// Returns true if an encrypted EML file exists for this message.
pub fn has_cached_eml(account_id: &str, mailbox: &str, uid: u32) -> bool {
    eml_cache_path(account_id, mailbox, uid).exists()
}

/// Encrypts and saves raw EML bytes to disk.
pub fn save_eml(
    security: &SecurityManager,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    raw_eml: &[u8],
) -> Result<PathBuf, DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(DBError::Io)?;
    }

    let encrypted = security.encrypt(raw_eml).map_err(|e| {
        DBError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("EML encryption failed: {}", e),
        ))
    })?;

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

    let decrypted = security.decrypt(&encrypted).map_err(|e| {
        DBError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("EML decryption failed: {}", e),
        ))
    })?;

    Ok(Some(decrypted))
}

/// Deletes the cached EML file for a message
pub fn delete_eml(account_id: &str, mailbox: &str, uid: u32) -> Result<(), DBError> {
    let path = eml_cache_path(account_id, mailbox, uid);
    if path.exists() {
        fs::remove_file(&path).map_err(DBError::Io)?;
    }
    Ok(())
}

/// Deletes all cached EML files for a given account.
pub fn delete_account_eml_cache(account_id: &str) -> Result<(), DBError> {
    let dir = get_eml_cache_dir().join(sanitize_path_component(account_id));
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(DBError::Io)?;
    }
    Ok(())
}
