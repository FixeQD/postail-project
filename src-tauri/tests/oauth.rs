//! Tests for OAuth modules:
//!   - PkceData generation and SHA-256/base64 challenge
//!   - ProviderKind parsing, display, imap-host detection, state extraction
//!   - ProviderInfo fields, email extraction, display-name prefix stripping

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

use postail_project_lib::oauth::{
    pkce::PkceData,
    provider::{ProviderInfo, ProviderKind},
};

// ── PkceData ──────────────────────────────────────────────────────────────────

#[test]
fn pkce_code_verifier_is_128_chars() {
    let pkce = PkceData::generate();
    assert_eq!(pkce.code_verifier.len(), 128);
}

#[test]
fn pkce_state_is_32_chars() {
    let pkce = PkceData::generate();
    assert_eq!(pkce.state.len(), 32);
}

#[test]
fn pkce_code_challenge_is_sha256_of_verifier() {
    let pkce = PkceData::generate();

    let mut hasher = Sha256::new();
    hasher.update(pkce.code_verifier.as_bytes());
    let expected = URL_SAFE_NO_PAD.encode(hasher.finalize());

    assert_eq!(pkce.code_challenge, expected);
}

#[test]
fn pkce_two_generations_are_unique() {
    let a = PkceData::generate();
    let b = PkceData::generate();
    assert_ne!(a.code_verifier, b.code_verifier);
    assert_ne!(a.state, b.state);
}

#[test]
fn pkce_code_verifier_is_alphanumeric() {
    let pkce = PkceData::generate();
    assert!(pkce.code_verifier.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn pkce_serializes_to_json() {
    let pkce = PkceData::generate();
    let json = serde_json::to_string(&pkce).unwrap();
    let back: PkceData = serde_json::from_str(&json).unwrap();
    assert_eq!(pkce.code_verifier, back.code_verifier);
    assert_eq!(pkce.code_challenge, back.code_challenge);
    assert_eq!(pkce.state, back.state);
}

// ── ProviderKind ──────────────────────────────────────────────────────────────

#[test]
fn provider_kind_parse_gmail() {
    assert_eq!(ProviderKind::parse("gmail"), Some(ProviderKind::Gmail));
}

#[test]
fn provider_kind_parse_outlook() {
    assert_eq!(ProviderKind::parse("outlook"), Some(ProviderKind::Outlook));
}

#[test]
fn provider_kind_parse_unknown_returns_none() {
    assert_eq!(ProviderKind::parse("yahoo"), None);
    assert_eq!(ProviderKind::parse(""), None);
    assert_eq!(ProviderKind::parse("GMAIL"), None); // case-sensitive
}

#[test]
fn provider_kind_as_str() {
    assert_eq!(ProviderKind::Gmail.as_str(), "gmail");
    assert_eq!(ProviderKind::Outlook.as_str(), "outlook");
}

#[test]
fn provider_kind_display_name() {
    assert_eq!(ProviderKind::Gmail.display_name(), "Gmail");
    assert_eq!(ProviderKind::Outlook.display_name(), "Outlook");
}

#[test]
fn provider_kind_display_trait() {
    assert_eq!(format!("{}", ProviderKind::Gmail), "Gmail");
    assert_eq!(format!("{}", ProviderKind::Outlook), "Outlook");
}

#[test]
fn provider_kind_all_contains_both() {
    let all = ProviderKind::all();
    assert!(all.contains(&ProviderKind::Gmail));
    assert!(all.contains(&ProviderKind::Outlook));
    assert_eq!(all.len(), 2);
}

#[test]
fn provider_kind_from_imap_host_gmail() {
    assert_eq!(
        ProviderKind::from_imap_host("imap.gmail.com"),
        Some(ProviderKind::Gmail)
    );
}

#[test]
fn provider_kind_from_imap_host_outlook() {
    assert_eq!(
        ProviderKind::from_imap_host("outlook.office365.com"),
        Some(ProviderKind::Outlook)
    );
}

#[test]
fn provider_kind_from_imap_host_unknown() {
    assert_eq!(ProviderKind::from_imap_host("imap.yahoo.com"), None);
    assert_eq!(ProviderKind::from_imap_host(""), None);
}

#[test]
fn provider_kind_from_state_gmail() {
    assert_eq!(
        ProviderKind::from_state("gmail:randomstuff123"),
        Some(ProviderKind::Gmail)
    );
}

#[test]
fn provider_kind_from_state_outlook() {
    assert_eq!(
        ProviderKind::from_state("outlook:abc123xyz"),
        Some(ProviderKind::Outlook)
    );
}

#[test]
fn provider_kind_from_state_invalid() {
    assert_eq!(ProviderKind::from_state("yahoo:xxx"), None);
    assert_eq!(ProviderKind::from_state(""), None);
    assert_eq!(ProviderKind::from_state("nocodon"), None);
}

#[test]
fn provider_kind_equality() {
    assert_eq!(ProviderKind::Gmail, ProviderKind::Gmail);
    assert_ne!(ProviderKind::Gmail, ProviderKind::Outlook);
}

// ── ProviderInfo ──────────────────────────────────────────────────────────────

#[test]
fn provider_info_gmail_imap_host() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    assert_eq!(info.imap_host, "imap.gmail.com");
}

#[test]
fn provider_info_outlook_imap_host() {
    let info = ProviderInfo::get(ProviderKind::Outlook);
    assert_eq!(info.imap_host, "outlook.office365.com");
}

#[test]
fn provider_info_gmail_smtp_host() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    assert_eq!(info.smtp_host, "smtp.gmail.com");
}

#[test]
fn provider_info_gmail_canonical_prefix() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    assert_eq!(info.canonical_prefix, Some("[Gmail]/"));
}

#[test]
fn provider_info_outlook_no_canonical_prefix() {
    let info = ProviderInfo::get(ProviderKind::Outlook);
    assert_eq!(info.canonical_prefix, None);
}

#[test]
fn provider_info_idle_timeout_is_29_minutes() {
    // RFC 2177: IMAP IDLE should re-issue before 30 min
    let gmail = ProviderInfo::get(ProviderKind::Gmail);
    let outlook = ProviderInfo::get(ProviderKind::Outlook);
    assert_eq!(gmail.idle_timeout_seconds, 29 * 60);
    assert_eq!(outlook.idle_timeout_seconds, 29 * 60);
}

#[test]
fn provider_info_gmail_max_idle_connections() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    assert_eq!(info.max_idle_connections, 5);
}

#[test]
fn provider_info_outlook_max_idle_connections() {
    let info = ProviderInfo::get(ProviderKind::Outlook);
    assert_eq!(info.max_idle_connections, 3);
}

#[test]
fn provider_info_extract_email_gmail() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    let json = serde_json::json!({ "email": "user@gmail.com" });
    assert_eq!(info.extract_email(&json), Some("user@gmail.com".to_string()));
}

#[test]
fn provider_info_extract_email_outlook_mail_field() {
    let info = ProviderInfo::get(ProviderKind::Outlook);
    let json = serde_json::json!({ "mail": "user@outlook.com" });
    assert_eq!(info.extract_email(&json), Some("user@outlook.com".to_string()));
}

#[test]
fn provider_info_extract_email_outlook_falls_back_to_upn() {
    let info = ProviderInfo::get(ProviderKind::Outlook);
    let json = serde_json::json!({ "userPrincipalName": "user@corp.com" });
    assert_eq!(info.extract_email(&json), Some("user@corp.com".to_string()));
}

#[test]
fn provider_info_extract_email_missing_returns_none() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    let json = serde_json::json!({ "other": "value" });
    assert_eq!(info.extract_email(&json), None);
}

#[test]
fn provider_info_strip_gmail_prefix() {
    let info = ProviderInfo::get(ProviderKind::Gmail);
    assert_eq!(info.strip_display_name_prefix("[Gmail]/Sent Mail"), "Sent Mail");
    assert_eq!(info.strip_display_name_prefix("[Gmail]/Trash"), "Trash");
    assert_eq!(info.strip_display_name_prefix("INBOX"), "INBOX"); // no prefix → unchanged
}

#[test]
fn provider_info_strip_prefix_outlook_noop() {
    let info = ProviderInfo::get(ProviderKind::Outlook);
    assert_eq!(info.strip_display_name_prefix("Sent Items"), "Sent Items");
    assert_eq!(info.strip_display_name_prefix("Inbox"), "Inbox");
}

#[test]
fn provider_info_auth_urls_are_https() {
    for kind in ProviderKind::all() {
        let info = ProviderInfo::get(*kind);
        assert!(
            info.auth_url.starts_with("https://"),
            "auth_url should be HTTPS: {}",
            info.auth_url
        );
        assert!(
            info.token_url.starts_with("https://"),
            "token_url should be HTTPS: {}",
            info.token_url
        );
    }
}

#[test]
fn provider_info_poll_interval_is_60s() {
    for kind in ProviderKind::all() {
        let info = ProviderInfo::get(*kind);
        assert_eq!(info.poll_interval_seconds, 60);
    }
}
