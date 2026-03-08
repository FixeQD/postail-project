use crate::db::Contact;
use crate::globals::{DB_CONN, SECURITY};
use std::sync::Arc;
use tauri::command;

#[command]
pub async fn dev_reset_data(
    clear_messages: bool,
    clear_eml_cache: bool,
    clear_body_cache: bool,
    clear_attachments: bool,
    clear_contacts: bool,
    clear_settings: bool,
    clear_outbox: bool,
) -> Result<Vec<String>, String> {
    let db_conn = Arc::clone(&DB_CONN);

    let mut log: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut log: Vec<String> = Vec::new();
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

        if clear_messages {
            conn.execute_batch("DELETE FROM messages; DELETE FROM attachments;")
                .map_err(|e| e.to_string())?;
            log.push("Cleared messages + attachments table".into());
        }

        if clear_attachments && !clear_messages {
            conn.execute("DELETE FROM attachments", [])
                .map_err(|e| e.to_string())?;
            log.push("Cleared attachments table".into());
        }

        if clear_contacts {
            conn.execute("DELETE FROM contacts", [])
                .map_err(|e| e.to_string())?;
            log.push("Cleared contacts table".into());
        }

        if clear_settings {
            conn.execute("DELETE FROM settings", [])
                .map_err(|e| e.to_string())?;
            log.push("Cleared settings table".into());
        }

        if clear_outbox {
            conn.execute("DELETE FROM outbox", [])
                .map_err(|e| e.to_string())?;
            log.push("Cleared outbox table".into());
        }

        Ok::<Vec<String>, String>(log)
    })
    .await
    .map_err(|e| e.to_string())??;

    // File cache ops outside spawn_blocking (they're sync fs but quick)
    let cache_dir = crate::db::eml_cache::get_eml_cache_dir();

    if clear_eml_cache || clear_body_cache {
        if cache_dir.exists() {
            // Walk account dirs
            if let Ok(accounts) = std::fs::read_dir(&cache_dir) {
                for account_entry in accounts.flatten() {
                    let account_path = account_entry.path();
                    if !account_path.is_dir() {
                        continue;
                    }
                    // Walk mailbox dirs
                    if let Ok(mailboxes) = std::fs::read_dir(&account_path) {
                        for mb_entry in mailboxes.flatten() {
                            let mb_path = mb_entry.path();
                            if !mb_path.is_dir() {
                                continue;
                            }
                            // Walk uid dirs
                            if let Ok(uids) = std::fs::read_dir(&mb_path) {
                                for uid_entry in uids.flatten() {
                                    let uid_path = uid_entry.path();
                                    if !uid_path.is_dir() {
                                        continue;
                                    }
                                    if clear_eml_cache {
                                        let eml = uid_path.join("eml.enc");
                                        if eml.exists() {
                                            let _ = std::fs::remove_file(&eml);
                                        }
                                    }
                                    if clear_body_cache {
                                        let body = uid_path.join("body.json.enc");
                                        if body.exists() {
                                            let _ = std::fs::remove_file(&body);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if clear_eml_cache {
            log.push("Cleared EML file cache".into());
        }
        if clear_body_cache {
            log.push("Cleared body JSON cache".into());
        }
    }

    Ok(log)
}

#[command]
pub async fn export_backup(passphrase: Option<String>) -> Result<String, String> {
    let db_conn = Arc::clone(&DB_CONN);
    let security = Arc::clone(&SECURITY);
    let passphrase_clone = passphrase;
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        let sec = security.blocking_lock();
        crate::db::export_backup(conn, &sec, passphrase_clone)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn import_backup(backup_path: String, passphrase: Option<String>) -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    let security = Arc::clone(&SECURITY);
    let path = std::path::PathBuf::from(backup_path);
    let passphrase_clone = passphrase;
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        let sec = security.blocking_lock();
        crate::db::import_backup(conn, &sec, &path, passphrase_clone).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn run_maintenance() -> Result<(), String> {
    let db_conn = Arc::clone(&DB_CONN);
    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().expect("Database not initialized");
        crate::db::run_maintenance(conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn search_contacts(query: String, limit: u32) -> Result<Vec<Contact>, String> {
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
    crate::db::search_contacts(conn, &query, limit).map_err(|e| e.to_string())
}
