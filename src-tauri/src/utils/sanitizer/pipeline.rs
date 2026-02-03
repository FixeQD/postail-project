//! Pipeline functions for HTML sanitization
//!
//! Orchestrates all stages: preprocessing → CSS processing → sanitization → postprocessing

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::stages::*;
use crate::utils::sanitizer::types::*;
use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;

/// Run the full HTML sanitization pipeline for email content.
///
/// This parses the provided HTML, resolves and inlines CSS, converts layout for email
/// compatibility, applies sanitization rules, and performs final DOM cleanup,
/// returning the sanitized body HTML ready for email delivery.
///
/// # Parameters
///
/// - `html`: the input HTML fragment or document to sanitize.
/// - `is_auto_fix`: when `true`, apply aggressive preprocessing fixes (e.g., remove imports and `@font-face`) before inlining.
/// - `track_issues`: when `true`, enable issue-tracking mode in the sanitizer builder (used by callers that collect sanitization details).
///
/// # Returns
///
/// The cleaned HTML body content as a `String`.
///
/// # Examples
///
/// ```
/// let raw = "<body><style>p{color:red}</style><p>Hello</p></body>";
/// let cleaned = run_sanitization_pipeline(raw, false, false);
/// assert!(cleaned.contains("Hello"));
/// ```
fn run_sanitization_pipeline(html: &str, is_auto_fix: bool, track_issues: bool) -> String {
    let document = parse_html().one(html);
    let resolved = resolve_css_variables(html);

    let body_styles = extract_body_styles_from_css(&resolved);
    replace_body_with_div_dom(&document, body_styles);

    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    // For auto-fix, we explicitly remove imports/font-faces to prevent issues,
    let css_input = if is_auto_fix {
        let without_imports = IMPORT_REGEX.replace_all(&expanded, "");
        FONT_FACE_REGEX
            .replace_all(&without_imports, "")
            .to_string()
    } else {
        expanded
    };

    let inlined = inline_css_styles(&css_input);

    let table_layout = convert_to_table_layout(&inlined);

    let scaled = scale_elements_for_email(&table_layout);

    let document2 = parse_html().one(scaled);
    strip_content_tags_dom(&document2);

    mark_positioned_elements_dom(&document2);

    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);

    let builder = if track_issues {
        create_sanitizer_with_tracking()
    } else {
        create_email_sanitizer()
    };

    let sanitized = builder.clean(&content_for_ammonia).to_string();

    let document3 = parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    cleanup_html_whitespace(&result)
}

/// Run the email HTML sanitization pipeline with auto-fix enabled and return the fixed HTML.
///
/// # Examples
///
/// ```
/// let input = "<style>@import url('x');</style><div style=\"color: red;\">Hello</div>";
/// let out = auto_fix_email_html(input);
/// assert!(out.contains("Hello"));
/// ```
///
/// Returns the sanitized and auto-fixed HTML string.
pub fn auto_fix_email_html(html: &str) -> String {
    run_sanitization_pipeline(html, true, false)
}

/// Sanitize HTML content for email without applying automatic fixes or collecting issues.
///
/// Returns the sanitized HTML as a `String`, suitable for email rendering.
///
/// # Examples
///
/// ```
/// let cleaned = sanitize_email_html("<div><script>alert(1)</script><p>Hello</p></div>");
/// assert!(cleaned.contains("<p>Hello</p>"));
/// ```
pub fn sanitize_email_html(html: &str) -> String {
    run_sanitization_pipeline(html, false, false)
}

/// Produces a sanitized HTML string for email and a list of detected sanitization issues.
///
/// Runs the full email sanitization pipeline on `html` with issue tracking enabled and
/// returns both the cleaned HTML and an aggregated list of `SanitizeIssue` entries discovered
/// during processing. Detected issues are deduplicated and counts are aggregated by the
/// combination of `property` and `reason`.
///
/// # Examples
///
/// ```
/// let input = "<div><script>alert(1)</script><p>Hello</p></div>";
/// let result = sanitize_email_html_with_details(input);
/// assert!(result.html.contains("<p>Hello</p>"));
/// assert!(result.issues.iter().any(|i| i.property == "<script>"));
/// ```
///
/// # Returns
///
/// A `SanitizeResult` containing:
/// - `html`: the sanitized HTML string suitable for email.
/// - `issues`: a vector of deduplicated `SanitizeIssue` entries with aggregated `count` values.
pub fn sanitize_email_html_with_details(html: &str) -> SanitizeResult {
    COLLECTED_ISSUES.with(|issues| issues.borrow_mut().clear());

    let unsupported_tags = detect_unsupported_tags(html);

    let cleaned = run_sanitization_pipeline(html, false, true);

    COLLECTED_ISSUES.with(|issues| {
        let mut issues = issues.borrow_mut();
        for (tag, reason) in unsupported_tags {
            let severity = match tag.as_str() {
                "script" | "iframe" | "object" | "embed" => IssueSeverity::Error,
                "!doctype" => IssueSeverity::Info,
                _ => IssueSeverity::Warning,
            };
            issues.push(SanitizeIssue {
                property: format!("<{}>", tag),
                reason: reason.to_string(),
                severity,
                count: 1,
            });
        }
    });

    // Deduplicate and aggregate issues
    let issues = COLLECTED_ISSUES.with(|issues| {
        let issues_vec = issues.borrow();
        let mut unique_map: std::collections::HashMap<String, SanitizeIssue> =
            std::collections::HashMap::new();

        for issue in issues_vec.iter() {
            // Create a composite key for deduplication
            let key = format!("{}|{}", issue.property, issue.reason);

            unique_map
                .entry(key)
                .and_modify(|existing| existing.count += 1)
                .or_insert(issue.clone());
        }

        // Convert map back to vector
        unique_map.into_values().collect()
    });

    SanitizeResult {
        html: cleaned,
        issues,
    }
}