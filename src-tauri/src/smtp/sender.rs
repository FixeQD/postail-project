use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde_json;

impl super::SmtpManager {
    pub(crate) fn get_credentials(&self, account_id: &str) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT creds_blob_path FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let creds_path: String = stmt
            .query_row([account_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        drop(stmt);
        drop(conn);

        let security = self.security.lock().unwrap();
        let encrypted = std::fs::read(&creds_path).map_err(|e| e.to_string())?;
        let decrypted = security.decrypt(&encrypted).map_err(|e| e.to_string())?;
        let creds_json = String::from_utf8(decrypted).map_err(|e| e.to_string())?;
        Ok(creds_json)
    }

    pub(crate) fn inline_css_in_html(html: &str) -> Result<String, String> {
        css_inline::inline(html).map_err(|e| e.to_string())
    }

    pub(crate) fn process_outgoing_eml(&self, raw_eml: &[u8]) -> Result<Vec<u8>, String> {
        let eml_str = String::from_utf8(raw_eml.to_vec()).map_err(|e| e.to_string());

        let eml_str = match eml_str {
            Ok(s) => s,
            Err(_) => return Ok(raw_eml.to_vec()),
        };

        if let Some(html_start) = eml_str.find("Content-Type: text/html") {
            let before_html = &eml_str[..html_start];
            if let Some(html_end) = eml_str[html_start..].find("\r\n\r\n") {
                let content_start = html_start + html_end + 4;
                if let Some(boundary_end) = eml_str[content_start..].find("\r\n--") {
                    let html_content = &eml_str[content_start..content_start + boundary_end];
                    let inlined = css_inline::inline(html_content).unwrap_or_else(|_| html_content.to_string());

                    let mut result = eml_str[..content_start].to_string();
                    result.push_str(&inlined);
                    result.push_str(&eml_str[content_start + boundary_end..]);

                    return Ok(result.into_bytes());
                }
            }
        }

        Ok(raw_eml.to_vec())
    }

    pub(crate) fn extract_attachments_from_eml(&self, raw_eml: &[u8]) -> Result<Vec<(String, String, Vec<u8>)>, String> {
        let mail = mailparse::parse_mail(raw_eml).map_err(|e| e.to_string())?;

        let mut attachments = Vec::new();

        fn find_attachments<'a>(part: &'a mailparse::ParsedMail<'a>, attachments: &mut Vec<(String, String, Vec<u8>)>) {
            let mime_type = part.ctype.mimetype.as_str();

            if mime_type.starts_with("multipart/") {
                for sp in &part.subparts {
                    find_attachments(sp, attachments);
                }
            } else if !mime_type.starts_with("text/") {
                if let Some(filename) = part.ctype.params.get("name") {
                    if let Ok(data) = part.get_body_raw() {
                        attachments.push((filename.to_string(), mime_type.to_string(), data));
                    }
                }
            }
        }

        find_attachments(&mail, &mut attachments);

        Ok(attachments)
    }

    pub async fn send_email(&self, account_id: &str, eml_content: &[u8]) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT smtp_host, smtp_port, smtp_tls, auth_type FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let (host, _port, _tls, auth_type): (String, u16, bool, String) = stmt
            .query_row([account_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, i64>(1)? as u16,
                    row.get::<_, i64>(2)? != 0,
                    row.get(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        drop(stmt);
        drop(conn);

        let creds_json = self.get_credentials(account_id)?;
        let creds: serde_json::Value =
            serde_json::from_str(&creds_json).map_err(|e| e.to_string())?;

        let username = if auth_type == "oauth2" {
            creds["email"].as_str().ok_or("No email for OAuth")?
        } else {
            creds["username"].as_str().ok_or("No username")?
        };
        let password = if auth_type == "oauth2" {
            creds["access_token"].as_str().ok_or("No access_token")?
        } else {
            creds["password"].as_str().ok_or("No password")?
        };

        let creds_smtp = Credentials::new(username.to_string(), password.to_string());

        let mailer = SmtpTransport::relay(&host)
            .map_err(|e| e.to_string())?
            .credentials(creds_smtp)
            .build();

        let processed_eml = self.process_outgoing_eml(eml_content)?;

        let message = Message::builder()
            .body(processed_eml)
            .map_err(|e| e.to_string())?;

        mailer.send(&message).map_err(|e| e.to_string())?;
        Ok(())
    }
}
