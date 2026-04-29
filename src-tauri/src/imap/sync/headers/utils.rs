pub fn flag_to_string(flag: &async_imap::types::Flag) -> String {
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

#[macro_export]
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
pub fn clean_snippet(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}

/// Decode raw MIME part bytes into a plain-text snippet.
/// Handles CTE (base64/QP/raw) and charset conversion directly
pub fn decode_part_preview(raw: &[u8], mime_type: &str, charset: &str, encoding: &str) -> String {
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
pub fn find_text_part(
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

pub fn section_path_to_string(path: &[u32]) -> String {
    if path.is_empty() {
        "1".to_string()
    } else {
        path.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}
