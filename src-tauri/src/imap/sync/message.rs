use async_std::stream::StreamExt;
use mailparse::parse_mail;

use crate::db;

impl crate::imap::ImapManager {
    pub fn fetch_message_full_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        match crate::db::fetch_message_full(conn, account_id, mailbox, uid) {
            Ok(message) => Ok(message),
            Err(e) => Err(format!("Failed to fetch message: {}", e)),
        }
    }

    pub async fn fetch_message_full(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        let mut session = self.connect_imap(account_id).await?;
        session.select(mailbox).await.map_err(|e| e.to_string())?;

        let fetch_result = {
            let mut fetches = session
                .fetch(format!("{}", uid), "(BODY[])")
                .await
                .map_err(|e| e.to_string())?;
            if let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let body = fetch.body().ok_or("No body")?;
                let parsed = parse_mail(body).map_err(|e| e.to_string())?;
                let body_html_safe =
                    ammonia::clean(&parsed.get_body().unwrap_or_default()).to_string();
                let body_plain = parsed.get_body().unwrap_or_default();
                let attachments = vec![];
                let inline_images = vec![];

                let message_full = {
                    let conn_guard = self.conn.lock().unwrap();
                    let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
                    db::fetch_message_full(conn, account_id, mailbox, uid)
                        .map_err(|e| e.to_string())?
                };

                if let Some(message) = message_full {
                    Ok(Some(crate::db::MessageFull {
                        header: message.header,
                        body_html_safe,
                        body_plain,
                        attachments,
                        inline_images,
                    }))
                } else {
                    Err("Message not found in database".to_string())
                }
            } else {
                Ok(None)
            }
        };

        session.logout().await.map_err(|e| e.to_string())?;
        fetch_result
    }
}
