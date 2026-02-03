use postail_project_lib::oauth::provider::{ProviderInfo, ProviderKind};

// ============================================================================
// ProviderKind tests
// ============================================================================

#[test]
fn test_provider_kind_as_str() {
    assert_eq!(ProviderKind::Gmail.as_str(), "gmail");
    assert_eq!(ProviderKind::Outlook.as_str(), "outlook");
}

#[test]
fn test_provider_kind_parse() {
    assert_eq!(ProviderKind::parse("gmail"), Some(ProviderKind::Gmail));
    assert_eq!(ProviderKind::parse("outlook"), Some(ProviderKind::Outlook));
    assert_eq!(ProviderKind::parse("GMAIL"), None); // Case sensitive
    assert_eq!(ProviderKind::parse("unknown"), None);
    assert_eq!(ProviderKind::parse(""), None);
}

#[test]
fn test_provider_kind_display_name() {
    assert_eq!(ProviderKind::Gmail.display_name(), "Gmail");
    assert_eq!(ProviderKind::Outlook.display_name(), "Outlook");
}

#[test]
fn test_provider_kind_display_trait() {
    assert_eq!(format!("{}", ProviderKind::Gmail), "Gmail");
    assert_eq!(format!("{}", ProviderKind::Outlook), "Outlook");
}

#[test]
fn test_provider_kind_from_imap_host() {
    assert_eq!(
        ProviderKind::from_imap_host("imap.gmail.com"),
        Some(ProviderKind::Gmail)
    );
    assert_eq!(
        ProviderKind::from_imap_host("outlook.office365.com"),
        Some(ProviderKind::Outlook)
    );
    assert_eq!(ProviderKind::from_imap_host("imap.example.com"), None);
    assert_eq!(ProviderKind::from_imap_host(""), None);
}

#[test]
fn test_provider_kind_debug() {
    let gmail = ProviderKind::Gmail;
    let debug_str = format!("{:?}", gmail);
    assert!(debug_str.contains("Gmail"));
}

#[test]
fn test_provider_kind_equality() {
    assert_eq!(ProviderKind::Gmail, ProviderKind::Gmail);
    assert_ne!(ProviderKind::Gmail, ProviderKind::Outlook);
}

#[test]
fn test_provider_kind_clone() {
    let gmail = ProviderKind::Gmail;
    let gmail_clone = gmail.clone();
    assert_eq!(gmail, gmail_clone);
}

#[test]
fn test_provider_kind_copy() {
    let gmail = ProviderKind::Gmail;
    let gmail_copy = gmail;
    assert_eq!(gmail, gmail_copy);
}

// ============================================================================
// ProviderInfo tests - Gmail
// ============================================================================

#[test]
fn test_gmail_provider_info() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);

    assert_eq!(gmail.kind, ProviderKind::Gmail);
    assert_eq!(gmail.name, "Gmail");
    assert_eq!(
        gmail.auth_url,
        "https://accounts.google.com/o/oauth2/v2/auth"
    );
    assert_eq!(gmail.token_url, "https://oauth2.googleapis.com/token");
    assert_eq!(gmail.imap_host, "imap.gmail.com");
    assert_eq!(gmail.smtp_host, "smtp.gmail.com");
    assert_eq!(gmail.sent_folder, "[Gmail]/Sent Mail");
}

#[test]
fn test_gmail_scopes() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);

    assert!(gmail.scopes.contains("https://mail.google.com/"));
    assert!(gmail
        .scopes
        .contains("https://www.googleapis.com/auth/gmail.send"));
    assert!(gmail
        .scopes
        .contains("https://www.googleapis.com/auth/userinfo.email"));
    assert!(gmail
        .scopes
        .contains("https://www.googleapis.com/auth/gmail.readonly"));
}

#[test]
fn test_gmail_client_id_and_secret() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);

    // These will be None unless compile-time env vars are set
    // Just ensure the methods don't panic
    let _client_id = gmail.client_id();
    let _client_secret = gmail.client_secret();
}

// ============================================================================
// ProviderInfo tests - Outlook
// ============================================================================

#[test]
fn test_outlook_provider_info() {
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    assert_eq!(outlook.kind, ProviderKind::Outlook);
    assert_eq!(outlook.name, "Outlook");
    assert_eq!(
        outlook.auth_url,
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
    );
    assert_eq!(
        outlook.token_url,
        "https://login.microsoftonline.com/common/oauth2/v2.0/token"
    );
    assert_eq!(outlook.imap_host, "outlook.office365.com");
    assert_eq!(outlook.smtp_host, "smtp-mail.outlook.com");
    assert_eq!(outlook.sent_folder, "Sent Items");
}

#[test]
fn test_outlook_scopes() {
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    assert!(outlook
        .scopes
        .contains("https://outlook.office.com/IMAP.AccessAsUser.All"));
    assert!(outlook
        .scopes
        .contains("https://outlook.office.com/SMTP.Send"));
}

#[test]
fn test_outlook_client_id_and_secret() {
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    // These will be None unless compile-time env vars are set
    // Just ensure the methods don't panic
    let _client_id = outlook.client_id();
    let _client_secret = outlook.client_secret();
}

// ============================================================================
// ProviderInfo comparison tests
// ============================================================================

#[test]
fn test_provider_info_equality() {
    let gmail1 = ProviderInfo::get(ProviderKind::Gmail);
    let gmail2 = ProviderInfo::get(ProviderKind::Gmail);

    assert_eq!(gmail1, gmail2);
}

#[test]
fn test_provider_info_inequality() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    assert_ne!(gmail, outlook);
    assert_ne!(gmail.kind, outlook.kind);
    assert_ne!(gmail.name, outlook.name);
    assert_ne!(gmail.auth_url, outlook.auth_url);
    assert_ne!(gmail.token_url, outlook.token_url);
    assert_ne!(gmail.imap_host, outlook.imap_host);
    assert_ne!(gmail.smtp_host, outlook.smtp_host);
    assert_ne!(gmail.sent_folder, outlook.sent_folder);
}

#[test]
fn test_provider_info_clone() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let gmail_clone = gmail.clone();

    assert_eq!(gmail, gmail_clone);
    assert_eq!(gmail.kind, gmail_clone.kind);
    assert_eq!(gmail.name, gmail_clone.name);
}

#[test]
fn test_provider_info_debug() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let debug_str = format!("{:?}", gmail);

    assert!(debug_str.contains("Gmail"));
    assert!(debug_str.contains("imap.gmail.com"));
}

// ============================================================================
// Integration tests
// ============================================================================

#[test]
fn test_provider_roundtrip_from_str_to_info() {
    let provider_str = "gmail";
    let kind = ProviderKind::parse(provider_str).unwrap();
    let info = ProviderInfo::get(kind);

    assert_eq!(info.name, "Gmail");
    assert_eq!(info.kind.as_str(), provider_str);
}

#[test]
fn test_provider_roundtrip_from_imap_host_to_info() {
    let imap_host = "imap.gmail.com";
    let kind = ProviderKind::from_imap_host(imap_host).unwrap();
    let info = ProviderInfo::get(kind);

    assert_eq!(info.imap_host, imap_host);
    assert_eq!(info.kind, ProviderKind::Gmail);
}

#[test]
fn test_all_providers_have_valid_urls() {
    let providers = vec![ProviderKind::Gmail, ProviderKind::Outlook];

    for provider_kind in providers {
        let info = ProviderInfo::get(provider_kind);

        // Check auth_url is valid HTTPS
        assert!(info.auth_url.starts_with("https://"));
        assert!(info.auth_url.contains("oauth"));

        // Check token_url is valid HTTPS
        assert!(info.token_url.starts_with("https://"));
        assert!(info.token_url.contains("token"));

        // Check IMAP host is not empty
        assert!(!info.imap_host.is_empty());

        // Check SMTP host is not empty
        assert!(!info.smtp_host.is_empty());

        // Check sent folder is not empty
        assert!(!info.sent_folder.is_empty());

        // Check scopes is not empty
        assert!(!info.scopes.is_empty());
    }
}

#[test]
fn test_all_providers_have_unique_properties() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    // Ensure each provider has unique properties
    assert_ne!(gmail.auth_url, outlook.auth_url);
    assert_ne!(gmail.token_url, outlook.token_url);
    assert_ne!(gmail.imap_host, outlook.imap_host);
    assert_ne!(gmail.smtp_host, outlook.smtp_host);
    assert_ne!(gmail.sent_folder, outlook.sent_folder);
}

// ============================================================================
// Edge cases and negative tests
// ============================================================================

#[test]
fn test_provider_kind_parse_empty_string() {
    assert_eq!(ProviderKind::parse(""), None);
}

#[test]
fn test_provider_kind_parse_whitespace() {
    assert_eq!(ProviderKind::parse(" "), None);
    assert_eq!(ProviderKind::parse("\t"), None);
    assert_eq!(ProviderKind::parse("\n"), None);
}

#[test]
fn test_provider_kind_parse_case_sensitive() {
    // Should be case-sensitive
    assert_eq!(ProviderKind::parse("Gmail"), None);
    assert_eq!(ProviderKind::parse("GMAIL"), None);
    assert_eq!(ProviderKind::parse("Outlook"), None);
    assert_eq!(ProviderKind::parse("OUTLOOK"), None);

    // Only lowercase should work
    assert_eq!(ProviderKind::parse("gmail"), Some(ProviderKind::Gmail));
    assert_eq!(ProviderKind::parse("outlook"), Some(ProviderKind::Outlook));
}

#[test]
fn test_provider_kind_parse_special_characters() {
    assert_eq!(ProviderKind::parse("gmail!"), None);
    assert_eq!(ProviderKind::parse("@outlook"), None);
    assert_eq!(ProviderKind::parse("gmail outlook"), None);
}

#[test]
fn test_provider_kind_from_imap_host_case_sensitive() {
    // Should match exact case
    assert_eq!(
        ProviderKind::from_imap_host("imap.gmail.com"),
        Some(ProviderKind::Gmail)
    );
    assert_eq!(ProviderKind::from_imap_host("IMAP.GMAIL.COM"), None);
    assert_eq!(ProviderKind::from_imap_host("Imap.Gmail.Com"), None);
}

#[test]
fn test_provider_kind_from_imap_host_partial_match() {
    // Should not match partial strings
    assert_eq!(ProviderKind::from_imap_host("gmail.com"), None);
    assert_eq!(ProviderKind::from_imap_host("imap.gmail"), None);
    assert_eq!(ProviderKind::from_imap_host("mail.google.com"), None);
}

#[test]
fn test_provider_info_sent_folder_differences() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    // Gmail uses a special folder format
    assert!(gmail.sent_folder.contains("[Gmail]"));

    // Outlook uses a different format
    assert!(outlook.sent_folder.contains("Sent Items"));

    // They should be different
    assert_ne!(gmail.sent_folder, outlook.sent_folder);
}

#[test]
fn test_provider_info_scope_formats() {
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let outlook = ProviderInfo::get(ProviderKind::Outlook);

    // Gmail uses googleapis.com domain
    assert!(gmail.scopes.contains("googleapis.com"));

    // Outlook uses outlook.office.com domain
    assert!(outlook.scopes.contains("outlook.office.com"));
}

#[test]
fn test_provider_kind_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(ProviderKind::Gmail);
    set.insert(ProviderKind::Outlook);
    set.insert(ProviderKind::Gmail); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&ProviderKind::Gmail));
    assert!(set.contains(&ProviderKind::Outlook));
}

// ============================================================================
// Stress tests
// ============================================================================

#[test]
fn test_provider_info_repeated_calls() {
    // Ensure repeated calls return consistent results
    for _ in 0..100 {
        let gmail = ProviderInfo::get(ProviderKind::Gmail);
        assert_eq!(gmail.name, "Gmail");
        assert_eq!(gmail.imap_host, "imap.gmail.com");
    }
}

#[test]
fn test_provider_kind_parse_many_times() {
    // Ensure parsing is consistent
    for _ in 0..1000 {
        assert_eq!(ProviderKind::parse("gmail"), Some(ProviderKind::Gmail));
        assert_eq!(ProviderKind::parse("outlook"), Some(ProviderKind::Outlook));
        assert_eq!(ProviderKind::parse("invalid"), None);
    }
}

#[test]
fn test_provider_kind_from_imap_host_many_times() {
    // Ensure consistency
    for _ in 0..1000 {
        assert_eq!(
            ProviderKind::from_imap_host("imap.gmail.com"),
            Some(ProviderKind::Gmail)
        );
        assert_eq!(
            ProviderKind::from_imap_host("outlook.office365.com"),
            Some(ProviderKind::Outlook)
        );
        assert_eq!(ProviderKind::from_imap_host("invalid.com"), None);
    }
}