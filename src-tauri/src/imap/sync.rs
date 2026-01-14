use crate::db::{fetch_mailboxes as db_fetch_mailboxes, upsert_mailbox, upsert_message, MailHeader, Mailbox};
use async_imap::Session;
use async_native_tls::TlsStream;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use chrono::DateTime;
use mailparse::parse_mail;
use rusqlite::Connection;
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

impl super::ImapManager {
    pub fn fetch_mailboxes_sync(&self, account_id: &str) -> Result<Vec<Mailbox>, String> {
        let conn = self.conn.lock().unwrap();
        db_fetch_mailboxes(&*conn, account_id).map_err(|e| e.to_string())
    }

    pub async fn fetch_mailboxes(&self, account_id: &str) -> Result<Vec<Mailbox>, String> {
        let mut session = self.connect_imap(account_id).await?;
        let mut result = Vec::new();
        {
            let mut mailboxes = session
                .list(None, Some("*"))
                .await
                .map_err(|e| e.to_string())?;
            while let Some(mb) = mailboxes.next().await {
                let mb = mb.map_err(|e| e.to_string())?;
                let name = mb.name().to_string();
                let mailbox = Mailbox {
                    name,
                    uid_validity: None,
                    highest_modseq: None,
                    last_synced_uid: None,
                };
                upsert_mailbox(&*self.conn.lock().unwrap(), account_id, &mailbox)
                    .map_err(|e| e.to_string())?;
                result.push(mailbox);
            }
        }
        session.logout().await.map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn fetch_headers_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        let conn = self.conn.lock().unwrap();
        crate::db::fetch_headers(&*conn, account_id, mailbox, anchor, limit).map_err(|e| e.to_string())
    }

    pub async fn fetch_headers(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        let mut session = self.connect_imap(account_id).await?;
        session.select(mailbox).await.map_err(|e| e.to_string())?;

        let sequence_set = if let Some(anchor) = anchor {
            format!("{}:*", anchor + 1)
        } else {
            "1:*".to_string()
        };

        let mut headers = Vec::new();
        {
            let mut fetches = session
                .fetch(
                    sequence_set,
                    "(UID INTERNALDATE FLAGS ENVELOPE BODY.PEEK[HEADER.FIELDS (SUBJECT FROM TO)])",
                )
                .await
                .map_err(|e| e.to_string())?;
            while let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let uid = fetch.uid.ok_or("No UID")?;
                let envelope = fetch.envelope().ok_or("No envelope")?;
                let subject = envelope
                    .subject
                    .map(|s| String::from_utf8_lossy(s).to_string());
                let from = envelope
                    .from
                    .as_ref()
                    .map(|addrs| {
                        addrs
                            .iter()
                            .map(|a| {
                                let mailbox = a
                                    .mailbox
                                    .as_ref()
                                    .map(|b| String::from_utf8_lossy(b))
                                    .unwrap_or_default();
                                let host = a
                                    .host
                                    .as_ref()
                                    .map(|b| String::from_utf8_lossy(b))
                                    .unwrap_or_default();
                                format!("{}@{}", mailbox, host)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let to = envelope
                    .to
                    .as_ref()
                    .map(|addrs| {
                        addrs
                            .iter()
                            .map(|a| {
                                let mailbox = a
                                    .mailbox
                                    .as_ref()
                                    .map(|b| String::from_utf8_lossy(b))
                                    .unwrap_or_default();
                                let host = a
                                    .host
                                    .as_ref()
                                    .map(|b| String::from_utf8_lossy(b))
                                    .unwrap_or_default();
                                format!("{}@{}", mailbox, host)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let flags = fetch
                    .flags()
                    .map(|flag| format!("{:?}", flag))
                    .collect::<Vec<_>>();
                let internal_date = fetch.internal_date().ok_or("No internal date")?;
                let snippet = None; // TODO: generate snippet

                let header = MailHeader {
                    uid,
                    message_id: envelope
                        .message_id
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                    internal_date: DateTime::from_timestamp(internal_date.timestamp(), 0).unwrap(),
                    subject,
                    from,
                    to,
                    flags,
                    snippet,
                    has_attachments: false, // TODO
                };

                upsert_message(
                    &*self.conn.lock().unwrap(),
                    account_id,
                    mailbox,
                    uid,
                    header.message_id.as_deref(),
                    header.internal_date,
                    header.from.first().map(|s| s.as_str()),
                    Some(&serde_json::to_string(&header.to).unwrap()),
                    header.subject.as_deref(),
                    header.snippet.as_deref(),
                    Some(&serde_json::to_string(&header.flags).unwrap()),
                    None,
                )
                .map_err(|e| e.to_string())?;

                headers.push(header);
                if headers.len() >= limit as usize {
                    break;
                }
            }
        }

        session.logout().await.map_err(|e| e.to_string())?;
        Ok(headers)
    }

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
                // TODO: Parse HTML, plain, attachments
                let body_html_safe =
                    ammonia::clean(&parsed.get_body().unwrap_or_default()).to_string();
                let body_plain = parsed.get_body().unwrap_or_default();
                let attachments = vec![]; // TODO
                let inline_images = vec![]; // TODO

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

    pub fn start_sync(&self, account_id: &str) -> Result<(), String> {
        Ok(())
    }
}
