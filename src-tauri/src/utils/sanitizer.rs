use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::LazyLock;

use ammonia::Builder;
use maplit::hashset;

static TAG_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"<([a-zA-Z][a-zA-Z0-9]*)[^>]*>").expect("Invalid regex pattern")
});

const DANGEROUS_CSS_PROPS: &[&str] = &[
    "position",
    "z-index",
    "fixed",
    "absolute",
    "sticky",
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
    "transform",
    "perspective",
    "filter",
    "backdrop-filter",
    "clip-path",
    "mask",
    "mix-blend-mode",
    "isolation",
    "will-change",
    "contain",
    "content-visibility",
    "opacity",
    "expression",
    "behavior",
    "-moz-binding",
];

const WEB_SAFE_FONTS: &[&str] = &[
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

const ALLOWED_TAGS: &[&str] = &[
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

#[derive(Debug, Clone, Default)]
pub struct StyleSanitizeResult {
    pub cleaned_style: String,
    pub removed_properties: Vec<String>,
    pub added_font_fallback: bool,
}

fn parse_css_declarations(style: &str) -> Vec<(String, String)> {
    let mut declarations = Vec::new();
    let mut current = String::new();
    let mut paren_depth: i32 = 0;
    let mut in_string = false;
    let mut string_char = '"';

    for ch in style.chars() {
        match ch {
            '"' | '\'' if !in_string => {
                in_string = true;
                string_char = ch;
                current.push(ch);
            }
            c if in_string && c == string_char => {
                in_string = false;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(ch);
            }
            ';' if !in_string && paren_depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    if let Some(colon) = trimmed.find(':') {
                        let prop = trimmed[..colon].trim().to_lowercase();
                        let val = trimmed[colon + 1..].trim().to_string();
                        declarations.push((prop, val));
                    }
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        if let Some(colon) = trimmed.find(':') {
            let prop = trimmed[..colon].trim().to_lowercase();
            let val = trimmed[colon + 1..].trim().to_string();
            declarations.push((prop, val));
        }
    }

    declarations
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

fn ensure_web_safe_font_fallback(value: &str) -> String {
    let fonts: Vec<&str> = value.split(',').map(|f| f.trim()).collect();

    let has_safe_fallback = fonts.iter().any(|f| {
        let clean = f.trim_matches(|c| c == '"' || c == '\'').to_lowercase();
        WEB_SAFE_FONTS
            .iter()
            .any(|safe| safe.to_lowercase() == clean)
    });

    if has_safe_fallback {
        return value.to_string();
    }

    format!("{}, sans-serif", value)
}

thread_local! {
    static COLLECTED_ISSUES: RefCell<Vec<SanitizeIssue>> = const { RefCell::new(Vec::new()) };
}

fn detect_unsupported_tags(html: &str) -> Vec<(String, String)> {
    let mut unsupported = Vec::new();

    for cap in TAG_REGEX.captures_iter(html) {
        if let Some(tag_match) = cap.get(1) {
            let tag = tag_match.as_str().to_lowercase();
            if !ALLOWED_TAGS.contains(&tag.as_str()) {
                let reason = match tag.as_str() {
                    "!doctype" => "DOCTYPE declaration is not needed in email HTML",
                    "head" => "<head> section is ignored by most email clients",
                    "title" => "<title> is not displayed in email clients",
                    "meta" => "<meta> tags are ignored in email HTML",
                    "link" => "<link> tags for external stylesheets are not supported",
                    "script" => "<script> tags are removed for security",
                    "style" => "<style> tags have limited support, use inline styles instead",
                    "iframe" => "<iframe> is not supported in emails",
                    "form" => "<form> elements have very limited support",
                    "input" => "<input> elements are not supported",
                    "button" => "<button> is not supported, use styled <a> instead",
                    "nav" => "<nav> semantic tag is not supported",
                    "header" => "<header> semantic tag is not supported",
                    "footer" => "<footer> semantic tag is not supported",
                    "article" => "<article> semantic tag is not supported",
                    "section" => "<section> semantic tag is not supported",
                    "aside" => "<aside> semantic tag is not supported",
                    "main" => "<main> semantic tag is not supported",
                    "figure" => "<figure> semantic tag is not supported",
                    "figcaption" => "<figcaption> semantic tag is not supported",
                    _ => "This tag is not supported by most email clients",
                };
                unsupported.push((tag, reason.to_string()));
            }
        }
    }

    unsupported.sort();
    unsupported.dedup_by(|a, b| a.0 == b.0);
    unsupported
}

fn get_issue_details(prop: &str) -> (String, IssueSeverity) {
    match prop {
        "position" | "fixed" | "absolute" | "sticky" => (
            "position property is not supported by most email clients".to_string(),
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
        "transition" | "transform" | "perspective" => (
            "CSS transitions/transforms are not supported in email clients".to_string(),
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

pub fn create_email_sanitizer<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);

    builder.tag_attributes(maplit::hashmap![
        "a" => hashset!["href", "title", "target", "style"],
        "img" => hashset!["src", "alt", "width", "height", "style"],
        "table" => hashset!["width", "height", "border", "cellpadding", "cellspacing", "align", "bgcolor", "style"],
        "td" => hashset!["width", "height", "align", "valign", "bgcolor", "colspan", "rowspan", "style"],
        "th" => hashset!["width", "height", "align", "valign", "bgcolor", "colspan", "rowspan", "style"],
        "tr" => hashset!["align", "valign", "bgcolor", "style"],
        "div" => hashset!["align", "style"],
        "span" => hashset!["style"],
        "p" => hashset!["align", "style"],
        "font" => hashset!["color", "face", "size", "style"],
        "hr" => hashset!["width", "size", "color", "style"],
        "col" => hashset!["width", "span", "style"],
        "colgroup" => hashset!["width", "span", "style"]
    ]);

    builder.generic_attributes(hashset!["style", "class", "id", "align", "valign"]);
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

fn inline_css_styles(html: &str) -> String {
    css_inline::inline(html).unwrap_or_else(|_| html.to_string())
}

pub fn sanitize_email_html(html: &str) -> String {
    let inlined = inline_css_styles(html);
    let builder = create_email_sanitizer();
    builder.clean(&inlined).to_string()
}

fn create_sanitizer_with_tracking<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);

    builder.tag_attributes(maplit::hashmap![
        "a" => hashset!["href", "title", "target", "style"],
        "img" => hashset!["src", "alt", "width", "height", "style"],
        "table" => hashset!["width", "height", "border", "cellpadding", "cellspacing", "align", "bgcolor", "style"],
        "td" => hashset!["width", "height", "align", "valign", "bgcolor", "colspan", "rowspan", "style"],
        "th" => hashset!["width", "height", "align", "valign", "bgcolor", "colspan", "rowspan", "style"],
        "tr" => hashset!["align", "valign", "bgcolor", "style"],
        "div" => hashset!["align", "style"],
        "span" => hashset!["style"],
        "p" => hashset!["align", "style"],
        "font" => hashset!["color", "face", "size", "style"],
        "hr" => hashset!["width", "size", "color", "style"],
        "col" => hashset!["width", "span", "style"],
        "colgroup" => hashset!["width", "span", "style"]
    ]);

    builder.generic_attributes(hashset!["style", "class", "id", "align", "valign"]);
    builder.link_rel(Some("noopener noreferrer"));

    builder.attribute_filter(|_element: &str, attribute: &str, value: &'_ str| {
        if attribute == "style" {
            let result = sanitize_style_attribute(value);

            COLLECTED_ISSUES.with(|issues| {
                let mut issues = issues.borrow_mut();
                for prop in &result.removed_properties {
                    let (reason, severity) = get_issue_details(prop);
                    issues.push(SanitizeIssue {
                        property: prop.clone(),
                        reason,
                        severity,
                    });
                }
                if result.added_font_fallback {
                    issues.push(SanitizeIssue {
                        property: "font-family".to_string(),
                        reason: "Added web-safe font fallback".to_string(),
                        severity: IssueSeverity::Info,
                    });
                }
            });

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

static IMPORT_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"@import\s+[^;]+;?"#).expect("Invalid @import regex pattern")
});

static FONT_FACE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"@font-face\s*\{[^}]*\}").expect("Invalid @font-face regex pattern")
});

pub fn auto_fix_email_html(html: &str) -> String {
    let without_imports = IMPORT_REGEX.replace_all(html, "");
    let without_font_faces = FONT_FACE_REGEX.replace_all(&without_imports, "");
    let inlined = inline_css_styles(&without_font_faces);
    let builder = create_email_sanitizer();
    let sanitized = builder.clean(&inlined).to_string();

    cleanup_html_whitespace(&sanitized)
}

fn cleanup_html_whitespace(html: &str) -> String {
    html.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn sanitize_email_html_with_details(html: &str) -> SanitizeResult {
    COLLECTED_ISSUES.with(|issues| issues.borrow_mut().clear());

    let unsupported_tags = detect_unsupported_tags(html);

    let inlined = inline_css_styles(html);

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

    let builder = create_sanitizer_with_tracking();
    let sanitized = builder.clean(&inlined).to_string();

    let issues = COLLECTED_ISSUES.with(|issues| issues.borrow().clone());

    SanitizeResult {
        html: sanitized,
        issues,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizeIssue {
    pub property: String,
    pub reason: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SanitizeResult {
    pub html: String,
    pub issues: Vec<SanitizeIssue>,
}
