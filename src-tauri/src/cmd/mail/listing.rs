use crate::db::{AttachmentMeta, MailHeader, Mailbox, MessageFull};
use crate::globals::{get_db_pool, IMAP_MANAGER};
use crate::oauth;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadMessage {
    pub header: MailHeader,
    pub body_html_safe: String,
    pub body_plain: String,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadView {
    pub messages: Vec<ThreadMessage>,
}

#[command]
pub async fn fetch_raw_eml_text(
    account_id: String,
    mailbox: String,
    uid: u64,
) -> Result<String, String> {
    let uid_u32: u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    let security = crate::globals::SECURITY.lock().await;

    // Try disk cache first
    if let Ok(Some(raw)) = crate::db::eml_cache::load_eml(&security, &account_id, &mailbox, uid_u32)
    {
        return String::from_utf8(raw).map_err(|_| "EML contains non-UTF-8 bytes".to_string());
    }

    drop(security);

    // Not cached yet - fetch from IMAP and cache it
    let imap = IMAP_MANAGER.lock().await.clone();
    let raw = imap
        .fetch_raw_eml_bytes(&account_id, &mailbox, uid_u32)
        .await?;

    let security = crate::globals::SECURITY.lock().await;
    let _ = crate::db::eml_cache::save_eml(&security, &account_id, &mailbox, uid_u32, &raw);

    String::from_utf8(raw).map_err(|_| "EML contains non-UTF-8 bytes".to_string())
}

#[command]
pub async fn fetch_mailboxes(account_id: String) -> Result<Vec<Mailbox>, String> {
    let imap = IMAP_MANAGER.lock().await;
    let mut mailboxes = imap.fetch_mailboxes_sync(&account_id).await?;

    let provider_kind = {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        if let Ok(conn) = pool.get() {
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
pub async fn update_mailbox_role(
    account_id: String,
    mailbox_name: String,
    role: String,
) -> Result<(), String> {
    let valid_roles = [
        "inbox", "sent", "drafts", "trash", "archive", "junk", "flagged", "all", "other",
    ];
    if !valid_roles.contains(&role.as_str()) {
        return Err(format!("Invalid role: {}", role));
    }

    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE mailboxes SET role = ?, role_customized = 1 WHERE account_id = ? AND name = ?",
        params![role, account_id, mailbox_name],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn fetch_headers(
    account_id: String,
    mailbox: String,
    anchor: Option<u64>,
    limit: u32,
) -> Result<Vec<MailHeader>, String> {
    tracing::info!(target: "postail", "[API] fetch_headers called for {}@{} anchor={:?} limit={}", mailbox, account_id, anchor, limit);

    // Handle virtual starred mailbox
    if mailbox == "Virtual_Starred" {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        return crate::db::fetch_starred_headers(&conn, &account_id, limit)
            .map_err(|e| e.to_string());
    }

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
    let uid_u32: u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    tracing::info!(
        target: "postail",
        "[API] fetch_message_full START uid={} mailbox={}",
        uid_u32, mailbox
    );

    // Step 1: check if parsed body file exists on disk (never touches SQLite)
    let body_on_disk = crate::db::eml_cache::has_cached_body_file(&account_id, &mailbox, uid_u32);
    tracing::info!(
        target: "postail",
        "[API] fetch_message_full body_on_disk={} uid={} mailbox={}",
        body_on_disk, uid_u32, mailbox
    );

    if body_on_disk {
        // Load header from DB (tiny read, no large blobs)
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        if let Ok(conn) = pool.get() {
            let mut msg = crate::db::fetch_message_full(&conn, &account_id, &mailbox, uid_u32)
                .map_err(|e| e.to_string())?;

            if let Some(ref mut m) = msg {
                // Inject body from encrypted file — zero DB reads for body content
                let security = crate::globals::SECURITY.lock().await;
                match crate::db::eml_cache::load_body(&security, &account_id, &mailbox, uid_u32) {
                    Ok(Some(mut body)) => {
                        tracing::info!(
                            target: "postail",
                            "[API] fetch_message_full file cache HIT uid={} html_len={} plain_len={}",
                            uid_u32, body.body_html.len(), body.body_plain.len()
                        );

                        // Old cache files predate read_receipt_to - try to backfill from raw EML.
                        if body.read_receipt_to.is_none() {
                            if let Ok(Some(raw_eml)) = crate::db::eml_cache::load_eml(
                                &security,
                                &account_id,
                                &mailbox,
                                uid_u32,
                            ) {
                                use mailparse::{parse_mail, MailHeaderMap};
                                if let Ok(parsed) = parse_mail(&raw_eml) {
                                    let receipt = parsed
                                        .headers
                                        .get_first_value("Disposition-Notification-To")
                                        .or_else(|| {
                                            parsed.headers.get_first_value("Return-Receipt-To")
                                        });

                                    if receipt.is_some() {
                                        body.read_receipt_to = receipt;
                                        // Persist the backfilled value so we don't re-parse next time.
                                        let _ = crate::db::eml_cache::save_body(
                                            &security,
                                            &account_id,
                                            &mailbox,
                                            uid_u32,
                                            &body,
                                        );
                                        tracing::info!(
                                            target: "postail",
                                            "[API] fetch_message_full backfilled read_receipt_to from raw EML uid={}",
                                            uid_u32
                                        );
                                    }
                                }
                            }
                        }

                        m.body_html_safe = body.body_html;
                        m.body_plain = body.body_plain;
                        m.read_receipt_to = body.read_receipt_to;
                    }
                    Ok(None) => {
                        tracing::warn!(target: "postail",
                            "[API] fetch_message_full body file missing despite has_cached_body_file=true uid={}", uid_u32);
                    }
                    Err(e) => {
                        tracing::error!(target: "postail",
                            "[API] fetch_message_full body file load error uid={} error={}", uid_u32, e);
                    }
                }
                return Ok(msg);
            }
        }
    }

    // Step 2: fetch from IMAP (or re-parse from existing EML file), cache, return
    tracing::info!(
        target: "postail",
        "[API] fetch_message_full => fetch_and_cache_message uid={} mailbox={}",
        uid_u32, mailbox
    );
    let imap = IMAP_MANAGER.lock().await.clone();
    let result = imap
        .fetch_and_cache_message(&account_id, &mailbox, uid_u32)
        .await;

    tracing::info!(
        target: "postail",
        "[API] fetch_message_full fetch_and_cache_message result: ok={} uid={} mailbox={}",
        result.is_ok(),
        uid_u32, mailbox
    );
    if let Err(ref e) = result {
        tracing::error!(
            target: "postail",
            "[API] fetch_message_full FAILED uid={} mailbox={} error={}",
            uid_u32, mailbox, e
        );
    }

    result
}

#[command]
pub async fn fetch_thread(
    account_id: String,
    mailbox: String,
    uid: u64,
) -> Result<ThreadView, String> {
    let uid_u32: u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    tracing::info!(
        target: "postail",
        "[API] fetch_thread START uid={} mailbox={}",
        uid_u32, mailbox
    );

    // Collect UIDs first without holding guard
    let thread_uids: Vec<u32> = {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        // Get current message's message_id
        let current_message_id: Option<String> = conn
            .query_row(
                "SELECT message_id FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
                params![&account_id, &mailbox, uid_u32],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let current_message_id = match current_message_id {
            Some(id) => id,
            None => {
                // No message found, return empty thread
                return Ok(ThreadView { messages: vec![] });
            }
        };

        // Get current message's subject for fallback search
        let current_subject: Option<String> = conn
            .query_row(
                "SELECT subject FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
                params![&account_id, &mailbox, uid_u32],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        // Find all messages in thread by message_id (exact match or conversation)
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT uid FROM messages
                 WHERE account_id = ? AND mailbox = ? AND (message_id = ? OR (subject = ? AND subject IS NOT NULL AND subject != ''))
                 ORDER BY internal_date ASC, uid ASC",
            )
            .map_err(|e| e.to_string())?;

        let uids: Vec<u32> = stmt
            .query_map(
                params![
                    &account_id,
                    &mailbox,
                    &current_message_id,
                    current_subject.as_deref().unwrap_or("")
                ],
                |row| row.get::<_, u32>(0),
            )
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        uids
    };

    // Now load full messages
    let security = crate::globals::SECURITY.lock().await;
    let mut thread_messages = vec![];

    for thread_uid in thread_uids {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;

        // Fetch full message
        let msg = crate::db::fetch_message_full(&conn, &account_id, &mailbox, thread_uid)
            .map_err(|e| e.to_string())?;

        if let Some(mut msg) = msg {
            // Load body from cache
            match crate::db::eml_cache::load_body(&security, &account_id, &mailbox, thread_uid) {
                Ok(Some(body)) => {
                    msg.body_html_safe = body.body_html;
                    msg.body_plain = body.body_plain;
                    msg.read_receipt_to = body.read_receipt_to;
                }
                _ => {
                    // Leave empty if no cached body
                }
            }

            thread_messages.push(ThreadMessage {
                header: msg.header,
                body_html_safe: msg.body_html_safe,
                body_plain: msg.body_plain,
                is_current: thread_uid == uid_u32,
            });
        }
    }

    tracing::info!(
        target: "postail",
        "[API] fetch_thread found {} messages uid={} mailbox={}",
        thread_messages.len(),
        uid_u32,
        mailbox
    );

    Ok(ThreadView {
        messages: thread_messages,
    })
}

#[command]
pub async fn save_attachment(
    account_id: String,
    mailbox: String,
    uid: u64,
    part_id: String,
) -> Result<AttachmentMeta, String> {
    use crate::globals::get_db_pool;

    let uid_u32: u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;

    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let row = conn
        .query_row(
            "SELECT a.part_id, a.filename, a.mime_type, a.size, a.cached_path, a.cid
             FROM attachments a
             JOIN messages m ON m.id = a.message_table_id
             WHERE m.account_id = ? AND m.mailbox = ? AND m.uid = ? AND a.part_id = ?",
            rusqlite::params![account_id, mailbox, uid_u32, part_id],
            |row| {
                Ok(AttachmentMeta {
                    part_id: row.get(0)?,
                    filename: row.get(1)?,
                    mime_type: row.get(2)?,
                    size: row.get::<_, i64>(3)? as u64,
                    cached_path: row.get(4)?,
                    cid: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(row)
}
