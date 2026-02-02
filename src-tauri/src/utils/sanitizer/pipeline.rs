//! Pipeline functions for HTML sanitization
//!
//! Orchestrates all stages: preprocessing → CSS processing → sanitization → postprocessing

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::stages::*;
use crate::utils::sanitizer::types::*;
use kuchiki::parse_html;
use kuchiki::traits::TendrilSink;

pub fn auto_fix_email_html(html: &str) -> String {
    // Stage 1: Parse to DOM and replace body with div
    let document = parse_html().one(html);

    // Extract body styles
    let body_styles = extract_body_styles_from_css(html);

    // Replace body with div
    replace_body_with_div_dom(&document, body_styles);

    // Serialize for CSS processing
    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    // Stage 2: Pseudo-elements expansion
    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    let without_imports = IMPORT_REGEX.replace_all(&expanded, "");
    let without_font_faces = FONT_FACE_REGEX.replace_all(&without_imports, "");
    let inlined = inline_css_styles(&without_font_faces);

    // Stage 4: Convert positioning to table layout
    let table_layout = convert_to_table_layout(&inlined);

    // Stage 5: Parse again and strip content tags
    let document2 = parse_html().one(table_layout);
    strip_content_tags_dom(&document2);

    // Stage 6: Mark positioned elements
    mark_positioned_elements_dom(&document2);

    // Stage 7: Ammonia sanitization
    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);
    let builder = create_email_sanitizer();
    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Stage 8: Strip dead elements using DOM
    let document3 = parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    // Stage 9: Aaaaaaand cleanup HTML whitespace
    cleanup_html_whitespace(&result)
}

pub fn sanitize_email_html(html: &str) -> String {
    let resolved = resolve_css_variables(html);
    let document = parse_html().one(resolved.clone());

    let body_styles = extract_body_styles_from_css(&resolved);
    replace_body_with_div_dom(&document, body_styles);

    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    // Stage 2: Pseudo-elements expansion
    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    let inlined = inline_css_styles(&expanded);

    // Stage 4: Convert positioning to table layout
    let table_layout = convert_to_table_layout(&inlined);

    // Stage 5: Parse again and strip content tags
    let document2 = parse_html().one(table_layout);
    strip_content_tags_dom(&document2);

    // Stage 6: Mark positioned elements
    mark_positioned_elements_dom(&document2);

    // Stage 7: Ammonia sanitization
    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);
    let builder = create_email_sanitizer();
    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Stage 8: Strip dead elements using DOM
    let document3 = parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    // Stage 9: Cleanup HTML whitespace
    cleanup_html_whitespace(&result)
}

pub fn sanitize_email_html_with_details(html: &str) -> SanitizeResult {
    COLLECTED_ISSUES.with(|issues| issues.borrow_mut().clear());

    let unsupported_tags = detect_unsupported_tags(html);

    let resolved = resolve_css_variables(html);
    let document = parse_html().one(resolved.clone());

    let body_styles = extract_body_styles_from_css(&resolved);
    replace_body_with_div_dom(&document, body_styles);

    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    // Stage 2: Pseudo-elements expansion
    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    let inlined = inline_css_styles(&expanded);

    // Stage 4: Convert positioning to table layout
    let table_layout = convert_to_table_layout(&inlined);

    // Stage 5: Parse again and strip content tags
    let document2 = parse_html().one(table_layout);
    strip_content_tags_dom(&document2);

    // Stage 6: Mark positioned elements
    mark_positioned_elements_dom(&document2);

    // Record unsupported tag issues
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
            });
        }
    });

    // Stage 7: Ammonia sanitization
    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);
    let builder = create_sanitizer_with_tracking();
    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Stage 8: Strip dead elements using DOM
    let document3 = parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    // Stage 9: Cleanup HTML whitespace
    let cleaned = cleanup_html_whitespace(&result);

    let issues = COLLECTED_ISSUES.with(|issues| issues.borrow().clone());

    SanitizeResult {
        html: cleaned,
        issues,
    }
}
