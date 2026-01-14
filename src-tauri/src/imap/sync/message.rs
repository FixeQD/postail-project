use std::sync::{Arc, Mutex};

use async_imap::Session;
use async_native_tls::TlsStream;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use mailparse::parse_mail;
use rusqlite::Connection;

impl crate::imap::ImapManager {
    pub fn fetch_message_full_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        let conn = self.conn.lock().unwrap();
        crate::db::fetch_message_full(&*conn, account_id, mailbox, uid).map_err(|e| e.to_string())
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

                let header = crate::db::fetch_headers(
                    &*self.conn.lock().unwrap(),
                    account_id,
                    mailbox,
                    Some(uid - 1),
                    1,
                )
                .map_err(|e| e.to_string())?
                .into_iter()
                .next()
                .ok_or("No header")?;

                Some(crate::db::MessageFull {
                    header,
                    body_html_safe,
                    body_plain,
                    attachments,
                    inline_images,
                })
            } else {
                None
            }
        };

        session.logout().await.map_err(|e| e.to_string())?;
        Ok(fetch_result)
    }
}
