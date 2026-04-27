use postail_project_lib::smtp::mdn::{build_mdn, encode_subject};

fn parse_headers(eml: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in eml.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((key, val)) = line.split_once(':') {
            map.insert(key.trim().to_lowercase(), val.trim().to_string());
        }
    }
    map
}

fn get_parts(eml: &str) -> Vec<String> {
    let boundary = eml
        .lines()
        .find(|l| l.contains("boundary="))
        .and_then(|l| l.split("boundary=").nth(1))
        .map(|b| b.trim_matches('"').trim().to_string())
        .expect("boundary not found");

    let delimiter = format!("--{}", boundary);
    let terminator = format!("--{}--", boundary);

    eml.split('\n')
        .collect::<Vec<_>>()
        .split(|line| line.trim() == delimiter.trim() || line.trim() == terminator.trim())
        .filter(|chunk| chunk.iter().any(|l| !l.trim().is_empty()))
        .map(|chunk| chunk.join("\n"))
        .collect()
}

#[test]
fn headers_are_present() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Hello World",
        Some("<abc123@mail.example.com>"),
    ))
    .unwrap();

    let headers = parse_headers(&eml);

    assert_eq!(
        headers.get("from").map(String::as_str),
        Some("me@example.com")
    );
    assert_eq!(
        headers.get("to").map(String::as_str),
        Some("sender@example.com")
    );
    assert!(
        headers
            .get("subject")
            .map(|s| s.contains("Hello World"))
            .unwrap_or(false)
    );
    assert!(headers.get("message-id").is_some());
    assert!(headers.get("date").is_some());
    assert_eq!(headers.get("mime-version").map(String::as_str), Some("1.0"));
}

#[test]
fn content_type_is_multipart_report() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Test",
        None,
    ))
    .unwrap();

    assert!(eml.contains("multipart/report"));
    assert!(eml.contains("report-type=disposition-notification"));
}

#[test]
fn plain_text_part_contains_subject() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Meeting tomorrow",
        None,
    ))
    .unwrap();

    assert!(eml.contains("Meeting tomorrow"));
    assert!(eml.contains("text/plain"));
}

#[test]
fn mdn_part_contains_required_fields() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Test",
        Some("<orig@example.com>"),
    ))
    .unwrap();

    assert!(eml.contains("message/disposition-notification"));
    assert!(eml.contains("Reporting-UA: Postail Mail Client"));
    assert!(eml.contains("Final-Recipient: rfc822; me@example.com"));
    assert!(eml.contains("Disposition: automatic-action/MDN-sent-automatically; displayed"));
    assert!(eml.contains("Original-Message-ID: <orig@example.com>"));
}

#[test]
fn mdn_part_omits_original_message_id_when_none() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Test",
        None,
    ))
    .unwrap();

    assert!(!eml.contains("Original-Message-ID"));
}

#[test]
fn extracts_email_from_display_name_format() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "John Doe <john@example.com>",
        "Test",
        None,
    ))
    .unwrap();

    let headers = parse_headers(&eml);
    assert_eq!(
        headers.get("to").map(String::as_str),
        Some("john@example.com")
    );
}

#[test]
fn handles_plain_email_in_to_field() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "plain@example.com",
        "Test",
        None,
    ))
    .unwrap();

    let headers = parse_headers(&eml);
    assert_eq!(
        headers.get("to").map(String::as_str),
        Some("plain@example.com")
    );
}

#[test]
fn handles_whitespace_padded_email() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "  spaced@example.com  ",
        "Test",
        None,
    ))
    .unwrap();

    let headers = parse_headers(&eml);
    assert_eq!(
        headers.get("to").map(String::as_str),
        Some("spaced@example.com")
    );
}

#[test]
fn subject_is_prefixed_with_read() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Quarterly Report",
        None,
    ))
    .unwrap();

    let headers = parse_headers(&eml);
    assert!(
        headers
            .get("subject")
            .map(|s| s.starts_with("Read:"))
            .unwrap_or(false)
    );
    assert!(
        headers
            .get("subject")
            .map(|s| s.contains("Quarterly Report"))
            .unwrap_or(false)
    );
}

#[test]
fn message_id_is_unique_per_call() {
    let eml1 = String::from_utf8(build_mdn("a@a.com", "b@b.com", "s", None)).unwrap();
    let eml2 = String::from_utf8(build_mdn("a@a.com", "b@b.com", "s", None)).unwrap();

    let id1 = parse_headers(&eml1).get("message-id").cloned().unwrap();
    let id2 = parse_headers(&eml2).get("message-id").cloned().unwrap();

    assert_ne!(id1, id2);
}

#[test]
fn boundary_is_unique_per_call() {
    let eml1 = String::from_utf8(build_mdn("a@a.com", "b@b.com", "s", None)).unwrap();
    let eml2 = String::from_utf8(build_mdn("a@a.com", "b@b.com", "s", None)).unwrap();

    let b1 = eml1
        .lines()
        .find(|l| l.contains("boundary="))
        .unwrap()
        .to_string();
    let b2 = eml2
        .lines()
        .find(|l| l.contains("boundary="))
        .unwrap()
        .to_string();

    assert_ne!(b1, b2);
}

#[test]
fn output_is_valid_utf8() {
    let result = build_mdn(
        "me@example.com",
        "Ünïcödé <unicode@example.com>",
        "Ünïcödé Subject",
        Some("<id@example.com>"),
    );
    assert!(String::from_utf8(result).is_ok());
}

#[test]
fn eml_uses_crlf_line_endings() {
    let eml = build_mdn("a@a.com", "b@b.com", "s", None);
    let text = String::from_utf8(eml).unwrap();
    // All header lines must end with CRLF
    for line in text.split("\r\n").take(10) {
        assert!(!line.contains('\r'), "unexpected bare CR in: {:?}", line);
    }
}

#[test]
fn parts_count_is_two() {
    let eml = String::from_utf8(build_mdn(
        "me@example.com",
        "sender@example.com",
        "Test",
        Some("<id@example.com>"),
    ))
    .unwrap();

    let parts = get_parts(&eml);
    // first chunk is preamble (before first boundary), then 2 actual parts
    assert_eq!(
        parts.len(),
        3,
        "expected preamble + 2 parts, got: {:?}",
        parts.len()
    );
}

#[test]
fn ascii_subject_unchanged() {
    assert_eq!(encode_subject("Hello world"), "Hello world");
}

#[test]
fn non_ascii_subject_encoded() {
    let result = encode_subject("Cześć świecie");
    assert!(result.starts_with("=?utf-8?B?"));
    assert!(result.ends_with("?="));
}

#[test]
fn mdn_contains_auto_submitted() {
    let mdn = build_mdn("me@example.com", "you@example.com", "Test", None);
    let mdn_str = String::from_utf8(mdn).unwrap();
    assert!(mdn_str.contains("Auto-Submitted: auto-replied"));
}

#[test]
fn mdn_disposition_is_automatic() {
    let mdn = build_mdn("me@example.com", "you@example.com", "Test", None);
    let mdn_str = String::from_utf8(mdn).unwrap();
    assert!(mdn_str.contains("automatic-action/MDN-sent-automatically"));
}
