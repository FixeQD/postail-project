use futures::StreamExt;

use crate::db;
use crate::db::eml_cache::{self, CachedBody};
use crate::db::messages::sync_message_attachments_flag;
use crate::security::SecurityManager;

impl crate::imap::ImapManager {
    /// Returns the full message from DB header + file-based body cache.
    pub async fn fetch_message_full_sync(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        let conn_guard = self.conn.lock().await;
        let conn = conn_guard
            .as_ref()
            .ok_or("Database not initialized".to_string())?;
        db::fetch_message_full(conn, account_id, mailbox, uid).map_err(|e| e.to_string())
    }

    /// Fetches EML (or reuses disk cache), parses body + attachments, saves everything.
    pub async fn fetch_and_cache_message(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        // ── Step 1: load or fetch raw EML ──────────────────────────────────
        let raw_eml = {
            let security = self.security.lock().await;
            eml_cache::load_eml(&security, account_id, mailbox, uid).map_err(|e| e.to_string())?
        };

        let raw_eml = match raw_eml {
            Some(bytes) => {
                tracing::info!(
                    target: "postail",
                    "[EmlCache] Disk cache HIT uid={} mailbox={}", uid, mailbox
                );
                bytes
            }
            None => {
                tracing::info!(
                    target: "postail",
                    "[EmlCache] Disk cache MISS uid={} mailbox={} — fetching from IMAP", uid, mailbox
                );
                let bytes = self.fetch_raw_eml_bytes(account_id, mailbox, uid).await?;

                // Encrypt and save to disk
                let security = self.security.lock().await;
                eml_cache::save_eml(&security, account_id, mailbox, uid, &bytes)
                    .map_err(|e| e.to_string())?;
                bytes
            }
        };

        // ── Step 2: parse body + receipt header ────────────────────────────
        let (html, plain, parse_error) = db::parse_mail_with_fallback(&raw_eml);

        let read_receipt_to = {
            use mailparse::{parse_mail, MailHeaderMap};
            parse_mail(&raw_eml).ok().and_then(|m| {
                m.headers
                    .get_first_value("Disposition-Notification-To")
                    .or_else(|| m.headers.get_first_value("Return-Receipt-To"))
            })
        };

        if let Some(ref err) = parse_error {
            tracing::warn!(target: "postail", "[BodyCache] Parse error uid={}: {}", uid, err);
        }

        tracing::info!(
            target: "postail",
            "[BodyCache] Parsed EML uid={} mailbox={}: html_len={} plain_len={} parse_error={:?}",
            uid, mailbox,
            html.as_deref().map(|s| s.len()).unwrap_or(0),
            plain.as_deref().map(|s| s.len()).unwrap_or(0),
            parse_error
        );

        let body_html = html.unwrap_or_default();
        let body_plain = plain.unwrap_or_default();

        // ── Step 4: save body to encrypted file ─────────────────────────────
        {
            let security = self.security.lock().await;
            let cached_body = CachedBody {
                body_html: body_html.clone(),
                body_plain: body_plain.clone(),
                read_receipt_to: read_receipt_to.clone(),
            };
            eml_cache::save_body(&security, account_id, mailbox, uid, &cached_body)
                .map_err(|e| e.to_string())?;
        }

        // ── Step 4b: extract and cache inline images ─────────────────────────
        {
            let security = self.security.lock().await;
            let conn_guard = self.conn.lock().await;
            if let Some(conn) = conn_guard.as_ref() {
                if let Ok(Some(table_id)) = db::get_message_table_id(conn, account_id, mailbox, uid)
                {
                    save_attachments_from_eml(
                        conn, &security, account_id, mailbox, uid, table_id, &raw_eml,
                    );
                }
            }
        }

        // ── Step 5: return full assembled message ──────────────────────────
        let conn_guard = self.conn.lock().await;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let mut msg =
            db::fetch_message_full(conn, account_id, mailbox, uid).map_err(|e| e.to_string())?;

        // Inject body + receipt header from file cache
        if let Some(ref mut m) = msg {
            m.body_html_safe = body_html;
            m.body_plain = body_plain;
            m.read_receipt_to = read_receipt_to;
        }

        tracing::info!(
            target: "postail",
            "[BodyCache] Final result uid={}: found={} html_len={} plain_len={}",
            uid,
            msg.is_some(),
            msg.as_ref().map(|m| m.body_html_safe.len()).unwrap_or(0),
            msg.as_ref().map(|m| m.body_plain.len()).unwrap_or(0),
        );

        Ok(msg)
    }

    pub async fn fetch_raw_eml_bytes(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Vec<u8>, String> {
        let mut session = self.connect_imap(account_id).await?;
        session.select(mailbox).await.map_err(|e| e.to_string())?;

        let raw: Option<Vec<u8>> = {
            let mut fetches = session
                .uid_fetch(format!("{}", uid), "(BODY.PEEK[])")
                .await
                .map_err(|e| e.to_string())?;
            if let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| e.to_string())?;
                fetch.body().map(|b| b.to_vec())
            } else {
                None
            }
        };

        session.logout().await.map_err(|e| e.to_string())?;
        raw.ok_or_else(|| format!("No body returned from IMAP for uid={}", uid))
    }
}

/// Parses raw EML and saves attachment metadata (small rows, no BLOBs) to DB.

fn save_attachments_from_eml(
    conn: &rusqlite::Connection,
    security: &SecurityManager,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    message_table_id: i64,
    raw_eml: &[u8],
) {
    use mailparse::MailHeaderMap;
    use mailparse::{parse_mail, ParsedMail};

    let Ok(mail) = parse_mail(raw_eml) else {
        return;
    };

    fn walk_parts(
        conn: &rusqlite::Connection,
        security: &SecurityManager,
        account_id: &str,
        mailbox: &str,
        uid: u32,
        message_table_id: i64,
        part: &ParsedMail,
        counter: &mut u32,
    ) {
        let mime_type = part.ctype.mimetype.to_lowercase();
        if mime_type.starts_with("multipart/") {
            for sub in &part.subparts {
                walk_parts(
                    conn,
                    security,
                    account_id,
                    mailbox,
                    uid,
                    message_table_id,
                    sub,
                    counter,
                );
            }
            return;
        }
        if mime_type == "text/html" || mime_type == "text/plain" {
            *counter += 1;
            return;
        }

        let disposition = part
            .get_headers()
            .get_first_value("Content-Disposition")
            .unwrap_or_default()
            .to_lowercase();
        let is_inline = disposition.starts_with("inline");

        let cid = part
            .get_headers()
            .get_first_value("Content-ID")
            .map(|s| s.trim_matches(|c| c == '<' || c == '>').to_string());

        let filename = part
            .get_headers()
            .get_first_value("Content-Disposition")
            .and_then(|d| {
                d.split(';')
                    .find(|p| p.trim().starts_with("filename"))
                    .and_then(|p| p.split('=').nth(1))
                    .map(|v| v.trim().trim_matches('"').to_string())
            })
            .or_else(|| {
                part.get_headers()
                    .get_first_value("Content-Type")
                    .and_then(|ct| {
                        ct.split(';')
                            .find(|p| p.trim().starts_with("name"))
                            .and_then(|p| p.split('=').nth(1))
                            .map(|v| v.trim().trim_matches('"').to_string())
                    })
            });

        let size = part.get_body_raw().map(|b| b.len() as i64).unwrap_or(0);
        let part_id = format!("{}", *counter);
        *counter += 1;

        // For inline images with a CID, extract binary and cache to disk.
        let cached_path: Option<String> = if is_inline && cid.is_some() {
            part.get_body_raw().ok().and_then(|raw| {
                eml_cache::save_inline_image(security, account_id, mailbox, uid, &part_id, &raw)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            })
        } else {
            None
        };

        let _ = conn.execute(
            "INSERT OR IGNORE INTO attachments
             (message_table_id, part_id, filename, mime_type, size, is_inline, cid, cached_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                message_table_id,
                part_id,
                filename,
                mime_type,
                size,
                if is_inline { 1i64 } else { 0i64 },
                cid,
                cached_path,
            ],
        );
    }

    let mut counter = 0u32;
    walk_parts(
        conn,
        security,
        account_id,
        mailbox,
        uid,
        message_table_id,
        &mail,
        &mut counter,
    );
    let _ = sync_message_attachments_flag(message_table_id, conn);
}
