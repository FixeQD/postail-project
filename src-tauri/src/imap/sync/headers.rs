use chrono::{TimeZone, Utc};
use futures::StreamExt;

use crate::db::{DEFAULT_BATCH_SIZE, MailHeader, MessageBatchItem};
use crate::globals::get_db_pool;

fn flag_to_string(flag: &async_imap::types::Flag) -> String {
    match flag {
        async_imap::types::Flag::Seen => "\\Seen".to_string(),
        async_imap::types::Flag::Answered => "\\Answered".to_string(),
        async_imap::types::Flag::Flagged => "\\Flagged".to_string(),
        async_imap::types::Flag::Deleted => "\\Deleted".to_string(),
        async_imap::types::Flag::Draft => "\\Draft".to_string(),
        async_imap::types::Flag::Recent => "\\Recent".to_string(),
        async_imap::types::Flag::MayCreate => "\\MayCreate".to_string(),
        async_imap::types::Flag::Custom(s) => s.to_string(),
    }
}

macro_rules! parse_address_list {
    ($addrs:expr) => {
        $addrs
            .iter()
            .map(|a| {
                let mailbox = a
                    .mailbox
                    .as_ref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .unwrap_or_default();
                let host = a
                    .host
                    .as_ref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .unwrap_or_default();
                let email = format!("{}@{}", mailbox, host);
                let name = a
                    .name
                    .as_ref()
                    .and_then(|b| crate::utils::mail::decode_mime_header(Some(b.as_ref())))
                    .filter(|n| !n.is_empty());
                match name {
                    Some(n) => format!("{} <{}>", n, email),
                    None => email,
                }
            })
            .collect::<Vec<String>>()
    };
}

/// Remove horizontal rule lines, quoted lines, and collapse whitespace.
fn clean_snippet(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}

/// Decode raw MIME part bytes into a plain-text snippet.
/// Handles CTE (base64/QP/raw) and charset conversion directly
fn decode_part_preview(raw: &[u8], mime_type: &str, charset: &str, encoding: &str) -> String {
    use base64::Engine;

    let decoded_bytes: Vec<u8> = match encoding.to_ascii_lowercase().as_str() {
        "base64" => {
            let stripped: Vec<u8> = raw
                .iter()
                .copied()
                .filter(|&b| b != b'\r' && b != b'\n')
                .collect();
            base64::engine::general_purpose::STANDARD
                .decode(&stripped)
                .unwrap_or_else(|_| raw.to_vec())
        }
        "quoted-printable" => {
            let wrapped = [
                b"Content-Transfer-Encoding: quoted-printable\r\n\r\n" as &[u8],
                raw,
            ]
            .concat();
            mailparse::parse_mail(&wrapped)
                .ok()
                .and_then(|m| m.get_body_raw().ok())
                .unwrap_or_else(|| raw.to_vec())
        }
        _ => raw.to_vec(),
    };

    let text = {
        let enc =
            encoding_rs::Encoding::for_label(charset.as_bytes()).unwrap_or(encoding_rs::UTF_8);
        let (cow, _, _) = enc.decode(&decoded_bytes);
        cow.into_owned()
    };

    let plain = if mime_type.contains("html") {
        use kuchikiki::traits::TendrilSink;
        kuchikiki::parse_html()
            .one(text.as_str())
            .document_node
            .text_contents()
    } else {
        text
    };

    clean_snippet(&plain)
}

/// Walk BODYSTRUCTURE depth-first to find the first text/plain or text/html part. Returns (section_path_nums, mime, charset, encoding).
fn find_text_part(
    bs: &imap_proto::types::BodyStructure,
    path: &[u32],
) -> Option<(Vec<u32>, String, String, String)> {
    use imap_proto::types::{BodyStructure, ContentEncoding};
    match bs {
        BodyStructure::Text { common, other, .. } => {
            let mime = format!(
                "{}/{}",
                common.ty.ty.to_ascii_lowercase(),
                common.ty.subtype.to_ascii_lowercase()
            );
            if mime == "text/plain" || mime == "text/html" {
                let charset = common
                    .ty
                    .params
                    .as_ref()
                    .and_then(|p| {
                        p.iter()
                            .find(|(k, _)| k.to_ascii_lowercase() == "charset")
                            .map(|(_, v)| v.to_string())
                    })
                    .unwrap_or_else(|| "utf-8".to_string());
                let encoding = match &other.transfer_encoding {
                    ContentEncoding::Base64 => "base64".to_string(),
                    ContentEncoding::QuotedPrintable => "quoted-printable".to_string(),
                    ContentEncoding::SevenBit => "7bit".to_string(),
                    ContentEncoding::EightBit => "8bit".to_string(),
                    ContentEncoding::Binary => "binary".to_string(),
                    ContentEncoding::Other(s) => s.to_string(),
                };
                return Some((path.to_vec(), mime, charset, encoding));
            }
            None
        }
        BodyStructure::Multipart { bodies, .. } => {
            let mut html_fallback: Option<(Vec<u32>, String, String, String)> = None;
            for (i, sub) in bodies.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(i as u32 + 1);
                if let Some(result) = find_text_part(sub, &child_path) {
                    if result.1 == "text/plain" {
                        return Some(result);
                    }
                    if html_fallback.is_none() {
                        html_fallback = Some(result);
                    }
                }
            }
            html_fallback
        }
        _ => None,
    }
}

fn section_path_to_string(path: &[u32]) -> String {
    if path.is_empty() {
        "1".to_string()
    } else {
        path.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

struct SnippetTarget {
    uid: u32,
    section: String,
    mime: String,
    charset: String,
    encoding: String,
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
                    crate::db::batch_insert_messages(
                        &mut *conn,
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
            let pool = get_db_pool().await.map_err(|e| e.to_string())?;
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            crate::db::batch_insert_messages(
                &mut *conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;
        }

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
            crate::db::batch_insert_messages(
                &mut *conn,
                account_id,
                mailbox,
                &batch_items,
                DEFAULT_BATCH_SIZE,
            )
            .map_err(|e| e.to_string())?;
        }

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

/// Second pass: for each unique section path, batch-fetch snippet bytes an patch the in-memory headers + DB rows.
async fn fetch_snippets_pass2(
    session: &mut crate::imap::connection::ImapSession,
    targets: &[SnippetTarget],
    headers: &mut Vec<MailHeader>,
    account_id: &str,
    mailbox: &str,
) {
    use std::collections::HashMap;

    if targets.is_empty() {
        return;
    }

    let pool = match get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(target: "postail", "[Snippet] Failed to get DB pool: {}", e);
            return;
        }
    };

    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(target: "postail", "[Snippet] Failed to get DB connection: {}", e);
            return;
        }
    };

    // Group targets by section string so one uid_fetch covers all messages
    let mut by_section: HashMap<&str, Vec<&SnippetTarget>> = HashMap::new();
    for t in targets {
        by_section.entry(t.section.as_str()).or_default().push(t);
    }

    for (section_str, group) in &by_section {
        let uid_list = group
            .iter()
            .map(|t| t.uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!("(UID BODY.PEEK[{}]<0.512>)", section_str);
        let mut fetches = match session.uid_fetch(&uid_list, &query).await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(target: "postail", "[Snippet] uid_fetch failed section={} err={}", section_str, e);
                continue;
            }
        };

        while let Some(fetch) = fetches.next().await {
            let Ok(fetch) = fetch else { continue };
            let Some(uid) = fetch.uid else { continue };

            let Some(target) = group.iter().find(|t| t.uid == uid) else {
                continue;
            };

            let path_nums: Vec<u32> = section_str
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();

            use imap_proto::types::SectionPath;
            let section_path = SectionPath::Part(path_nums, None);

            let snippet = fetch
                .section(&section_path)
                .filter(|b| !b.is_empty())
                .map(|b| decode_part_preview(b, &target.mime, &target.charset, &target.encoding))
                .filter(|s| !s.is_empty());

            if let Some(ref s) = snippet {
                if let Some(h) = headers.iter_mut().find(|h| h.uid == uid) {
                    h.snippet = Some(s.clone());
                }
                let _ = conn.execute(
                    "UPDATE messages SET snippet = ? \
                     WHERE account_id = ? AND mailbox = ? AND uid = ? \
                     AND (snippet IS NULL OR snippet = '')",
                    rusqlite::params![s, account_id, mailbox, uid],
                );
            }
        }
    }
}
