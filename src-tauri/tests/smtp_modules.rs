use postail_project_lib::db::{Draft, DraftAttachment};
use postail_project_lib::smtp::mime_builder::{build_multipart_email, replace_asset_urls_with_cids};
use postail_project_lib::smtp::EncryptionType;
use std::fs;
use std::str::FromStr;
use tempfile::TempDir;

// ============================================================================
// EncryptionType tests
// ============================================================================

#[test]
fn test_encryption_type_from_str() {
    assert_eq!(EncryptionType::from_str("tls").unwrap(), EncryptionType::Tls);
    assert_eq!(EncryptionType::from_str("TLS").unwrap(), EncryptionType::Tls);
    assert_eq!(EncryptionType::from_str("ssl").unwrap(), EncryptionType::Tls);

    assert_eq!(
        EncryptionType::from_str("starttls").unwrap(),
        EncryptionType::StartTls
    );
    assert_eq!(
        EncryptionType::from_str("START_TLS").unwrap(),
        EncryptionType::StartTls
    );

    assert_eq!(EncryptionType::from_str("plain").unwrap(), EncryptionType::Plain);
    assert_eq!(EncryptionType::from_str("none").unwrap(), EncryptionType::Plain);
    assert_eq!(EncryptionType::from_str("").unwrap(), EncryptionType::Plain);

    assert!(EncryptionType::from_str("invalid").is_err());
}

// ============================================================================
// MIME Builder tests
// ============================================================================

#[test]
fn test_build_simple_email() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test Subject";
    let html_body = "<html><body><p>Hello, World!</p></body></html>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_bytes = result.unwrap();
    assert!(!eml_bytes.is_empty());

    let eml_string = String::from_utf8_lossy(&eml_bytes);
    assert!(eml_string.contains("From: sender@example.com"));
    assert!(eml_string.contains("To: recipient@example.com"));
    assert!(eml_string.contains("Subject: Test Subject"));
    assert!(eml_string.contains("Hello, World!"));
}

#[test]
fn test_build_email_with_cc_and_bcc() {
    let from = "sender@example.com";
    let to = vec!["recipient1@example.com"];
    let cc = vec!["cc@example.com"];
    let bcc = vec!["bcc@example.com"];
    let subject = "Test with CC and BCC";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_string = String::from_utf8_lossy(&result.unwrap());
    assert!(eml_string.contains("Cc: cc@example.com"));
    assert!(eml_string.contains("Bcc: bcc@example.com"));
}

#[test]
fn test_build_email_with_multiple_recipients() {
    let from = "sender@example.com";
    let to = vec!["recipient1@example.com", "recipient2@example.com", "recipient3@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test Multiple Recipients";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_string = String::from_utf8_lossy(&result.unwrap());
    assert!(eml_string.contains("recipient1@example.com"));
    assert!(eml_string.contains("recipient2@example.com"));
    assert!(eml_string.contains("recipient3@example.com"));
}

#[test]
fn test_build_email_with_regular_attachment() {
    let temp_dir = TempDir::new().unwrap();
    let attachment_path = temp_dir.path().join("test_attachment.txt");
    fs::write(&attachment_path, b"Attachment content").unwrap();

    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test with Attachment";
    let html_body = "<p>See attachment</p>";
    let attachments = vec![DraftAttachment {
        id: "att1".to_string(),
        filename: "test_attachment.txt".to_string(),
        content_type: "text/plain".to_string(),
        size: 18,
        hash: "hash".to_string(),
        path: attachment_path.to_string_lossy().to_string(),
        cid: None,
        inline: false,
    }];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_string = String::from_utf8_lossy(&result.unwrap());
    assert!(eml_string.contains("test_attachment.txt"));
    assert!(eml_string.contains("text/plain"));
}

#[test]
fn test_build_email_with_inline_attachment() {
    let temp_dir = TempDir::new().unwrap();
    let image_path = temp_dir.path().join("image.png");
    fs::write(&image_path, b"\x89PNG\r\n\x1a\n").unwrap();

    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test with Inline Image";
    let html_body = "<p>Image: <img src=\"cid:image123@postail.local\"></p>";
    let attachments = vec![DraftAttachment {
        id: "att1".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 8,
        hash: "hash".to_string(),
        path: image_path.to_string_lossy().to_string(),
        cid: Some("image123@postail.local".to_string()),
        inline: true,
    }];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_string = String::from_utf8_lossy(&result.unwrap());
    assert!(eml_string.contains("image123@postail.local"));
}

#[test]
fn test_build_email_with_mixed_attachments() {
    let temp_dir = TempDir::new().unwrap();

    let regular_path = temp_dir.path().join("document.pdf");
    fs::write(&regular_path, b"PDF content").unwrap();

    let inline_path = temp_dir.path().join("inline.jpg");
    fs::write(&inline_path, b"JPG content").unwrap();

    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test with Mixed Attachments";
    let html_body = "<p>Inline: <img src=\"cid:inline@postail.local\"></p>";
    let attachments = vec![
        DraftAttachment {
            id: "att1".to_string(),
            filename: "document.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 11,
            hash: "hash1".to_string(),
            path: regular_path.to_string_lossy().to_string(),
            cid: None,
            inline: false,
        },
        DraftAttachment {
            id: "att2".to_string(),
            filename: "inline.jpg".to_string(),
            content_type: "image/jpeg".to_string(),
            size: 11,
            hash: "hash2".to_string(),
            path: inline_path.to_string_lossy().to_string(),
            cid: Some("inline@postail.local".to_string()),
            inline: true,
        },
    ];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_string = String::from_utf8_lossy(&result.unwrap());
    assert!(eml_string.contains("document.pdf"));
    assert!(eml_string.contains("inline@postail.local"));
}

#[test]
fn test_build_email_with_invalid_from() {
    let from = "invalid-email";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_err());
}

#[test]
fn test_build_email_with_invalid_to() {
    let from = "sender@example.com";
    let to = vec!["invalid-email"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_err());
}

#[test]
fn test_build_email_with_missing_attachment_file() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test";
    let html_body = "<p>Test</p>";
    let attachments = vec![DraftAttachment {
        id: "att1".to_string(),
        filename: "missing.txt".to_string(),
        content_type: "text/plain".to_string(),
        size: 100,
        hash: "hash".to_string(),
        path: "/nonexistent/path/missing.txt".to_string(),
        cid: None,
        inline: false,
    }];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_err());
}

#[test]
fn test_build_email_with_inline_attachment_missing_cid() {
    let temp_dir = TempDir::new().unwrap();
    let image_path = temp_dir.path().join("image.png");
    fs::write(&image_path, b"PNG content").unwrap();

    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test";
    let html_body = "<p>Test</p>";
    let attachments = vec![DraftAttachment {
        id: "att1".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 11,
        hash: "hash".to_string(),
        path: image_path.to_string_lossy().to_string(),
        cid: None, // Missing CID for inline attachment
        inline: true,
    }];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_err());
}

#[test]
fn test_build_email_with_special_characters_in_subject() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test with émojis 🎉 and ñ characters 日本語";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_string = String::from_utf8_lossy(&result.unwrap());
    assert!(eml_string.contains("Subject:"));
}

#[test]
fn test_build_email_with_html_entities() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test HTML";
    let html_body = "<html><body><p>&lt;Hello&gt; &amp; &quot;World&quot;</p></body></html>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());

    let eml_bytes = result.unwrap();
    let eml_string = String::from_utf8_lossy(&eml_bytes);
    assert!(eml_string.contains("&lt;Hello&gt;"));
}

// ============================================================================
// Asset URL replacement tests
// ============================================================================

#[test]
fn test_replace_asset_urls_with_cids_no_assets() {
    let html = "<html><body><p>No assets here</p></body></html>";
    let attachments = vec![];

    let result = replace_asset_urls_with_cids(html, &attachments);
    assert_eq!(result, html);
}

#[test]
fn test_replace_asset_urls_with_cids_single_asset() {
    let html = r#"<html><body><img src="asset://localhost/att-123" /></body></html>"#;
    let attachments = vec![DraftAttachment {
        id: "att-123".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 1024,
        hash: "hash".to_string(),
        path: "/path/to/image.png".to_string(),
        cid: Some("cid123@postail.local".to_string()),
        inline: true,
    }];

    let result = replace_asset_urls_with_cids(html, &attachments);
    assert!(result.contains("cid:cid123@postail.local"));
    assert!(!result.contains("asset://"));
}

#[test]
fn test_replace_asset_urls_with_cids_multiple_assets() {
    let html = r#"<html><body>
        <img src="asset://localhost/att-1" />
        <img src="asset://localhost/att-2" />
    </body></html>"#;
    let attachments = vec![
        DraftAttachment {
            id: "att-1".to_string(),
            filename: "image1.png".to_string(),
            content_type: "image/png".to_string(),
            size: 1024,
            hash: "hash1".to_string(),
            path: "/path/to/image1.png".to_string(),
            cid: Some("cid1@postail.local".to_string()),
            inline: true,
        },
        DraftAttachment {
            id: "att-2".to_string(),
            filename: "image2.png".to_string(),
            content_type: "image/png".to_string(),
            size: 2048,
            hash: "hash2".to_string(),
            path: "/path/to/image2.png".to_string(),
            cid: Some("cid2@postail.local".to_string()),
            inline: true,
        },
    ];

    let result = replace_asset_urls_with_cids(html, &attachments);
    assert!(result.contains("cid:cid1@postail.local"));
    assert!(result.contains("cid:cid2@postail.local"));
    assert!(!result.contains("asset://"));
}

#[test]
fn test_replace_asset_urls_with_cids_ignores_non_inline() {
    let html = r#"<html><body><img src="asset://localhost/att-1" /></body></html>"#;
    let attachments = vec![DraftAttachment {
        id: "att-1".to_string(),
        filename: "document.pdf".to_string(),
        content_type: "application/pdf".to_string(),
        size: 1024,
        hash: "hash".to_string(),
        path: "/path/to/document.pdf".to_string(),
        cid: None,
        inline: false,
    }];

    let result = replace_asset_urls_with_cids(html, &attachments);
    // Non-inline attachments should not be replaced
    assert!(result.contains("asset://"));
}

#[test]
fn test_replace_asset_urls_with_cids_preserves_other_urls() {
    let html = r#"<html><body>
        <img src="asset://localhost/att-1" />
        <a href="https://example.com">Link</a>
        <img src="http://example.com/image.png" />
    </body></html>"#;
    let attachments = vec![DraftAttachment {
        id: "att-1".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 1024,
        hash: "hash".to_string(),
        path: "/path/to/image.png".to_string(),
        cid: Some("cid1@postail.local".to_string()),
        inline: true,
    }];

    let result = replace_asset_urls_with_cids(html, &attachments);
    assert!(result.contains("cid:cid1@postail.local"));
    assert!(result.contains("https://example.com"));
    assert!(result.contains("http://example.com/image.png"));
}

#[test]
fn test_replace_asset_urls_with_cids_empty_html() {
    let html = "";
    let attachments = vec![DraftAttachment {
        id: "att-1".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 1024,
        hash: "hash".to_string(),
        path: "/path/to/image.png".to_string(),
        cid: Some("cid1@postail.local".to_string()),
        inline: true,
    }];

    let result = replace_asset_urls_with_cids(html, &attachments);
    assert_eq!(result, "");
}

#[test]
fn test_replace_asset_urls_with_cids_no_matching_attachment() {
    let html = r#"<html><body><img src="asset://localhost/att-999" /></body></html>"#;
    let attachments = vec![DraftAttachment {
        id: "att-1".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 1024,
        hash: "hash".to_string(),
        path: "/path/to/image.png".to_string(),
        cid: Some("cid1@postail.local".to_string()),
        inline: true,
    }];

    let result = replace_asset_urls_with_cids(html, &attachments);
    // URL without matching attachment should remain unchanged
    assert!(result.contains("asset://localhost/att-999"));
}

// ============================================================================
// Edge cases and stress tests
// ============================================================================

#[test]
fn test_build_email_with_very_long_subject() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "A".repeat(500);
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, &subject, html_body, &attachments);
    assert!(result.is_ok());
}

#[test]
fn test_build_email_with_very_long_body() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test";
    let html_body = format!("<html><body>{}</body></html>", "<p>Test paragraph</p>".repeat(1000));
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, &html_body, &attachments);
    assert!(result.is_ok());

    let eml_bytes = result.unwrap();
    assert!(eml_bytes.len() > 10000);
}

#[test]
fn test_build_email_with_many_recipients() {
    let from = "sender@example.com";
    let to: Vec<&str> = (1..=50)
        .map(|i| format!("recipient{}@example.com", i))
        .collect::<Vec<_>>()
        .iter()
        .map(|s| s.as_str())
        .collect();
    let cc = vec![];
    let bcc = vec![];
    let subject = "Test Many Recipients";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to.clone(), cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());
}

#[test]
fn test_replace_asset_urls_with_same_attachment_multiple_times() {
    let html = r#"<html><body>
        <img src="asset://localhost/att-1" />
        <img src="asset://localhost/att-1" />
        <img src="asset://localhost/att-1" />
    </body></html>"#;
    let attachments = vec![DraftAttachment {
        id: "att-1".to_string(),
        filename: "image.png".to_string(),
        content_type: "image/png".to_string(),
        size: 1024,
        hash: "hash".to_string(),
        path: "/path/to/image.png".to_string(),
        cid: Some("cid1@postail.local".to_string()),
        inline: true,
    }];

    let result = replace_asset_urls_with_cids(html, &attachments);

    // Count occurrences of the CID
    let cid_count = result.matches("cid:cid1@postail.local").count();
    assert!(cid_count >= 1);

    // Ensure no asset:// URLs remain
    assert!(!result.contains("asset://"));
}

#[test]
fn test_encryption_type_case_insensitive() {
    assert_eq!(EncryptionType::from_str("TLS").unwrap(), EncryptionType::Tls);
    assert_eq!(EncryptionType::from_str("tls").unwrap(), EncryptionType::Tls);
    assert_eq!(EncryptionType::from_str("Tls").unwrap(), EncryptionType::Tls);

    assert_eq!(EncryptionType::from_str("STARTTLS").unwrap(), EncryptionType::StartTls);
    assert_eq!(EncryptionType::from_str("starttls").unwrap(), EncryptionType::StartTls);
    assert_eq!(EncryptionType::from_str("StartTls").unwrap(), EncryptionType::StartTls);
}

#[test]
fn test_build_email_empty_subject() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "";
    let html_body = "<p>Test</p>";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());
}

#[test]
fn test_build_email_empty_body() {
    let from = "sender@example.com";
    let to = vec!["recipient@example.com"];
    let cc = vec![];
    let bcc = vec![];
    let subject = "Empty Body";
    let html_body = "";
    let attachments = vec![];

    let result = build_multipart_email(from, to, cc, bcc, subject, html_body, &attachments);
    assert!(result.is_ok());
}