use futures::StreamExt;
use mailparse::MailHeaderMap;

use crate::db;
use crate::db::eml_cache;
use crate::db::messages::sync_message_attachments_flag;

impl crate::imap::ImapManager {
    /// Cache-first sync read: returns full message from DB if body is already parsed.
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

    pub async fn fetch_and_cache_message(
        &self,
        account_id: &str,
        mailbox: &str,
        uid: u32,
    ) -> Result<Option<crate::db::MessageFull>, String> {
        // Step 1: try disk cache first
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
                // Step 2: fetch from IMAP
                tracing::info!(
                    target: "postail",
                    "[EmlCache] Disk cache MISS uid={} mailbox={} — fetching from IMAP", uid, mailbox
                );
                let bytes = self
                    .fetch_raw_eml_from_imap(account_id, mailbox, uid)
                    .await?;

                // Encrypt and save to disk
                let security = self.security.lock().await;
                eml_cache::save_eml(&security, account_id, mailbox, uid, &bytes)
                    .map_err(|e| e.to_string())?;

                bytes
            }
        };

        // Get message table id
        let message_table_id = {
            let conn_guard = self.conn.lock().await;
            let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
            db::get_message_table_id(conn, account_id, mailbox, uid).map_err(|e| e.to_string())?
        };

        let message_table_id = match message_table_id {
            Some(id) => id,
            None => return Err("Message header not found in database".to_string()),
        };

        // Parse body
        let (html, plain, parse_error) = db::parse_mail_with_fallback(&raw_eml);

        {
            let conn_guard = self.conn.lock().await;
            let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

            // Save parsed body to DB (no raw_content BLOB)
            db::save_message_body(conn, message_table_id, html.as_deref(), plain.as_deref())
                .map_err(|e| e.to_string())?;

            if let Some(err) = &parse_error {
                tracing::warn!(
                    target: "postail",
                    "[EmlCache] Parse error uid={}: {}", uid, err
                );
            }

            // Parse and save attachments
            save_attachments_from_eml(conn, message_table_id, &raw_eml);

            // Return full message from DB
            db::fetch_message_full(conn, account_id, mailbox, uid).map_err(|e| e.to_string())
        }
    }

    async fn fetch_raw_eml_from_imap(
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

/// Parses raw EML and saves attachments + inline images to DB.
fn save_attachments_from_eml(conn: &rusqlite::Connection, message_table_id: i64, raw_eml: &[u8]) {
    use mailparse::{parse_mail, ParsedMail};

    let Ok(mail) = parse_mail(raw_eml) else {
        return;
    };

    fn walk_parts(
        conn: &rusqlite::Connection,
        message_table_id: i64,
        part: &ParsedMail,
        part_counter: &mut u32,
    ) {
        let mime_type = part.ctype.mimetype.to_lowercase();
        let is_text = mime_type == "text/html" || mime_type == "text/plain";
        let is_multipart = mime_type.starts_with("multipart/");

        if is_multipart {
            for sub in &part.subparts {
                walk_parts(conn, message_table_id, sub, part_counter);
            }
            return;
        }

        if is_text {
            *part_counter += 1;
            return;
        }

        let disposition = part
            .get_headers()
            .get_first_value("Content-Disposition")
            .unwrap_or_default()
            .to_lowercase();
        let is_inline = disposition.starts_with("inline");

        let content_id = part
            .get_headers()
            .get_first_value("Content-ID")
            .map(|cid| cid.trim_matches(|c| c == '<' || c == '>').to_string());

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
        let part_id = format!("{}", *part_counter);
        *part_counter += 1;

        let _ = conn.execute(
            "INSERT OR IGNORE INTO attachments (message_table_id, part_id, filename, mime_type, size, is_inline, cid)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                message_table_id,
                part_id,
                filename,
                mime_type,
                size,
                if is_inline { 1i64 } else { 0i64 },
                content_id,
            ],
        );
    }

    let mut counter = 0u32;
    walk_parts(conn, message_table_id, &mail, &mut counter);
    let _ = sync_message_attachments_flag(message_table_id, conn);
}
