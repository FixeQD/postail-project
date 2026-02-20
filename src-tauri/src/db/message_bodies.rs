use crate::error::DBError;
use ammonia;
use mailparse::MailHeaderMap;
use mailparse::{parse_mail, ParsedMail};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::panic;
use std::fs;

type MessageBodyFull = (Option<String>, String, Option<Vec<u8>>, Option<String>);

pub fn create_message_bodies_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_bodies (
            message_id INTEGER PRIMARY KEY,
            body_html_safe TEXT,
            body_plain TEXT,
            raw_content BLOB,
            parse_error TEXT,
            FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_bodies_message_id 
         ON message_bodies(message_id)",
        [],
    )?;

    Ok(())
}

pub fn parse_mail_with_fallback(
    raw_eml: &[u8],
) -> (Option<String>, Option<String>, Vec<crate::db::AttachmentMeta>, Vec<crate::db::AttachmentMeta>, Option<String>) {
    let parse_result = panic::catch_unwind(|| parse_mail(raw_eml));

    match parse_result {
        Ok(Ok(mail)) => {
            let mut html_content = None;
            let mut plain_content = None;
            let mut attachments = Vec::new();
            let mut inline_images = Vec::new();
            let mut part_counter = 0;

            fn extract_parts(
                part: &ParsedMail,
                html: &mut Option<String>,
                plain: &mut Option<String>,
                attachments: &mut Vec<crate::db::AttachmentMeta>,
                inline_images: &mut Vec<crate::db::AttachmentMeta>,
                part_counter: &mut usize,
            ) {
                let mime_type = part.ctype.mimetype.as_str();
                let current_part_id = part_counter.to_string();
                *part_counter += 1;

                // Check if it's an attachment
                let disp = part.get_content_disposition();
                let filename = disp.params.get("filename")
                    .cloned()
                    .or_else(|| part.ctype.params.get("name").cloned());

                let is_attachment = disp.disposition == mailparse::DispositionType::Attachment;
                let is_inline = disp.disposition == mailparse::DispositionType::Inline;

                // Treat as attachment if explicitly marked or if it has a filename and isn't plain text
                if !mime_type.starts_with("multipart/") && (is_attachment || (filename.is_some() && !mime_type.starts_with("text/")) || (is_inline && !mime_type.starts_with("text/"))) {
                    let bytes = part.get_body_raw().unwrap_or_default();
                    if filename.is_none() && bytes.is_empty() {
                        // Skip it
                    } else {
                        let cid = part.headers.get_first_value("Content-ID");
                        let mut cached_path = None;

                        // If it's an inline image, we MUST cache it for immediate display
                        if is_inline && mime_type.starts_with("image/") {
                            if let Ok(dir) = crate::db::attachments::get_attachments_dir() {
                                let id = uuid::Uuid::new_v4().to_string();
                                let target_path = dir.join(&id);
                                if fs::write(&target_path, &bytes).is_ok() {
                                    cached_path = Some(target_path.to_string_lossy().to_string());
                                }
                            }
                        }

                        let meta = crate::db::AttachmentMeta {
                            part_id: current_part_id,
                            filename,
                            mime_type: mime_type.to_string(),
                            size: bytes.len() as u64,
                            cid,
                            cached_path,
                        };

                        if is_inline && meta.cid.is_some() {
                            inline_images.push(meta);
                        } else {
                            attachments.push(meta);
                        }
                    }
                } else if mime_type == "text/html" {
                    if let Ok(content) = part.get_body() {
                        if html.is_none() {
                            *html = Some(content);
                        }
                    }
                } else if mime_type == "text/plain" {
                    if let Ok(content) = part.get_body() {
                        if plain.is_none() {
                            *plain = Some(content);
                        }
                    }
                }

                if mime_type.starts_with("multipart/") {
                    for sp in &part.subparts {
                        extract_parts(sp, html, plain, attachments, inline_images, part_counter);
                    }
                }
            }

            extract_parts(&mail, &mut html_content, &mut plain_content, &mut attachments, &mut inline_images, &mut part_counter);

            (html_content, plain_content, attachments, inline_images, None)
        }
        Ok(Err(_)) | Err(_) => {
            let raw_str = String::from_utf8_lossy(raw_eml);
            let preview = raw_str.chars().take(500).collect::<String>();
            (None, None, vec![], vec![], Some(preview))
        }
    }
}

pub fn save_message_body(
    conn: &Connection,
    message_table_id: i64,
    body_html: Option<&str>,
    body_plain: Option<&str>,
) -> Result<(), DBError> {
    let body_html_safe = body_html.map(ammonia::clean).unwrap_or_default();

    let body_text = body_plain.unwrap_or_else(|| body_html.unwrap_or(""));
    let snippet = body_text.chars().take(200).collect::<String>();

    conn.execute(
        "UPDATE messages SET snippet = ? WHERE id = ?",
        params![snippet, message_table_id],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO message_bodies (message_id, body_html_safe, body_plain)
         VALUES (?, ?, ?)",
        params![message_table_id, body_html_safe, body_plain.unwrap_or("")],
    )?;

    Ok(())
}

pub fn save_message_body_with_fallback(
    conn: &Connection,
    message_table_id: i64,
    raw_eml: &[u8],
) -> Result<(), DBError> {
    let (html, plain, attachments, inline_images, error_preview) = parse_mail_with_fallback(raw_eml);

    let html_cleaned = html.as_deref().map(ammonia::clean);
    let body_html = html_cleaned.as_deref().unwrap_or("");
    let body_plain = plain.as_deref().unwrap_or("");
    let snippet = error_preview
        .clone()
        .or_else(|| Some(body_text_for_snippet(body_plain)))
        .unwrap_or_default();

    conn.execute(
        "UPDATE messages SET snippet = ? WHERE id = ?",
        params![snippet, message_table_id],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO message_bodies 
         (message_id, body_html_safe, body_plain, parse_error)
         VALUES (?, ?, ?, ?)",
        params![message_table_id, body_html, body_plain, error_preview.as_deref().unwrap_or("")],
    )?;

    // Save attachments and inline images
    let all_attachments = attachments.into_iter().chain(inline_images.into_iter());
    
    let mut some_attachments = false;
    for att in all_attachments {
        some_attachments = true;
        conn.execute(
            "INSERT OR REPLACE INTO attachments (message_table_id, part_id, filename, mime_type, size, cid, cached_path)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                message_table_id,
                att.part_id,
                att.filename,
                att.mime_type,
                att.size as i64,
                att.cid,
                att.cached_path,
            ],
        )?;
    }

    if some_attachments {
        // Update has_attachments flag
        conn.execute(
            "UPDATE messages SET has_attachments = 1 WHERE id = ?",
            params![message_table_id],
        )?;
    }

    Ok(())
}

fn body_text_for_snippet(text: &str) -> String {
    text.lines()
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}

pub fn load_message_body(
    conn: &Connection,
    message_table_id: i64,
) -> Result<(Option<String>, String), DBError> {
    conn.query_row(
        "SELECT body_html_safe, body_plain FROM message_bodies WHERE message_id = ?",
        params![message_table_id],
        |row| {
            let html: Option<String> = row.get(0)?;
            let plain: String = row.get(1)?;
            Ok((html, plain))
        },
    )
    .optional()?
    .ok_or_else(|| DBError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

pub fn load_message_body_full(
    conn: &Connection,
    message_table_id: i64,
) -> Result<MessageBodyFull, DBError> {
    conn.query_row(
        "SELECT body_html_safe, body_plain, raw_content, parse_error FROM message_bodies WHERE message_id = ?",
        params![message_table_id],
        |row| {
            let html: Option<String> = row.get(0)?;
            let plain: String = row.get(1)?;
            let raw: Option<Vec<u8>> = row.get(2)?;
            let error: Option<String> = row.get(3)?;
            Ok((html, plain, raw, error))
        },
    )
    .optional()?
    .ok_or_else(|| DBError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}
