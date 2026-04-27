use lettre::message::{Attachment, Body, Message, MultiPart, SinglePart, header};
use std::fs;

#[derive(Debug)]
pub struct EmailBuildError(String);

impl std::fmt::Display for EmailBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EmailBuildError {}

pub fn build_multipart_email(
    from: &str,
    to: Vec<&str>,
    cc: Vec<&str>,
    bcc: Vec<&str>,
    subject: &str,
    html_body: &str,
    attachments: &[crate::db::DraftAttachment],
    disposition_notification_to: Option<&str>,
) -> Result<Vec<u8>, EmailBuildError> {
    tracing::info!(
        target: "postail",
        "[MimeBuilder] Building email - from={}, to={}, cc={}, bcc={}, subject='{}', read_receipt={}",
        from, to.len(), cc.len(), bcc.len(), subject, disposition_notification_to.is_some()
    );

    let (inline_attachments, regular_attachments): (
        Vec<&crate::db::DraftAttachment>,
        Vec<&crate::db::DraftAttachment>,
    ) = attachments.iter().partition(|att| att.inline);

    tracing::debug!(
        target: "postail",
        "[MimeBuilder] Attachments - inline: {}, regular: {}",
        inline_attachments.len(), regular_attachments.len()
    );

    let mut message_builder = Message::builder()
        .from(
            from.parse()
                .map_err(|e| EmailBuildError(format!("Invalid from address: {}", e)))?,
        )
        .subject(subject);

    for recipient in &to {
        message_builder = message_builder.to(recipient
            .parse()
            .map_err(|e| EmailBuildError(format!("Invalid to address '{}': {}", recipient, e)))?);
    }
    for recipient in &cc {
        message_builder = message_builder.cc(recipient
            .parse()
            .map_err(|e| EmailBuildError(format!("Invalid cc address '{}': {}", recipient, e)))?);
    }
    for recipient in &bcc {
        message_builder =
            message_builder.bcc(recipient.parse().map_err(|e| {
                EmailBuildError(format!("Invalid bcc address '{}': {}", recipient, e))
            })?);
    }

    if let Some(notify_addr) = disposition_notification_to {
        message_builder = message_builder.raw_header(header::HeaderValue::new(
            header::HeaderName::new_from_ascii_str("Disposition-Notification-To"),
            notify_addr.to_string(),
        ));
    }

    let mut multipart_mixed = MultiPart::mixed().build();

    if !inline_attachments.is_empty() {
        let mut multipart_related = MultiPart::related().build();
        multipart_related = multipart_related.singlepart(
            SinglePart::builder()
                .header(header::ContentType::parse("text/html; charset=utf-8").unwrap())
                .body(html_body.to_string()),
        );

        for attachment in &inline_attachments {
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

            let content_type = attachment
                .content_type
                .parse()
                .map_err(|e| EmailBuildError(format!("Invalid content type: {}", e)))?;

            let inline_part =
                Attachment::new_inline(cid.clone()).body(Body::new(file_data), content_type);

            multipart_related = multipart_related.singlepart(inline_part);
        }

        multipart_mixed = multipart_mixed.multipart(multipart_related);
    } else {
        multipart_mixed = multipart_mixed.singlepart(
            SinglePart::builder()
                .header(header::ContentType::parse("text/html; charset=utf-8").unwrap())
                .body(html_body.to_string()),
        );
    }

    for attachment in &regular_attachments {
        let file_data = fs::read(&attachment.path).map_err(|e| {
            EmailBuildError(format!(
                "Failed to read attachment {}: {}",
                attachment.id, e
            ))
        })?;

        let content_type = attachment
            .content_type
            .parse()
            .map_err(|e| EmailBuildError(format!("Invalid content type: {}", e)))?;

        let file_part =
            Attachment::new(attachment.filename.clone()).body(Body::new(file_data), content_type);

        multipart_mixed = multipart_mixed.singlepart(file_part);
    }

    let message = message_builder
        .multipart(multipart_mixed)
        .map_err(|e| EmailBuildError(format!("Failed to build message: {}", e)))?;

    let email_bytes = message.formatted();
    tracing::info!(
        target: "postail",
        "[MimeBuilder] Email built successfully, size={} bytes",
        email_bytes.len()
    );
    Ok(email_bytes)
}

/// Rewrites `asset://…<attachment_id>…` URLs in the HTML body to `cid:<cid>` references.
pub fn replace_asset_urls_with_cids(
    html: &str,
    attachments: &[crate::db::DraftAttachment],
) -> String {
    // Build a lookup: attachment_id → "cid:<cid>"
    let cid_map: std::collections::HashMap<&str, String> = attachments
        .iter()
        .filter_map(|att| {
            if att.inline {
                att.cid
                    .as_deref()
                    .map(|cid| (att.id.as_str(), format!("cid:{}", cid)))
            } else {
                None
            }
        })
        .collect();

    if cid_map.is_empty() {
        return html.to_string();
    }

    const ASSET_PREFIX: &str = "asset://";
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(ASSET_PREFIX) {
        // Copy everything before this asset:// URL verbatim.
        result.push_str(&rest[..start]);
        rest = &rest[start..];

        // Find the closing delimiter (usually `"` or `'`).
        let url_end = rest[ASSET_PREFIX.len()..]
            .find(|c| c == '"' || c == '\'')
            .map(|p| ASSET_PREFIX.len() + p)
            .unwrap_or(rest.len());

        let url = &rest[..url_end];

        // Check if this URL belongs to any of our inline attachments.
        let replacement = cid_map
            .iter()
            .find_map(|(id, cid_ref)| url.contains(*id).then_some(cid_ref.as_str()));

        match replacement {
            Some(cid_ref) => result.push_str(cid_ref),
            None => result.push_str(url),
        }

        rest = &rest[url_end..];
    }

    // Append whatever's left after the last asset:// URL.
    result.push_str(rest);
    result
}
