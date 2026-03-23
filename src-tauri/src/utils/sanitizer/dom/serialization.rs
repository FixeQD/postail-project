//! DOM serialization utilities

use kuchikiki::NodeRef;
use regex::Regex;

/// Serialize document and clean up empty attributes
pub fn serialize_clean(document: &NodeRef) -> String {
    let html = document.to_string();
    let clean_re = Regex::new(r#"(\s+[a-zA-Z-]+)="""#).expect("invalid clean regex");
    clean_re.replace_all(&html, "$1").to_string()
}

/// Extract content from inside <body> tags
pub fn extract_body_content(html: &str) -> String {
    let body_re = Regex::new(r"(?s)<body[^>]*>(.*)</body>").unwrap();

    if let Some(caps) = body_re.captures(html) {
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
