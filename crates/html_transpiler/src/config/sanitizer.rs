//! Sanitizer configuration and builder functions.

use std::borrow::Cow;

use ammonia::Builder;
use maplit::{hashmap, hashset};

use crate::css::parser::parse_css_declarations;
use crate::types::{IssueSeverity, SanitizeIssue, StyleSanitizeResult};

/// HTML tags that survive ammonia sanitization.
pub const ALLOWED_TAGS: &[&str] = &[
    "a",
    "abbr",
    "b",
    "blockquote",
    "br",
    "caption",
    "center",
    "cite",
    "code",
    "col",
    "colgroup",
    "dd",
    "del",
    "div",
    "dl",
    "dt",
    "em",
    "font",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "ins",
    "li",
    "ol",
    "p",
    "pre",
    "q",
    "s",
    "small",
    "span",
    "strike",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "tt",
    "u",
    "ul",
];

pub(crate) fn build_tag_attributes(
) -> std::collections::HashMap<&'static str, std::collections::HashSet<&'static str>> {
    hashmap! [
        "a"        => hashset!["href", "title", "target", "style"],
        "body"     => hashset!["style", "bgcolor", "text", "link", "vlink", "alink"],
        "img"      => hashset!["src", "alt", "width", "height", "style"],
        "table"    => hashset!["width", "height", "border", "cellpadding", "cellspacing",
                               "align", "bgcolor", "style", "role"],
        "td"       => hashset!["width", "height", "align", "valign", "bgcolor",
                               "colspan", "rowspan", "style"],
        "th"       => hashset!["width", "height", "align", "valign", "bgcolor",
                               "colspan", "rowspan", "style"],
        "tr"       => hashset!["align", "valign", "bgcolor", "style"],
        "div"      => hashset!["align", "style"],
        "span"     => hashset!["style"],
        "p"        => hashset!["align", "style"],
        "font"     => hashset!["color", "face", "size", "style"],
        "hr"       => hashset!["width", "size", "color", "style"],
        "html"     => hashset!["lang", "style"],
        "col"      => hashset!["width", "span", "style"],
        "colgroup" => hashset!["width", "span", "style"]
    ]
}

/// Build a shared ammonia sanitizer base. `track` enables issue recording.
pub(crate) fn build_sanitizer<'a>(track: bool) -> Builder<'a> {
    let mut builder = Builder::default();
    builder.tags(ALLOWED_TAGS.iter().cloned().collect());
    builder.tag_attributes(build_tag_attributes());
    builder.generic_attributes(hashset![
        "style",
        "class",
        "id",
        "align",
        "valign",
        "data-dead-if-empty"
    ]);
    builder.link_rel(Some("noopener noreferrer"));
    builder.attribute_filter(move |_element, attribute, value| {
        if attribute == "style" {
            let result = sanitize_style_inner(value, track);
            if result.cleaned_style.is_empty() {
                None
            } else {
                Some(Cow::Owned(result.cleaned_style))
            }
        } else {
            Some(Cow::Borrowed(value))
        }
    });
    builder
}

/// Ammonia builder that silently strips dangerous CSS.
pub fn create_email_sanitizer<'a>() -> Builder<'a> {
    build_sanitizer(false)
}

/// Ammonia builder that strips dangerous CSS and records each removal in
/// [`COLLECTED_ISSUES`].
pub fn create_sanitizer_with_tracking<'a>() -> Builder<'a> {
    build_sanitizer(true)
}

/// Strip dangerous CSS properties from a `style=""` attribute value.
pub fn sanitize_style_attribute(style: &str) -> StyleSanitizeResult {
    sanitize_style_inner(style, false)
}

#[allow(dead_code)]
fn sanitize_style_attribute_with_tracking(style: &str) -> StyleSanitizeResult {
    sanitize_style_inner(style, true)
}

pub(crate) fn sanitize_style_inner(style: &str, track: bool) -> StyleSanitizeResult {
    let mut result = StyleSanitizeResult::default();
    let mut cleaned: Vec<String> = Vec::new();

    for (prop, value) in parse_css_declarations(style) {
        if is_dangerous_property(&prop) {
            result.removed_properties.push(prop.clone());
            if track {
                let (reason, severity) = get_issue_details(&prop);
                crate::config::COLLECTED_ISSUES.with(|issues| {
                    issues.borrow_mut().push(SanitizeIssue {
                        property: prop,
                        reason,
                        severity,
                        count: 1,
                    });
                });
            }
            continue;
        }

        if prop == "font-family" {
            let sanitized = crate::config::fonts::ensure_web_safe_font_fallback(&value);
            if sanitized != value {
                result.added_font_fallback = true;
            }
            cleaned.push(format!("{}: {}", prop, sanitized));
        } else {
            cleaned.push(format!("{}: {}", prop, value));
        }
    }

    result.cleaned_style = cleaned.join("; ");
    result
}

/// Returns `true` if `prop` (or its vendor-prefixed form) is dangerous.
pub(crate) fn is_dangerous_property(prop: &str) -> bool {
    let lower = prop.to_lowercase();
    if crate::config::properties::DANGEROUS_CSS_SET.contains(lower.as_str()) {
        return true;
    }
    if lower.contains("expression") || lower.contains("behavior") {
        return true;
    }
    for prefix in ["-webkit-", "-moz-", "-ms-", "-o-"] {
        if let Some(unprefixed) = lower.strip_prefix(prefix) {
            if is_dangerous_property(unprefixed) {
                return true;
            }
        }
    }
    false
}

pub(crate) fn get_issue_details(prop: &str) -> (String, IssueSeverity) {
    match prop {
        "position" => (
            "position not supported — converted to table layout".into(),
            IssueSeverity::Warning,
        ),
        "z-index" => (
            "z-index ignored in Outlook and Gmail".into(),
            IssueSeverity::Info,
        ),
        p if p.starts_with("animation") => (
            "CSS animations not supported in email".into(),
            IssueSeverity::Warning,
        ),
        p if p.starts_with("transition") => (
            "CSS transitions not supported in email".into(),
            IssueSeverity::Warning,
        ),
        "transform" | "transform-origin" | "transform-style" | "perspective"
        | "perspective-origin" => (
            "CSS transforms not supported in email".into(),
            IssueSeverity::Warning,
        ),
        "filter" | "backdrop-filter" => (
            "CSS filters not supported in most email clients".into(),
            IssueSeverity::Warning,
        ),
        "expression" | "behavior" | "-moz-binding" => (
            "Dangerous CSS property removed for security".into(),
            IssueSeverity::Error,
        ),
        _ => (
            format!("{} removed for email compatibility", prop),
            IssueSeverity::Info,
        ),
    }
}
