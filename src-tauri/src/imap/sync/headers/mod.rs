pub mod utils;
pub mod snippets;

use chrono::{TimeZone, Utc};
use futures::StreamExt;

use crate::db::{DEFAULT_BATCH_SIZE, MailHeader, MessageBatchItem};
use crate::globals::get_db_pool;
use crate::parse_address_list;
use utils::*;
use snippets::*;

pub struct SnippetTarget {
    pub uid: u32,
    pub section: String,
    pub mime: String,
    pub charset: String,
    pub encoding: String,
}

impl crate::imap::ImapManager {
    pub async fn fetch_headers_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        anchor: Option<u32>,
        limit: u32,
    ) -> Result<Vec<MailHeader>, String> {
        let pool = get_db_pool().await.map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        crate::db::fetch_headers(&*conn, account_id, mailbox, anchor, limit)
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

        let uid_set = match anchor {
            Some(a) => format!("{}:*", a),
            None => "1:*".to_string(),
        };

        let mut batch_items: Vec<MessageBatchItem> = Vec::with_capacity(DEFAULT_BATCH_SIZE);
        let mut headers: Vec<MailHeader> = Vec::new();
        let mut snippet_targets: Vec<SnippetTarget> = Vec::new();

        // Pass 1 – headers + BODYSTRUCTURE
        {
            let mut fetches = session
                .uid_fetch(uid_set, "(UID INTERNALDATE FLAGS ENVELOPE BODYSTRUCTURE)")
                .await
                .map_err(|e| e.to_string())?;

            while let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let uid = fetch.uid.ok_or("No UID")?;
                let envelope = fetch.envelope().ok_or("No envelope")?;

                let subject = crate::utils::mail::decode_mime_header(envelope.subject.as_deref());
                let from = envelope
                    .from
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let to = envelope
                    .to
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let cc = envelope
                    .cc
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let all_flags = fetch
                    .flags()
                    .map(|f| flag_to_string(&f))
                    .collect::<Vec<_>>();

                let mut system_flags = Vec::new();
                let mut tags = Vec::new();
                for f in all_flags {
                    if f.starts_with('\\') {
                        system_flags.push(f);
                    } else {
                        tags.push(f);
                    }
                }

                let internal_date = fetch.internal_date().ok_or("No internal date")?;
                let internal_date = Utc
                    .timestamp_opt(internal_date.timestamp(), 0)
                    .single()
                    .unwrap_or_else(Utc::now);

                if let Some((path, mime, charset, encoding)) =
                    fetch.bodystructure().and_then(|bs| find_text_part(bs, &[]))
                {
                    snippet_targets.push(SnippetTarget {
                        uid,
                        section: section_path_to_string(&path),
                        mime,
                        charset,
                        encoding,
                    });
                }

                let header = MailHeader {
                    uid,
                    mailbox: mailbox.to_string(),
                    message_id: envelope
                        .message_id
                        .as_ref()
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                    internal_date,
                    subject: subject.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    cc: cc.clone(),
                    flags: system_flags.clone(),
                    snippet: None,
                    has_attachments: false,
                    starred: system_flags.iter().any(|f| f == "\\Flagged"),
                    tags: tags.clone(),
                };

                batch_items.push(MessageBatchItem {
                    uid,
                    message_id: header.message_id.clone(),
                    internal_date: header.internal_date,
                    from: header.from.first().cloned(),
                    to,
                    cc,
                    subject: header.subject.clone(),
                    snippet: None,
                    flags: system_flags,
                    tags,
                    structure_json: None,
                });

                headers.push(header);

                if batch_items.len() >= DEFAULT_BATCH_SIZE {
                    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
                    let mut conn = pool.get().map_err(|e| e.to_string())?;
                    let new_uids = crate::db::batch_insert_messages(
                        &mut *conn,
                        account_id,
                        mailbox,
                        &batch_items,
                        DEFAULT_BATCH_SIZE,
                    )
                    .map_err(|e| e.to_string())?;

                    if let Err(e) = crate::db::filters::apply_rules_to_messages(
                        &mut *conn, account_id, mailbox, &new_uids,
                    ) {
                        tracing::warn!(target: "postail", "[Filters] Rule apply error for batch in {}: {}", mailbox, e);
                    }

                    batch_items.clear();
                }

                if headers.len() >= limit as usize {
                    break;
                }
            }
        }

        if !batch_items.is_empty() {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            let new_uids = crate::db::batch_insert_messages(
                &mut *conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;

            if let Err(e) = crate::db::filters::apply_rules_to_messages(
                &mut *conn, account_id, mailbox, &new_uids,
            ) {
                tracing::warn!(target: "postail", "[Filters] Rule apply error for batch in {}: {}", mailbox, e);
            }
        }

        // Process any flag/move operations queued by rule actions during pass 1
        let aid = account_id.to_string();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = crate::cmd::mail::actions::process_flag_queue(&aid).await {
                tracing::error!(target: "postail", "[Filters] Failed to process flag queue after rules: {}", e);
            }
        });

        // Pass 2 – fetch snippet bytes grouped by section path
        fetch_snippets_pass2(
            &mut session,
            &snippet_targets,
            &mut headers,
            account_id,
            mailbox,
        )
        .await;

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
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let conn = pool.get().map_err(|e| e.to_string())?;
            crate::db::fetch_headers(&*conn, account_id, mailbox, anchor, limit)
                .map_err(|e| e.to_string())?
        };

        if headers.len() < limit as usize {
            let needed = limit - headers.len() as u32;
            let next_anchor = headers.last().map(|h| h.uid).or(anchor);

            match self
                .fetch_history_from_imap(account_id, mailbox, next_anchor, needed)
                .await
            {
                Ok(mut new_headers) => {
                    if !new_headers.is_empty() {
                        tracing::info!(
                            target: "postail",
                            "[IMAP] Hydrated {} older messages for {}",
                            new_headers.len(),
                            mailbox
                        );
                        headers.append(&mut new_headers);
                    }
                }
                Err(e) => {
                    tracing::error!(target: "postail", "[IMAP] Failed to fetch history: {}", e);
                }
            }
        }

        headers.sort_by(|a, b| b.uid.cmp(&a.uid));
        headers.truncate(limit as usize);

        // Backfill snippets for DB messages that came back with snippet=NULL.
        let missing_uids: Vec<u32> = headers
            .iter()
            .filter(|h| h.snippet.is_none())
            .map(|h| h.uid)
            .collect();

        if !missing_uids.is_empty() {
            tracing::info!(
                target: "postail",
                "[Snippet] hybrid: backfilling {} messages with missing snippets in {}",
                missing_uids.len(),
                mailbox
            );

            if let Ok(mut session) = self.connect_imap(account_id).await {
                if session.select(mailbox).await.is_ok() {
                    let uid_list = missing_uids
                        .iter()
                        .map(|u| u.to_string())
                        .collect::<Vec<_>>()
                        .join(",");

                    let mut snippet_targets: Vec<SnippetTarget> = Vec::new();

                    match session.uid_fetch(&uid_list, "(UID BODYSTRUCTURE)").await {
                        Ok(mut fetches) => {
                            while let Some(fetch) = fetches.next().await {
                                let Ok(fetch) = fetch else { continue };
                                let Some(uid) = fetch.uid else { continue };
                                if let Some((path, mime, charset, encoding)) =
                                    fetch.bodystructure().and_then(|bs| find_text_part(bs, &[]))
                                {
                                    snippet_targets.push(SnippetTarget {
                                        uid,
                                        section: section_path_to_string(&path),
                                        mime,
                                        charset,
                                        encoding,
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                target: "postail",
                                "[Snippet] hybrid: BODYSTRUCTURE fetch failed: {}",
                                e
                            );
                        }
                    }

                    if !snippet_targets.is_empty() {
                        fetch_snippets_pass2(
                            &mut session,
                            &snippet_targets,
                            &mut headers,
                            account_id,
                            mailbox,
                        )
                        .await;
                    }
                }
                let _ = session.logout().await;
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
        let mut session = self.connect_imap(account_id).await?;
        let selected = session.select(mailbox).await.map_err(|e| e.to_string())?;
        let total = selected.exists;

        if total == 0 {
            let _ = session.logout().await;
            return Ok(Vec::new());
        }

        let range = if let Some(uid) = anchor_uid {
            let range_result: Result<Option<String>, String> = {
                let mut fetches = session
                    .uid_fetch(uid.to_string(), "UID")
                    .await
                    .map_err(|e| e.to_string())?;
                if let Some(fetch) = fetches.next().await {
                    let fetch = fetch.map_err(|e| e.to_string())?;
                    let seq = fetch.message;
                    if seq <= 1 {
                        Ok(None)
                    } else {
                        let end = seq - 1;
                        let start = end.saturating_sub(limit - 1).max(1);
                        Ok(Some(format!("{}:{}", start, end)))
                    }
                } else {
                    tracing::warn!(
                        target: "postail",
                        "[IMAP] Anchor UID {} not found on server", uid
                    );
                    Ok(None)
                }
            };

            match range_result? {
                Some(r) => r,
                None => {
                    let _ = session.logout().await;
                    return Ok(Vec::new());
                }
            }
        } else {
            let end = total;
            let start = total.saturating_sub(limit - 1).max(1);
            format!("{}:{}", start, end)
        };

        let mut batch_items: Vec<MessageBatchItem> = Vec::with_capacity(DEFAULT_BATCH_SIZE);
        let mut headers: Vec<MailHeader> = Vec::new();
        let mut snippet_targets: Vec<SnippetTarget> = Vec::new();

        // Pass 1 – headers + BODYSTRUCTURE
        {
            let mut fetches = session
                .fetch(range, "(UID INTERNALDATE FLAGS ENVELOPE BODYSTRUCTURE)")
                .await
                .map_err(|e| e.to_string())?;

            while let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let uid = fetch.uid.ok_or("No UID")?;
                let envelope = fetch.envelope().ok_or("No envelope")?;

                let subject = crate::utils::mail::decode_mime_header(envelope.subject.as_deref());
                let from = envelope
                    .from
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let to = envelope
                    .to
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let cc = envelope
                    .cc
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let all_flags = fetch
                    .flags()
                    .map(|f| flag_to_string(&f))
                    .collect::<Vec<_>>();

                let mut system_flags = Vec::new();
                let mut tags = Vec::new();
                for f in all_flags {
                    if f.starts_with('\\') {
                        system_flags.push(f);
                    } else {
                        tags.push(f);
                    }
                }

                let internal_date = fetch.internal_date().ok_or("No internal date")?;
                let internal_date = Utc
                    .timestamp_opt(internal_date.timestamp(), 0)
                    .single()
                    .unwrap_or_else(Utc::now);

                if let Some((path, mime, charset, encoding)) =
                    fetch.bodystructure().and_then(|bs| find_text_part(bs, &[]))
                {
                    snippet_targets.push(SnippetTarget {
                        uid,
                        section: section_path_to_string(&path),
                        mime,
                        charset,
                        encoding,
                    });
                }

                let header = MailHeader {
                    uid,
                    mailbox: mailbox.to_string(),
                    message_id: envelope
                        .message_id
                        .as_ref()
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                    internal_date,
                    subject: subject.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    cc: cc.clone(),
                    flags: system_flags.clone(),
                    snippet: None,
                    has_attachments: false,
                    starred: system_flags.iter().any(|f| f == "\\Flagged"),
                    tags: tags.clone(),
                };

                batch_items.push(MessageBatchItem {
                    uid,
                    message_id: header.message_id.clone(),
                    internal_date: header.internal_date,
                    from: header.from.first().cloned(),
                    to,
                    cc,
                    subject: header.subject.clone(),
                    snippet: None,
                    flags: system_flags,
                    tags,
                    structure_json: None,
                });

                headers.push(header);
            }
        }

        if !batch_items.is_empty() {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            let new_uids = crate::db::batch_insert_messages(
                &mut *conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;

            if let Err(e) = crate::db::filters::apply_rules_to_messages(
                &mut *conn, account_id, mailbox, &new_uids,
            ) {
                tracing::warn!(target: "postail", "[Filters] Rule apply error for batch in {}: {}", mailbox, e);
            }
        }

        let aid = account_id.to_string();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = crate::cmd::mail::actions::process_flag_queue(&aid).await {
                tracing::error!(target: "postail", "[Filters] Failed to process flag queue after rules (history): {}", e);
            }
        });

        // Pass 2 – snippet bytes
        fetch_snippets_pass2(
            &mut session,
            &snippet_targets,
            &mut headers,
            account_id,
            mailbox,
        )
        .await;

        let _ = session.logout().await;
        headers.sort_by(|a, b| b.uid.cmp(&a.uid));
        Ok(headers)
    }

    pub async fn fetch_uids_from_imap(
        &self,
        account_id: &str,
        mailbox: &str,
        uids: &[u32],
    ) -> Result<Vec<MailHeader>, String> {
        if uids.is_empty() {
            return Ok(Vec::new());
        }

        let mut session = self.connect_imap(account_id).await?;
        let _ = session.select(mailbox).await.map_err(|e| e.to_string())?;

        let mut batch_items: Vec<MessageBatchItem> = Vec::with_capacity(uids.len());
        let mut headers: Vec<MailHeader> = Vec::new();
        let mut snippet_targets: Vec<SnippetTarget> = Vec::new();

        for chunk in uids.chunks(500) {
            let uid_set = chunk
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let mut fetches = session
                .uid_fetch(&uid_set, "(UID INTERNALDATE FLAGS ENVELOPE BODYSTRUCTURE)")
                .await
                .map_err(|e| e.to_string())?;

            while let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                let uid = fetch.uid.ok_or("No UID")?;
                let envelope = fetch.envelope().ok_or("No envelope")?;

                let subject = crate::utils::mail::decode_mime_header(envelope.subject.as_deref());
                let from = envelope
                    .from
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let to = envelope
                    .to
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let cc = envelope
                    .cc
                    .as_deref()
                    .map(|a| parse_address_list!(a))
                    .unwrap_or_default();
                let all_flags = fetch
                    .flags()
                    .map(|f| flag_to_string(&f))
                    .collect::<Vec<_>>();

                let mut system_flags = Vec::new();
                let mut tags = Vec::new();
                for f in all_flags {
                    if f.starts_with('\\') {
                        system_flags.push(f);
                    } else {
                        tags.push(f);
                    }
                }

                let internal_date = fetch.internal_date().ok_or("No internal date")?;
                let internal_date = Utc
                    .timestamp_opt(internal_date.timestamp(), 0)
                    .single()
                    .unwrap_or_else(Utc::now);

                if let Some((path, mime, charset, encoding)) =
                    fetch.bodystructure().and_then(|bs| find_text_part(bs, &[]))
                {
                    snippet_targets.push(SnippetTarget {
                        uid,
                        section: section_path_to_string(&path),
                        mime,
                        charset,
                        encoding,
                    });
                }

                let header = MailHeader {
                    uid,
                    mailbox: mailbox.to_string(),
                    message_id: envelope
                        .message_id
                        .as_ref()
                        .map(|s| String::from_utf8_lossy(s).to_string()),
                    internal_date,
                    subject: subject.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    cc: cc.clone(),
                    flags: system_flags.clone(),
                    snippet: None,
                    has_attachments: false,
                    starred: system_flags.iter().any(|f| f == "\\Flagged"),
                    tags: tags.clone(),
                };

                batch_items.push(MessageBatchItem {
                    uid,
                    message_id: header.message_id.clone(),
                    internal_date: header.internal_date,
                    from: header.from.first().cloned(),
                    to,
                    cc,
                    subject: header.subject.clone(),
                    snippet: None,
                    flags: system_flags,
                    tags,
                    structure_json: None,
                });

                headers.push(header);
            }
        }

        if !batch_items.is_empty() {
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            let new_uids = crate::db::batch_insert_messages(
                &mut *conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;

            if let Err(e) = crate::db::filters::apply_rules_to_messages(
                &mut *conn, account_id, mailbox, &new_uids,
            ) {
                tracing::warn!(target: "postail", "[Filters] Rule apply error for batch in {}: {}", mailbox, e);
            }
        }

        let aid = account_id.to_string();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = crate::cmd::mail::actions::process_flag_queue(&aid).await {
                tracing::error!(target: "postail", "[Filters] Failed to process flag queue after rules (fetch_uids): {}", e);
            }
        });

        // Pass 2 – snippet bytes
        fetch_snippets_pass2(
            &mut session,
            &snippet_targets,
            &mut headers,
            account_id,
            mailbox,
        )
        .await;

        let _ = session.logout().await;
        headers.sort_by(|a, b| b.uid.cmp(&a.uid));
        Ok(headers)
    }
}
