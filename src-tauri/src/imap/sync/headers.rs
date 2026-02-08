use async_std::stream::StreamExt;
use chrono::{TimeZone, Utc};

use crate::db::{MailHeader, MessageBatchItem, DEFAULT_BATCH_SIZE};

impl crate::imap::ImapManager {
    pub fn fetch_headers_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        let conn_guard = self.conn.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        crate::db::fetch_headers(conn, account_id, mailbox, anchor, limit)
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

        let uid_set = if let Some(anchor) = anchor {
            format!("{}:*", anchor)
        } else {
            "1:*".to_string()
        };

        let mut batch_items: Vec<MessageBatchItem> = Vec::with_capacity(DEFAULT_BATCH_SIZE);
        let mut headers = Vec::new();
        {
            let mut fetches = session
                .uid_fetch(
                    uid_set,
                    "(UID INTERNALDATE FLAGS ENVELOPE BODY.PEEK[HEADER.FIELDS (SUBJECT FROM TO)])",
                )
                .await
                .map_err(|e| e.to_string())?;
            while let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let uid = fetch.uid.ok_or("No UID")?;
                let envelope = fetch.envelope().ok_or("No envelope")?;
                let subject = crate::utils::mail::decode_mime_header(envelope.subject.as_deref());

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
                let to: Vec<String> = envelope
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

                let header_internal_date = Utc
                    .timestamp_opt(internal_date.timestamp(), 0)
                    .single()
                    .unwrap_or_else(Utc::now);

                let header = MailHeader {
                    uid,
                    message_id: envelope
                        .message_id
                        .as_ref()
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                    internal_date: header_internal_date,
                    subject,
                    from,
                    to: to.clone(),
                    flags,
                    snippet,
                    has_attachments: false,
                };

                batch_items.push(MessageBatchItem {
                    uid,
                    message_id: header.message_id.clone(),
                    internal_date: header.internal_date,
                    from: header.from.first().cloned(),
                    to,
                    subject: header.subject.clone(),
                    snippet: header.snippet.clone(),
                    flags: header.flags.clone(),
                    structure_json: None,
                });

                headers.push(header);

                if batch_items.len() >= DEFAULT_BATCH_SIZE {
                    let mut conn_guard = self.conn.lock().unwrap();
                    let conn = conn_guard.as_mut().ok_or("Database not initialized")?;
                    crate::db::batch_insert_messages(
                        conn,
                        account_id,
                        mailbox,
                        &batch_items,
                        DEFAULT_BATCH_SIZE,
                    )
                    .map_err(|e| e.to_string())?;
                    batch_items.clear();
                }

                if headers.len() >= limit as usize {
                    break;
                }
            }
        }

        if !batch_items.is_empty() {
            let mut conn_guard = self.conn.lock().unwrap();
            let conn = conn_guard.as_mut().ok_or("Database not initialized")?;
            crate::db::batch_insert_messages(
                conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;
        }

        session.logout().await.map_err(|e| e.to_string())?;
        Ok(headers)
    }
    pub async fn fetch_headers_hybrid(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        let mut headers = {
            let conn_guard = self.conn.lock().unwrap();
            let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
            crate::db::fetch_headers(conn, account_id, mailbox, anchor, limit)
                .map_err(|e| e.to_string())?
        };

        if headers.len() < limit as usize {
            let needed = limit - headers.len() as u32;

            let next_anchor = if headers.is_empty() {
                anchor
            } else {
                headers.last().map(|h| h.uid)
            };

            match self
                .fetch_history_from_imap(account_id, mailbox, next_anchor, needed)
                .await
            {
                Ok(mut new_headers) => {
                    if !new_headers.is_empty() {
                        tracing::info!(target: "postail", "[IMAP] Hydrated {} older messages for {}", new_headers.len(), mailbox);
                        headers.append(&mut new_headers);
                    }
                }
                Err(e) => {
                    tracing::error!(target: "postail", "[IMAP] Failed to fetch history: {}", e);
                }
            }
        }

        Ok(headers)
    }

    async fn fetch_history_from_imap(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor_uid: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        // Connect to IMAP
        let mut session = self.connect_imap(account_id).await?;
        let selected = session.select(mailbox).await.map_err(|e| e.to_string())?;
        let total = selected.exists;

        if total == 0 {
            return Ok(Vec::new());
        }

        let range = if let Some(uid) = anchor_uid {
            // Find Sequence Number for this UID
            let mut fetches = session
                .uid_fetch(uid.to_string(), "UID")
                .await
                .map_err(|e| e.to_string())?;
            if let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let seq = fetch.message; // Message Sequence Number

                if seq <= 1 {
                    return Ok(Vec::new()); // No older messages
                }

                let end = seq - 1;
                let start = end.saturating_sub(limit).max(1);
                format!("{}:{}", start, end)
            } else {
                tracing::warn!(target: "postail", "[IMAP] Anchor UID {} not found on server", uid);
                return Ok(Vec::new());
            }
        } else {
            let end = total;
            let start = total.saturating_sub(limit).max(1);
            format!("{}:{}", start, end)
        };

        let mut batch_items: Vec<MessageBatchItem> = Vec::with_capacity(DEFAULT_BATCH_SIZE);
        let mut headers = Vec::new();

        let mut fetches = session
            .fetch(
                range,
                "(UID INTERNALDATE FLAGS ENVELOPE BODY.PEEK[HEADER.FIELDS (SUBJECT FROM TO)])",
            )
            .await
            .map_err(|e| e.to_string())?;

        while let Some(fetch) = fetches.next().await {
            let fetch = fetch.map_err(|e| e.to_string())?;
            let uid = fetch.uid.ok_or("No UID")?;
            let envelope = fetch.envelope().ok_or("No envelope")?;
            let subject = crate::utils::mail::decode_mime_header(envelope.subject.as_deref());

            let from: Vec<String> = envelope
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
            let to: Vec<String> = envelope
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
            let header_internal_date = Utc
                .timestamp_opt(internal_date.timestamp(), 0)
                .single()
                .unwrap_or_else(Utc::now);

            let header = MailHeader {
                uid,
                message_id: envelope
                    .message_id
                    .as_ref()
                    .map(|s| String::from_utf8_lossy(s).to_string()),
                internal_date: header_internal_date,
                subject: subject.clone(),
                from: from.clone(),
                to: to.clone(),
                flags: flags.clone(),
                snippet: None,
                has_attachments: false,
            };

            batch_items.push(MessageBatchItem {
                uid,
                message_id: header.message_id.clone(),
                internal_date: header.internal_date,
                from: header.from.first().cloned(),
                to,
                subject: header.subject.clone(),
                snippet: header.snippet.clone(),
                flags: header.flags.clone(),
                structure_json: None,
            });

            headers.push(header);
        }

        drop(fetches); // Release borrow on session

        if !batch_items.is_empty() {
            let mut conn_guard = self.conn.lock().unwrap();
            let conn = conn_guard.as_mut().ok_or("Database not initialized")?;
            crate::db::batch_insert_messages(
                conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;
        }

        let _ = session.logout().await;
        headers.sort_by(|a, b| b.uid.cmp(&a.uid));

        Ok(headers)
    }
}
