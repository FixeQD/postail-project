use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::LazyLock;

use ammonia::Builder;
use maplit::{hashmap, hashset};

use crate::utils::sanitizer::css::fonts::ensure_web_safe_font_fallback;
use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::{IssueSeverity, SanitizeIssue, StyleSanitizeResult};

pub static TAG_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"<([a-zA-Z][a-zA-Z0-9]*)[^>]*>").expect("Invalid regex pattern")
});

pub const DANGEROUS_CSS_PROPS: &[&str] = &[
    "position",
    "z-index",
    "animation",
    "animation-name",
    "animation-duration",
    "animation-timing-function",
    "animation-delay",
    "animation-iteration-count",
    "animation-direction",
    "animation-fill-mode",
    "animation-play-state",
    "transition",
    "transition-property",
    "transition-duration",
    "transition-timing-function",
    "transition-delay",
    "transform",
    "transform-origin",
    "transform-style",
    "perspective",
    "perspective-origin",
    "filter",
    "backdrop-filter",
    "clip-path",
    "mask",
    "mask-image",
    "mix-blend-mode",
    "isolation",
    "will-change",
    "contain",
    "content-visibility",
    "expression",
    "behavior",
    "-moz-binding",
];

pub const WEB_SAFE_FONTS: &[&str] = &[
    "Arial",
    "Helvetica",
    "Times New Roman",
    "Times",
    "Courier New",
    "Courier",
    "Verdana",
    "Georgia",
    "Palatino",
    "Garamond",
    "Comic Sans MS",
    "Trebuchet MS",
    "Arial Black",
    "Impact",
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
];

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

thread_local! {
    pub static COLLECTED_ISSUES: RefCell<Vec<SanitizeIssue>> = const { RefCell::new(Vec::new()) };
}

fn build_tag_attributes(
) -> std::collections::HashMap<&'static str, std::collections::HashSet<&'static str>> {
    hashmap! [
        "a" => hashset!["href", "title", "target", "style"],
        "body" => hashset!["style", "bgcolor", "text", "link", "vlink", "alink"],
        "img" => hashset!["src", "alt", "width", "height", "style"],
        "table" => hashset!["width", "height", "border", "cellpadding", "cellspacing", "align", "bgcolor", "style", "role"],
        "td" => hashset!["width", "height", "align", "valign", "bgcolor", "colspan", "rowspan", "style"],
        "th" => hashset!["width", "height", "align", "valign", "bgcolor", "colspan", "rowspan", "style"],
        "tr" => hashset!["align", "valign", "bgcolor", "style"],
        "div" => hashset!["align", "style"],
        "span" => hashset!["style"],
        "p" => hashset!["align", "style"],
        "font" => hashset!["color", "face", "size", "style"],
        "hr" => hashset!["width", "size", "color", "style"],
        "html" => hashset!["lang", "style"],
        "col" => hashset!["width", "span", "style"],
        "colgroup" => hashset!["width", "span", "style"]
    ]
}

pub fn create_email_sanitizer<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);
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

    builder.attribute_filter(|_element: &str, attribute: &str, value: &'_ str| {
        if attribute == "style" {
            let result = sanitize_style_attribute(value);
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

pub fn create_sanitizer_with_tracking<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);
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

    builder.attribute_filter(|_element: &str, attribute: &str, value: &'_ str| {
        if attribute == "style" {
            let result = sanitize_style_attribute_with_tracking(value);
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

pub fn sanitize_style_attribute(style: &str) -> StyleSanitizeResult {
    let mut result = StyleSanitizeResult::default();
    let mut cleaned_parts: Vec<String> = Vec::new();

    for (prop, value) in parse_css_declarations(style) {
        if is_dangerous_property(&prop) {
            result.removed_properties.push(prop);
            continue;
        }

        if prop == "font-family" {
            let sanitized_value = ensure_web_safe_font_fallback(&value);
            if sanitized_value != value {
                result.added_font_fallback = true;
            }
            cleaned_parts.push(format!("{}: {}", prop, sanitized_value));
        } else {
            cleaned_parts.push(format!("{}: {}", prop, value));
        }
    }

    result.cleaned_style = cleaned_parts.join("; ");
    result
}

fn sanitize_style_attribute_with_tracking(style: &str) -> StyleSanitizeResult {
    let mut result = StyleSanitizeResult::default();
    let mut cleaned_parts: Vec<String> = Vec::new();

    for (prop, value) in parse_css_declarations(style) {
        if is_dangerous_property(&prop) {
            result.removed_properties.push(prop.clone());
            let (reason, severity) = get_issue_details(&prop);
            COLLECTED_ISSUES.with(|issues| {
                issues.borrow_mut().push(SanitizeIssue {
                    property: prop,
                    reason,
                    severity,
                    count: 1,
                });
            });
            continue;
        }

        if prop == "font-family" {
            let sanitized_value = ensure_web_safe_font_fallback(&value);
            if sanitized_value != value {
                result.added_font_fallback = true;
            }
            cleaned_parts.push(format!("{}: {}", prop, sanitized_value));
        } else {
            cleaned_parts.push(format!("{}: {}", prop, value));
        }
    }

    result.cleaned_style = cleaned_parts.join("; ");
    result
}

fn is_dangerous_property(prop: &str) -> bool {
    let prop_lower = prop.to_lowercase();

    for dangerous in DANGEROUS_CSS_PROPS {
        if prop_lower == *dangerous {
            return true;
        }
    }

    if prop_lower.contains("expression") || prop_lower.contains("behavior") {
        return true;
    }

    let prefixes = ["-webkit-", "-moz-", "-ms-", "-o-"];
    for prefix in prefixes {
        if let Some(unprefixed) = prop_lower.strip_prefix(prefix) {
            if is_dangerous_property(unprefixed) {
                return true;
            }
        }
    }

    false
}

fn get_issue_details(prop: &str) -> (String, IssueSeverity) {
    match prop {
        "position" => (
            "position property is not supported by most email clients - converted to table layout"
                .to_string(),
            IssueSeverity::Warning,
        ),
        "z-index" => (
            "z-index is often ignored in Outlook and Gmail".to_string(),
            IssueSeverity::Info,
        ),
        p if p.starts_with("animation") => (
            "CSS animations are not supported in email clients".to_string(),
            IssueSeverity::Warning,
        ),
        p if p.starts_with("transition") => (
            "CSS transitions are not supported in email clients".to_string(),
            IssueSeverity::Warning,
        ),
        "transform" | "transform-origin" | "transform-style" | "perspective"
        | "perspective-origin" => (
            "CSS transforms/perspective are not supported in email clients".to_string(),
            IssueSeverity::Warning,
        ),
        "filter" | "backdrop-filter" => (
            "CSS filters are not supported in most email clients".to_string(),
            IssueSeverity::Warning,
        ),
        "expression" | "behavior" | "-moz-binding" => (
            "Potentially dangerous CSS property removed for security".to_string(),
            IssueSeverity::Error,
        ),
        _ => (
            format!("{} property removed for email compatibility", prop),
            IssueSeverity::Info,
        ),
    }
}
