use chrono::Utc;
use uuid::Uuid;

pub fn build_mdn(
    from_email: &str,
    to_address: &str,
    original_subject: &str,
    original_message_id: Option<&str>,
) -> Vec<u8> {
    let boundary = format!("MDN_{}", Uuid::new_v4().simple());
    let message_id = format!("<{}@postail.local>", Uuid::new_v4());
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S +0000").to_string();

    let to_email = extract_email_addr(to_address);

    let plain_body = format!("Your message \"{}\" has been read.", original_subject);

    let mut mdn_fields = format!(
        "Reporting-UA: Postail Mail Client\r\n\
         Final-Recipient: rfc822; {}\r\n\
         Disposition: manual-action/MDN-sent-manually; displayed\r\n",
        from_email
    );
    if let Some(mid) = original_message_id {
        mdn_fields.push_str(&format!("Original-Message-ID: {}\r\n", mid));
    }

    let eml = format!(
        "From: {from}\r\n\
         To: {to}\r\n\
         Subject: Read: {subject}\r\n\
         Message-ID: {mid}\r\n\
         Date: {date}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/report; report-type=disposition-notification;\r\n\
         \tboundary=\"{boundary}\"\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {plain}\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: message/disposition-notification\r\n\
         \r\n\
         {mdn_fields}\
         --{boundary}--\r\n",
        from = from_email,
        to = to_email,
        subject = original_subject,
        mid = message_id,
        date = date,
        boundary = boundary,
        plain = plain_body,
        mdn_fields = mdn_fields,
    );

    eml.into_bytes()
}

fn extract_email_addr(addr: &str) -> &str {
    if let (Some(start), Some(end)) = (addr.rfind('<'), addr.rfind('>')) {
        if end > start {
            return &addr[start + 1..end];
        }
    }
    addr.trim()
}
