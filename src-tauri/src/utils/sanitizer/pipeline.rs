//! Pipeline functions for HTML sanitization
//!
//! Orchestrates all stages: preprocessing → CSS processing → sanitization → postprocessing

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::stages::*;
use crate::utils::sanitizer::types::*;
use kuchiki::NodeRef;
use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;

/// Internal helper to run the sanitization pipeline
fn run_sanitization_pipeline(
    html: &str,
    is_auto_fix: bool,
    track_issues: bool,
) -> (String, HtmlDiff) {
    // ── Phase 1: Pre-processing (DOM-based) ──────────────────────────────
    // Initial DOM parse
    let document = parse_html().one(html);

    // Snapshot for diffing before we start changing things
    let diff_tracker = DiffTracker::new(&document);

    resolve_css_variables_dom(&document);
    let body_styles = extract_body_styles_dom(&document);

    // ── Phase 2: Reflow & Style Optimization (DOM-based) ──────────────────
    replace_body_with_div_dom(&document, body_styles);
    expand_pseudo_elements_dom(&document);

    if is_auto_fix {
        for node in document.descendants() {
            if let Some(element) = node.as_element() {
                if element.name.local.to_string().to_lowercase() == "style" {
                    let css = node.text_contents();
                    let without_imports = IMPORT_REGEX.replace_all(&css, "");
                    let without_font_face = FONT_FACE_REGEX.replace_all(&without_imports, "");
                    for child in node.children() {
                        child.detach();
                    }
                    node.append(NodeRef::new_text(without_font_face.to_string()));
                }
            }
        }
    }

    inline_css_styles_dom(&document);
    convert_to_table_layout_dom(&document);
    scale_elements_for_email_dom(&document);

    // ── Phase 3: Final Sanitization ───────────────────────────────────────
    strip_content_tags_dom(&document);
    mark_positioned_elements_dom(&document);

    // Initial cleaning with ammonia
    let serialized = serialize_clean(&document);
    let content_for_ammonia = extract_body_content(&serialized);

    let builder = if track_issues {
        create_sanitizer_with_tracking()
    } else {
        create_email_sanitizer()
    };

    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Final DOM cleanup of the sanitized output
    let document_final = parse_html().one(sanitized);
    strip_dead_elements_dom(&document_final);

    // Final diff calculation
    let diff = diff_tracker.calculate_diff(&document_final);

    let final_serialized = serialize_clean(&document_final);
    let result = extract_body_content(&final_serialized);

    (cleanup_html_whitespace(&result), diff)
}

pub fn auto_fix_email_html(html: &str) -> String {
    run_sanitization_pipeline(html, true, false).0
}

pub fn sanitize_email_html(html: &str) -> String {
    run_sanitization_pipeline(html, false, false).0
}

pub fn sanitize_email_html_with_details(html: &str) -> SanitizeResult {
    COLLECTED_ISSUES.with(|issues| issues.borrow_mut().clear());

    let unsupported_tags = detect_unsupported_tags(html);

    let (cleaned, diff) = run_sanitization_pipeline(html, false, true);

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
            let key = format!("{}|{}", issue.property, issue.reason);

            unique_map
                .entry(key)
                .and_modify(|existing| existing.count += 1)
                .or_insert(issue.clone());
        }

        unique_map.into_values().collect()
    });

    SanitizeResult {
        html: cleaned,
        issues,
        diff,
    }
}
