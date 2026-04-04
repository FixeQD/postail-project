use futures::StreamExt;

use crate::db::{Mailbox, fetch_mailboxes as db_fetch_mailboxes, upsert_mailbox};
use crate::globals::get_db_pool;

fn detect_mailbox_role_from_attributes(
    attributes: &[async_imap::types::NameAttribute],
) -> Option<String> {
    for attr in attributes {
        match attr {
            async_imap::types::NameAttribute::All => return Some("all".to_string()),
            async_imap::types::NameAttribute::Archive => return Some("archive".to_string()),
            async_imap::types::NameAttribute::Drafts => return Some("drafts".to_string()),
            async_imap::types::NameAttribute::Flagged => return Some("flagged".to_string()),
            async_imap::types::NameAttribute::Junk => return Some("junk".to_string()),
            async_imap::types::NameAttribute::Sent => return Some("sent".to_string()),
            async_imap::types::NameAttribute::Trash => return Some("trash".to_string()),
            _ => {}
        }
    }
    None
}

impl crate::imap::ImapManager {
    pub async fn fetch_mailboxes_sync(&self, account_id: &str) -> Result<Vec<Mailbox>, String> {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        db_fetch_mailboxes(&*conn, account_id).map_err(|e| e.to_string())
    }

    pub async fn fetch_mailboxes(&self, account_id: &str) -> Result<Vec<Mailbox>, String> {
        tracing::info!(target: "postail", "[IMAP] fetch_mailboxes: calling connect_imap for {}", account_id);
        let mut session = self.connect_imap(account_id).await?;
        tracing::info!(target: "postail", "[IMAP] fetch_mailboxes: connected, listing mailboxes");
        let mut result = Vec::new();
        {
            let mut mailboxes = session
                .list(None, Some("*"))
                .await
                .map_err(|e| e.to_string())?;
            while let Some(mb) = mailboxes.next().await {
                let mb = mb.map_err(|e| e.to_string())?;
                let name = mb.name().to_string();

                // Detect role from SPECIAL-USE attributes (RFC 6154), default to "other"
                let role = detect_mailbox_role_from_attributes(mb.attributes())
                    .unwrap_or_else(|| "other".to_string());
                let mailbox = Mailbox {
                    name,
                    display_name: mb.name().to_string(),
                    role,
                    uid_validity: None,
                    highest_modseq: None,
                    last_synced_uid: None,
                    hidden: false,
                };
                {
                    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
                    let conn = pool.get().map_err(|e| e.to_string())?;
                    upsert_mailbox(&*conn, account_id, &mailbox).map_err(|e| e.to_string())?;
                }
                result.push(mailbox);
            }
        }
        session.logout().await.map_err(|e| e.to_string())?;
        tracing::info!(target: "postail", "[IMAP] fetch_mailboxes: done, got {} mailboxes", result.len());
        Ok(result)
    }
}
