//! DOM serialization utilities

use kuchikiki::NodeRef;
use regex::Regex;
use std::sync::LazyLock;

static CLEAN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(\s+[a-zA-Z-]+)="""#).expect("invalid clean regex"));

static BODY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<body[^>]*>(.*)</body>").unwrap());

/// Serialize document and clean up empty attributes
pub fn serialize_clean(document: &NodeRef) -> String {
    let html = document.to_string();
    CLEAN_RE.replace_all(&html, "$1").to_string()
}

/// Extract content from inside <body> tags
pub fn extract_body_content(html: &str) -> String {
    if let Some(caps) = BODY_RE.captures(html) {
        caps[1].trim().to_string()
    } else {
        html.to_string()
    }
}

/// Clean up HTML whitespace
pub fn cleanup_html_whitespace(html: &str) -> String {
    html.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
