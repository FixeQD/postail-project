use postail_project_lib::smtp::mime_builder::build_multipart_email;

fn find_header<'a>(eml: &'a str, name: &str) -> Option<&'a str> {
    let lower = name.to_lowercase();
    for line in eml.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((key, val)) = line.split_once(':') {
            if key.trim().to_lowercase() == lower {
                return Some(val.trim());
            }
        }
    }
    None
}

#[test]
fn disposition_notification_header_is_added_when_requested() {
    let eml_bytes = build_multipart_email(
        "sender@example.com",
        vec!["recipient@example.com"],
        vec![],
        vec![],
        "Test subject",
        "<p>Hello</p>",
        &[],
        Some("sender@example.com"),
    )
    .expect("build_multipart_email failed");

    let eml = String::from_utf8(eml_bytes).expect("eml is not valid utf-8");
    let header = find_header(&eml, "Disposition-Notification-To");

    assert!(
        header.is_some(),
        "Disposition-Notification-To header missing from EML.\nEML headers:\n{}",
        eml.lines().take(20).collect::<Vec<_>>().join("\n")
    );
    assert!(
        header.unwrap().contains("sender@example.com"),
        "Disposition-Notification-To has unexpected value: {:?}",
        header
    );
}

#[test]
fn disposition_notification_header_absent_when_not_requested() {
    let eml_bytes = build_multipart_email(
        "sender@example.com",
        vec!["recipient@example.com"],
        vec![],
        vec![],
        "Test subject",
        "<p>Hello</p>",
        &[],
        None,
    )
    .expect("build_multipart_email failed");

    let eml = String::from_utf8(eml_bytes).expect("eml is not valid utf-8");
    let header = find_header(&eml, "Disposition-Notification-To");

    assert!(
        header.is_none(),
        "Disposition-Notification-To should not be present but found: {:?}",
        header
    );
}
