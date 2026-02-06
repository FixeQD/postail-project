use crate::db::{MailHeader, Mailbox, MessageFull};
use crate::globals::{DB_CONN, IMAP_MANAGER};
use crate::oauth;
use tauri::command;

#[command]
pub fn fetch_mailboxes(account_id: String) -> Result<Vec<Mailbox>, String> {
    let imap = IMAP_MANAGER.blocking_lock();
    let mut mailboxes = imap.fetch_mailboxes_sync(&account_id)?;

    let provider_kind = {
        let conn_guard = DB_CONN.lock().unwrap();
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

    for mailbox in &mut mailboxes {
        let decoded = utf7_imap::decode_utf7_imap(mailbox.name.clone());
        mailbox.display_name = decoded.clone();

        let lower = decoded.to_lowercase();
        mailbox.role = "other".to_string();

        if lower == "inbox" {
            mailbox.role = "inbox".to_string();
            mailbox.display_name = "Inbox".to_string();
        } else if lower.contains("draft") {
            mailbox.role = "drafts".to_string();
        } else if lower.contains("sent") {
            mailbox.role = "sent".to_string();
        } else if lower.contains("trash") || lower.contains("bin") || lower.contains("deleted") {
            mailbox.role = "trash".to_string();
        } else if lower.contains("junk") || lower.contains("spam") {
            mailbox.role = "junk".to_string();
        } else if lower.contains("archive") {
            mailbox.role = "archive".to_string();
        }

        if let Some(kind) = provider_kind {
            let info = oauth::ProviderInfo::get(kind);
            if mailbox.name == info.sent_folder {
                mailbox.role = "sent".to_string();
            }

            if kind == oauth::ProviderKind::Gmail && mailbox.display_name.starts_with("[Gmail]/") {
                mailbox.display_name = mailbox.display_name.replace("[Gmail]/", "");
            }
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
    let anchor: Option<u32> = anchor
        .map(|a| a.try_into().map_err(|_| "Anchor too large".to_string()))
        .transpose()?;
    let imap = IMAP_MANAGER.lock().await.clone();
    imap.fetch_headers_hybrid(&account_id, &mailbox, anchor, limit)
        .await
}

#[command]
pub fn fetch_message_full(
    account_id: String,
    mailbox: String,
    uid: u64,
) -> Result<Option<MessageFull>, String> {
    let uid_u32 = uid.try_into().map_err(|_| "UID too large".to_string())?;
    let imap = IMAP_MANAGER.blocking_lock();
    imap.fetch_message_full_sync(&account_id, &mailbox, uid_u32)
}
