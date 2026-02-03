//! DOM serialization utilities

use kuchiki::NodeRef;
use regex::Regex;

/// Serialize a DOM node to an HTML string and remove attributes with empty string values.
///
/// The returned string is the serialized HTML where attributes that were written with an empty
/// value (e.g., `disabled=""`) are converted to bare attributes (`disabled`).
///
/// # Examples
///
/// ```
/// // Construct a document with an empty attribute and clean it.
/// let html = r#"<input type="text" disabled="">"#;
/// let document = kuchiki::parse_html().one(html);
/// let cleaned = serialize_clean(&document);
/// assert!(cleaned.contains("disabled"));
/// assert!(!cleaned.contains("disabled=\"\""));
/// ```
pub fn serialize_clean(document: &NodeRef) -> String {
    let html = document.to_string();
    let clean_re = Regex::new(r#"(\s+[a-zA-Z-]+)="""#).expect("invalid clean regex");
    clean_re.replace_all(&html, "$1").to_string()
}

/// Extracts and returns the inner HTML of the document's `<body>` element if present.
///
/// If a `<body>` element is found, this returns the content between the opening and closing
/// `<body>` tags with leading and trailing whitespace removed. If no `<body>` element is present,
/// the original `html` input is returned unchanged.
///
/// # Examples
///
/// ```
/// let with_body = "<html><body>\n  <div>content</div>\n</body></html>";
/// assert_eq!(extract_body_content(with_body), "<div>content</div>");
///
/// let no_body = "<div>no body here</div>";
/// assert_eq!(extract_body_content(no_body), no_body.to_string());
/// ```
pub fn extract_body_content(html: &str) -> String {
    let body_re = Regex::new(r"(?s)<body[^>]*>(.*)</body>").unwrap();

    if let Some(caps) = body_re.captures(html) {
        caps[1].trim().to_string()
    } else {
        html.to_string()
    }
}

/// Normalize HTML whitespace by trimming each line and removing empty lines.
///
/// This function trims leading and trailing whitespace from every line in `html`,
/// drops lines that are empty after trimming, and joins the remaining lines with `\n`.
///
/// # Examples
///
/// ```
/// let input = "  <div>  \n\n  <p>Text</p>  \n  \n</div>  ";
/// let cleaned = cleanup_html_whitespace(input);
/// assert_eq!(cleaned, "<div>\n<p>Text</p>\n</div>");
/// ```
pub fn cleanup_html_whitespace(html: &str) -> String {
    html.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}