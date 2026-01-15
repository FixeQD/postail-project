use async_std::stream::StreamExt;
use chrono::DateTime;
use serde_json;

use crate::db::{upsert_message, MailHeader};

impl crate::imap::ImapManager {
    pub fn fetch_headers_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        let conn = self.conn.lock().unwrap();
        crate::db::fetch_headers(&conn, account_id, mailbox, anchor, limit)
            .map_err(|e| e.to_string())
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
                let snippet = None;

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
                    has_attachments: false,
                };

                upsert_message(
                    &self.conn.lock().unwrap(),
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
}
