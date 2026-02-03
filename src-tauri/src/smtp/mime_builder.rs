use lettre::message::{header, Attachment, Body, Message, MultiPart, SinglePart};
use std::fs;

#[derive(Debug)]
pub struct EmailBuildError(String);

impl std::fmt::Display for EmailBuildError {
    /// Formats the error by writing its contained message.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = crate::smtp::mime_builder::EmailBuildError("failed".into());
    /// assert_eq!(format!("{}", err), "failed");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EmailBuildError {}

/// Builds a MIME multipart email from the provided fields and attachments and returns the formatted raw message bytes.
///
/// The function assembles a "mixed" multipart message that contains the HTML body and any regular file attachments.
/// Inline attachments (attachments with `inline = true`) are embedded using a `related` multipart where each inline
/// attachment must have a `cid`. Addresses in `from`, `to`, `cc`, and `bcc` are validated.
///
/// Errors are returned as `EmailBuildError` for invalid addresses, missing CIDs for inline attachments, file I/O failures,
/// content-type parsing failures, or message construction failures.
///
/// # Parameters
///
/// - `from`: sender address as a string.
/// - `to`, `cc`, `bcc`: recipient address lists; each address is validated.
/// - `subject`: message subject.
/// - `html_body`: HTML content used as the main body (Content-Type `text/html; charset=utf-8`).
/// - `attachments`: slice of `DraftAttachment` items; inline attachments must have `cid` set and will be embedded.
///
/// # Returns
///
/// A `Vec<u8>` containing the MIME-formatted email bytes.
///
/// # Examples
///
/// ```no_run
/// use crate::smtp::mime_builder::build_multipart_email;
///
/// // Example with no attachments
/// let email_bytes = build_multipart_email(
///     "alice@example.com",
///     vec!["bob@example.com"],
///     vec![],
///     vec![],
///     "Hello",
///     "<p>Hi Bob</p>",
///     &[],
/// ).expect("failed to build email");
///
/// assert!(email_bytes.len() > 0);
/// ```
pub fn build_multipart_email(
    from: &str,
    to: Vec<&str>,
    cc: Vec<&str>,
    bcc: Vec<&str>,
    subject: &str,
    html_body: &str,
    attachments: &[crate::db::DraftAttachment],
) -> Result<Vec<u8>, EmailBuildError> {
    tracing::info!(target: "postail", "[MimeBuilder] Building email - from={}, to_count={}, cc_count={}, bcc_count={}, subject='{}'",
		from, to.len(), cc.len(), bcc.len(), subject);

    let (inline_attachments, regular_attachments): (
        Vec<&crate::db::DraftAttachment>,
        Vec<&crate::db::DraftAttachment>,
    ) = attachments.iter().partition(|att| att.inline);

    tracing::debug!(target: "postail", "[MimeBuilder] Attachments - inline: {}, regular: {}",
		inline_attachments.len(), regular_attachments.len());

    let mut message_builder = Message::builder()
        .from(
            from.parse()
                .map_err(|e| EmailBuildError(format!("Invalid from address: {}", e)))?,
        )
        .subject(subject);

    for recipient in to {
        message_builder = message_builder.to(recipient
            .parse()
            .map_err(|e| EmailBuildError(format!("Invalid to address '{}': {}", recipient, e)))?);
    }

    for recipient in cc {
        message_builder = message_builder.cc(recipient
            .parse()
            .map_err(|e| EmailBuildError(format!("Invalid cc address '{}': {}", recipient, e)))?);
    }

    for recipient in bcc {
        message_builder =
            message_builder.bcc(recipient.parse().map_err(|e| {
                EmailBuildError(format!("Invalid bcc address '{}': {}", recipient, e))
            })?);
    }

    let mut multipart_mixed = MultiPart::mixed().build();

    if !inline_attachments.is_empty() {
        let mut multipart_related = MultiPart::related().build();
        multipart_related = multipart_related.singlepart(
            SinglePart::builder()
                .header(header::ContentType::parse("text/html; charset=utf-8").unwrap())
                .body(html_body.to_string()),
        );

        for attachment in inline_attachments {
            let cid = attachment.cid.as_ref().ok_or_else(|| {
                EmailBuildError(format!(
                    "Missing CID for inline attachment {}",
                    attachment.id
                ))
            })?;

            let file_data = fs::read(&attachment.path).map_err(|e| {
                EmailBuildError(format!(
                    "Failed to read attachment {}: {}",
                    attachment.id, e
                ))
            })?;

            let body = Body::new(file_data);

            let content_type = attachment
                .content_type
                .parse()
                .map_err(|e| EmailBuildError(format!("Invalid content type: {}", e)))?;

            let inline_attachment = Attachment::new_inline(cid.clone()).body(body, content_type);

            multipart_related = multipart_related.singlepart(inline_attachment);
        }

        multipart_mixed = multipart_mixed.multipart(multipart_related);
    } else {
        multipart_mixed = multipart_mixed.singlepart(
            SinglePart::builder()
                .header(header::ContentType::parse("text/html; charset=utf-8").unwrap())
                .body(html_body.to_string()),
        );
    }

    for attachment in regular_attachments {
        let file_data = fs::read(&attachment.path).map_err(|e| {
            EmailBuildError(format!(
                "Failed to read attachment {}: {}",
                attachment.id, e
            ))
        })?;

        let body = Body::new(file_data);

        let content_type = attachment
            .content_type
            .parse()
            .map_err(|e| EmailBuildError(format!("Invalid content type: {}", e)))?;

        let file_attachment = Attachment::new(attachment.filename.clone()).body(body, content_type);

        multipart_mixed = multipart_mixed.singlepart(file_attachment);
    }

    let message = message_builder
        .multipart(multipart_mixed)
        .map_err(|e| EmailBuildError(format!("Failed to build message: {}", e)))?;

    let email_bytes = message.formatted();
    tracing::info!(target: "postail", "[MimeBuilder] Email built successfully, size={} bytes", email_bytes.len());
    Ok(email_bytes)
}

/// Replace asset:// URLs in the given HTML with corresponding `cid:` references for inline attachments.
///
/// This scans the HTML for occurrences of `asset://<...>` and, for each inline attachment that has a CID,
/// replaces any asset URL containing that attachment's `id` with `cid:<cid>`.
///
/// # Examples
///
/// ```
/// let html = r#"<img src="asset://images/img1.png" />"#;
/// let attachments = vec![crate::db::DraftAttachment {
///     id: "images/img1.png".to_string(),
///     inline: true,
///     cid: Some("abc123".to_string()),
///     ..Default::default()
/// }];
/// let result = replace_asset_urls_with_cids(html, &attachments);
/// assert_eq!(result, r#"<img src="cid:abc123" />"#);
/// ```
pub fn replace_asset_urls_with_cids(
    html: &str,
    attachments: &[crate::db::DraftAttachment],
) -> String {
    let mut result = html.to_string();

    let cid_map: std::collections::HashMap<&str, &str> = attachments
        .iter()
        .filter(|att| att.inline && att.cid.is_some())
        .map(|att| (att.id.as_str(), att.cid.as_deref().unwrap()))
        .collect();

    for (attachment_id, cid) in &cid_map {
        let mut start = 0;
        while let Some(pos) = result[start..].find("asset://") {
            let absolute_pos = start + pos;
            let end_pos = result[absolute_pos..]
                .find('"')
                .map(|p| absolute_pos + p)
                .unwrap_or(result.len());

            let url = &result[absolute_pos..end_pos];

            if url.contains(attachment_id) {
                let cid_ref = format!("cid:{}", cid);
                result.replace_range(absolute_pos..end_pos, &cid_ref);
                continue;
            }

            start = end_pos;
        }
    }

    result
}