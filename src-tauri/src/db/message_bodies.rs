use crate::error::DBError;
use ammonia;
use mailparse::{parse_mail, MailPart};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::panic;

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

pub fn parse_mail_with_fallback(raw_eml: &[u8]) -> (Option<String>, Option<String>, Option<String>) {
    let parse_result = panic::catch_unwind(|| parse_mail(raw_eml));

    match parse_result {
        Ok(mail) => {
            let mut html_content = None;
            let mut plain_content = None;

            fn extract_text_parts(part: &MailPart, html: &mut Option<String>, plain: &mut Option<String>) {
                let mime_type = part.ctype.mimetype.as_str();
                
                if mime_type == "text/html" {
                    if let Ok(content) = std::str::from_utf8(&part.body.raw) {
                        if html.is_none() {
                            *html = Some(content.to_string());
                        }
                    }
                } else if mime_type == "text/plain" {
                    if let Ok(content) = std::str::from_utf8(&part.body.raw) {
                        if plain.is_none() {
                            *plain = Some(content.to_string());
                        }
                    }
                }

                if part.ctype.mimetype.starts_with("multipart/") {
                    if let Some(ref subparts) = part.body.subparts {
                        for sp in subparts {
                            extract_text_parts(sp, html, plain);
                        }
                    }
                }
            }

            extract_text_parts(&mail, &mut html_content, &mut plain_content);

            (html_content, plain_content, None)
        }
        Err(_) => {
            let raw_str = String::from_utf8_lossy(raw_eml);
            let preview = raw_str.chars().take(500).collect::<String>();
            (None, None, Some(preview))
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
    let (html, plain, error_preview) = parse_mail_with_fallback(raw_eml);

    let html_cleaned = html.as_deref().map(ammonia::clean);
    let body_html = html_cleaned.as_deref().unwrap_or("");
    let body_plain = plain.as_deref().unwrap_or("");
    let snippet = error_preview.clone()
        .or_else(|| Some(body_text_for_snippet(body_plain)))
        .unwrap_or_default();

    conn.execute(
        "UPDATE messages SET snippet = ? WHERE id = ?",
        params![snippet, message_table_id],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO message_bodies (message_id, body_html_safe, body_plain, raw_content, parse_error)
         VALUES (?, ?, ?, ?, ?)",
        params![
            message_table_id,
            body_html,
            body_plain,
            raw_eml,
            error_preview.as_deref().unwrap_or("")
        ],
    )?;

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
) -> Result<(Option<String>, String, Option<Vec<u8>>, Option<String>), DBError> {
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
