use crate::db::{MailHeader, Mailbox, MessageFull};
use crate::globals::{DB_CONN, IMAP_MANAGER};
use crate::oauth;
use tauri::command;

#[command]
pub async fn fetch_mailboxes(account_id: String) -> Result<Vec<Mailbox>, String> {
    let imap = IMAP_MANAGER.lock().await;
    let mut mailboxes = imap.fetch_mailboxes_sync(&account_id).await?;

    let provider_kind = {
        let conn_guard = DB_CONN.lock().await;
        if let Some(conn) = conn_guard.as_ref() {
            let mut stmt = conn
                .prepare("SELECT provider_type, imap_host FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            let mut rows = stmt.query([&account_id]).map_err(|e| e.to_string())?;

            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let provider_type: String = row.get(0).unwrap_or_default();
                let imap_host: String = row.get(1).unwrap_or_default();

                oauth::ProviderKind::parse(&provider_type)
                    .or_else(|| oauth::ProviderKind::from_imap_host(&imap_host))
            } else {
                None
            }
        } else {
            None
        }
    };

    let canonical_prefix = provider_kind.and_then(|k| oauth::ProviderInfo::get(k).canonical_prefix);

    // Filter out provider namespace folder (e.g., "[Gmail]" for Gmail)
    if let Some(prefix) = canonical_prefix {
        let namespace = prefix.trim_end_matches('/');
        mailboxes.retain(|mb| mb.name != namespace);
    }

    if let Some(prefix) = canonical_prefix {
        let canonical_roles = &[
            "sent", "trash", "drafts", "junk", "flagged", "all", "archive",
        ];

        let mut canonical_roles_seen = std::collections::HashSet::new();
        for mb in &mailboxes {
            if mb.name.starts_with(prefix) && canonical_roles.contains(&mb.role.as_str()) {
                canonical_roles_seen.insert(mb.role.clone());
            }
        }

        mailboxes.retain(|mb| {
            if mb.name.starts_with(prefix) {
                return true; // Keep all canonical folders
            }
            !canonical_roles_seen.contains(&mb.role)
        });
    }

    for mailbox in &mut mailboxes {
        // Decode UTF-7 IMAP to display name
        let decoded = utf7_imap::decode_utf7_imap(mailbox.name.clone());
        mailbox.display_name = decoded.clone();

        // Clean up provider-specific prefixes
        if let Some(kind) = provider_kind {
            let info = oauth::ProviderInfo::get(kind);
            mailbox.display_name = info.strip_display_name_prefix(&mailbox.display_name);
        }
    }

    mailboxes.sort_by(|a, b| {
        let role_score = |role: &str| match role {
            "inbox" => 0,
            "sent" => 1,
            "drafts" => 2,
            "trash" => 3,
            "archive" => 4,
            "junk" => 5,
            "flagged" => 6,
            "all" => 7,
            _ => 10,
        };

        let score_a = role_score(&a.role);
        let score_b = role_score(&b.role);

        if score_a != score_b {
            score_a.cmp(&score_b)
        } else {
            a.display_name.cmp(&b.display_name)
        }
    });

    Ok(mailboxes)
}

#[command]
pub async fn fetch_headers(
    account_id: String,
    mailbox: String,
    anchor: Option<u64>,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    tracing::info!(target: "postail", "[API] fetch_headers called for {}@{} anchor={:?} limit={}", mailbox, account_id, anchor, limit);
    let anchor: Option<u32> = anchor
        .map(|a| a.try_into().map_err(|_| "Anchor too large".to_string()))
        .transpose()?;
    let imap = IMAP_MANAGER.lock().await.clone();
    let result = imap
        .fetch_headers_hybrid(&account_id, &mailbox, anchor, limit)
        .await;
    match &result {
        Ok(headers) => {
            tracing::info!(target: "postail", "[API] fetch_headers returned {} headers", headers.len())
        }
        Err(e) => tracing::error!(target: "postail", "[API] fetch_headers error: {}", e),
    }
    result
}

#[command]
pub async fn fetch_message_full(
    account_id: String,
    mailbox: String,
    uid: u64,
) -> Result<Option<MessageFull>, String> {
    let uid_u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    // Release the lock before doing heavy work
    let imap = {
        let guard = IMAP_MANAGER.lock().await;
        guard.clone()
    };

    // Try DB first
    if let Ok(Some(msg)) = imap
        .fetch_message_full_sync(&account_id, &mailbox, uid_u32)
        .await
    {
        if !msg.body_html_safe.is_empty() || !msg.body_plain.is_empty() {
            tracing::info!(target: "postail", "[API] fetch_message_full: cache hit for uid={}", uid_u32);
            return Ok(Some(msg));
        }
    }

    tracing::info!(target: "postail", "[API] fetch_message_full: cache miss, fetching from IMAP for uid={}", uid_u32);
    let result = imap
        .fetch_message_full(&account_id, &mailbox, uid_u32)
        .await;

    if let Ok(None) = &result {
        tracing::warn!(target: "postail", "[API] fetch_message_full: IMAP returned None for uid={}", uid_u32);
    }

    result
}
#[command]
pub async fn save_attachment(
    app: tauri::AppHandle,
    account_id: String,
    mailbox: String,
    uid: u64,
    part_id: String,
    filename: String,
) -> Result<bool, String> {
    use futures::StreamExt;
    use mailparse::parse_mail;
    use std::fs;
    use tauri_plugin_dialog::DialogExt;

    let uid_u32: u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    // 1. Fetch message from IMAP
    let imap = crate::globals::IMAP_MANAGER.lock().await.clone();
    let mut session = imap.connect_imap(&account_id).await?;
    session.select(&mailbox).await.map_err(|e| e.to_string())?;

    let mut fetches = session
        .uid_fetch(format!("{}", uid_u32), "(BODY[])")
        .await
        .map_err(|e| e.to_string())?;

    let fetch = fetches
        .next()
        .await
        .ok_or("Message not found")?
        .map_err(|e| e.to_string())?;
    let body = fetch.body().ok_or("No body in fetch result")?.to_vec();
    drop(fetch);
    drop(fetches);
    let _ = session.logout().await;

    // 2. Parse and find part
    let parsed = parse_mail(&body).map_err(|e| e.to_string())?;

    fn find_part_bytes<'a>(
        part: &'a mailparse::ParsedMail<'a>,
        target_id: &str,
        current_id: &mut usize,
    ) -> Option<Vec<u8>> {
        let my_id = current_id.to_string();
        *current_id += 1;

        if my_id == target_id {
            return Some(part.get_body_raw().unwrap_or_default());
        }

        for sub in &part.subparts {
            if let Some(bytes) = find_part_bytes(sub, target_id, current_id) {
                return Some(bytes);
            }
        }
        None
    }

    let bytes = find_part_bytes(&parsed, &part_id, &mut 0).ok_or("Attachment part not found")?;

    // 3. Save dialog
    let save_path = app
        .dialog()
        .file()
        .set_file_name(&filename)
        .blocking_save_file();

    match save_path {
        Some(target) => {
            let target_path = match target {
                tauri_plugin_dialog::FilePath::Path(p) => p,
                tauri_plugin_dialog::FilePath::Url(u) => u
                    .to_file_path()
                    .map_err(|_| "Invalid URL target".to_string())?,
            };
            fs::write(target_path, bytes).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false), // User cancelled
    }
}
