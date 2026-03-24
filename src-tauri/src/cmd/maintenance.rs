use crate::db::Contact;
use crate::globals::{DB_CONN, SECURITY};
use std::sync::Arc;
use tauri::command;

fn make_snippet(plain: &str, html: &str) -> String {
    let source = if !plain.is_empty() {
        plain.to_string()
    } else {
        use kuchikiki::traits::TendrilSink;
        let doc = kuchikiki::parse_html().one(html).document_node;
        doc.text_contents()
    };
    source
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}

#[command]
pub async fn clear_cache() -> Result<u64, String> {
    let cache_dir = crate::db::eml_cache::get_eml_cache_dir();

    if !cache_dir.exists() {
        return Ok(0);
    }

    let freed = dir_size(&cache_dir);

    std::fs::remove_dir_all(&cache_dir).map_err(|e| format!("Failed to clear cache: {}", e))?;

    tracing::info!(target: "postail", "[Cache] Cleared eml_cache, freed {} bytes", freed);

    Ok(freed)
}

fn dir_size(path: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries.flatten().fold(0u64, |acc, entry| {
        let p = entry.path();
        if p.is_file() {
            acc + entry.metadata().map(|m| m.len()).unwrap_or(0)
        } else if p.is_dir() {
            acc + dir_size(&p)
        } else {
            acc
        }
    })
}

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

#[command]
pub async fn backfill_snippets(account_id: String, mailbox: String) -> Result<u32, String> {
    // Find messages with no snippet but a body cache file on disk
    let rows: Vec<(i64, u32)> = {
        let conn_guard = DB_CONN.lock().await;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let mut stmt = conn
            .prepare(
                "SELECT id, uid FROM messages
                 WHERE account_id = ? AND mailbox = ?
                   AND (snippet IS NULL OR snippet = '')",
            )
            .map_err(|e| e.to_string())?;
        let collected: Vec<(i64, u32)> = stmt
            .query_map(rusqlite::params![account_id, mailbox], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, u32>(1)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        collected
    };

    if rows.is_empty() {
        return Ok(0);
    }

    let security = SECURITY.lock().await;
    let mut updated = 0u32;

    for (table_id, uid) in rows {
        let body = match crate::db::eml_cache::load_body(&security, &account_id, &mailbox, uid) {
            Ok(Some(b)) => b,
            _ => continue,
        };

        let snippet = make_snippet(&body.body_plain, &body.body_html);
        if snippet.is_empty() {
            continue;
        }

        let conn_guard = DB_CONN.lock().await;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let rows_changed = conn
            .execute(
                "UPDATE messages SET snippet = ? WHERE id = ? AND (snippet IS NULL OR snippet = '')",
                rusqlite::params![snippet, table_id],
            )
            .map_err(|e| e.to_string())?;

        if rows_changed > 0 {
            updated += 1;
        }
    }

    tracing::info!(
        target: "postail",
        "[Snippets] Backfilled {} snippets for {}@{}",
        updated, mailbox, account_id
    );

    Ok(updated)
}
