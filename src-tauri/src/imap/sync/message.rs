use futures::StreamExt;
use rusqlite::params;

use crate::db;

impl crate::imap::ImapManager {
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
            tracing::info!(target: "postail", "[IMAP] fetch_message_full: calling uid_fetch for {}", uid);
            let mut fetches = session
                .uid_fetch(format!("{}", uid), "(BODY[])")
                .await
                .map_err(|e| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: uid_fetch failed: {}", e);
                    e.to_string()
                })?;

            if let Some(fetch) = fetches.next().await {
                let fetch = fetch.map_err(|e| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: fetch.next() failed: {}", e);
                    e.to_string()
                })?;
                let body_owned: Vec<u8> = fetch.body().ok_or_else(|| {
                    tracing::error!(target: "postail", "[IMAP] fetch_message_full: No body in fetch result for uid={}", uid);
                    "No body".to_string()
                })?.to_vec(); // owned

                tracing::info!(target: "postail", "[IMAP] fetch_message_full: got {} bytes for uid={}", body_owned.len(), uid);

                drop(fetch);
                drop(fetches);

                let conn_guard = self.conn.lock().await;
                let conn = conn_guard
                    .as_ref()
                    .ok_or_else(|| "Database not initialized".to_string())?;

                // Fetch the header from the 'messages' table (read-only)
                let header = {
                    let mut stmt = conn.prepare(
                        "SELECT message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments 
                         FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?"
                    ).map_err(|e| e.to_string())?;

                    stmt.query_row(params![account_id, mailbox, uid], |row| {
                        let to_json: Option<String> = row.get(4)?;
                        let to: Vec<String> = to_json
                            .map(|s| serde_json::from_str(&s).unwrap_or_default())
                            .unwrap_or_default();
                        let cc_json: Option<String> = row.get(5)?;
                        let cc: Vec<String> = cc_json
                            .map(|s| serde_json::from_str(&s).unwrap_or_default())
                            .unwrap_or_default();
                        let flags_json: Option<String> = row.get(6)?;
                        let flags: Vec<String> = flags_json
                            .map(|s| serde_json::from_str(&s).unwrap_or_default())
                            .unwrap_or_default();

                        Ok(db::MailHeader {
                            uid,
                            message_id: row.get(0)?,
                            internal_date: db::messages::safe_timestamp_from_utc(
                                row.get::<_, i64>(1)?,
                            )
                            .ok_or_else(|| {
                                rusqlite::Error::InvalidColumnName("internal_date".into())
                            })?,
                            subject: row.get(2)?,
                            from: vec![row.get::<_, Option<String>>(3)?.unwrap_or_default()],
                            to,
                            cc,
                            flags,
                            snippet: row.get(7)?,
                            has_attachments: row.get::<_, i64>(8)? != 0,
                        })
                    })
                    .map_err(|e| e.to_string())?
                };

                // Parse the body in-memory
                let parts = db::message_bodies::parse_mail_with_fallback(&body_owned);
                let attachments = parts.attachments;
                let inline_images = parts.inline_images;
                let html = parts.html_content;
                let plain = parts.plain_content;

                let mut header = header;
                if !attachments.is_empty() || !inline_images.is_empty() {
                    header.has_attachments = true;
                }

                tracing::info!(target: "postail", "[IMAP] fetch_message_full: returning direct MessageFull for uid={} (no cache, no sanitizer)", uid);

                Ok(Some(db::MessageFull {
                    header,
                    body_html_safe: html.unwrap_or_else(|| plain.clone().unwrap_or_default()),
                    body_plain: plain.unwrap_or_default(),
                    attachments,
                    inline_images,
                }))
            } else {
                tracing::warn!(target: "postail", "[IMAP] fetch_message_full: No fetch results for uid={}", uid);
                Ok(None)
            }
        };

        let _ = session.logout().await;
        fetch_result
    }
}
