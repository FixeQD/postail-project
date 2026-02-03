//! Pipeline functions for HTML sanitization
//!
//! Orchestrates all stages: preprocessing → CSS processing → sanitization → postprocessing

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::stages::*;
use crate::utils::sanitizer::types::*;
use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;

/// Internal helper to run the sanitization pipeline
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

pub fn auto_fix_email_html(html: &str) -> String {
    run_sanitization_pipeline(html, true, false)
}

pub fn sanitize_email_html(html: &str) -> String {
    run_sanitization_pipeline(html, false, false)
}

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
