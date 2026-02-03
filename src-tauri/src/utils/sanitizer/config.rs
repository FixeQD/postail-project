use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::LazyLock;

use ammonia::Builder;
use maplit::{hashmap, hashset};

use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::{IssueSeverity, SanitizeIssue, StyleSanitizeResult};

pub static TAG_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"<([a-zA-Z][a-zA-Z0-9]*)[^>]*>").expect("Invalid regex pattern")
});

pub const DANGEROUS_CSS_PROPS: &[&str] = &[
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

/// Creates an HTML sanitizer Builder preconfigured for email-safe output.
///
/// The returned `Builder` is configured with a restricted tag set, per-element
/// allowed attributes, common generic attributes, `rel="noopener noreferrer"`
/// for links, and an attribute filter that sanitizes `style` attributes using
/// the email-safe style sanitizer.
///
/// # Examples
///
/// ```
/// let _builder = create_email_sanitizer();
/// ```
pub fn create_email_sanitizer<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);

    builder.tag_attributes(hashmap! [
        "a" => hashset!["href", "title", "target", "style"],
        "body" => hashset!["style", "bgcolor", "text", "link", "vlink", "alink"],
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
        "html" => hashset!["lang", "style"],
        "col" => hashset!["width", "span", "style"],
        "colgroup" => hashset!["width", "span", "style"]
    ]);

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

/// Creates a sanitizer Builder configured for email-safe HTML that sanitizes style attributes and records removed CSS properties and font-fallback decisions.
///
/// The returned Builder is preconfigured with a restricted set of allowed tags and per-element allowed attributes suitable for email rendering, generic attributes (including `style`), `link_rel` set to `"noopener noreferrer"`, and an attribute filter that sanitizes `style` values while pushing sanitization issues into the thread-local issue collection.
///
/// # Examples
///
/// ```
/// let builder = create_sanitizer_with_tracking();
/// // Use `builder` to sanitize HTML; style-related removals will be tracked.
/// ```
pub fn create_sanitizer_with_tracking<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);

    builder.tag_attributes(hashmap! [
        "a" => hashset!["href", "title", "target", "style"],
        "body" => hashset!["style", "bgcolor", "text", "link", "vlink", "alink"],
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
        "html" => hashset!["lang", "style"],
        "col" => hashset!["width", "span", "style"],
        "colgroup" => hashset!["width", "span", "style"]
    ]);

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

/// Sanitizes a CSS `style` attribute for email-compatible rendering.
///
/// The returned `StyleSanitizeResult` contains:
/// - `cleaned_style`: the sanitized CSS declarations joined into a single string,
/// - `removed_properties`: a list of properties that were stripped because they are considered dangerous for email clients,
/// - `added_font_fallback`: `true` if a web-safe font fallback was appended for `font-family`.
///
/// # Examples
///
/// ```
/// let res = sanitize_style_attribute("position: absolute; color: red; font-family: CustomFont, Arial;");
/// assert!(res.removed_properties.contains(&"position".to_string()));
/// assert!(res.cleaned_style.contains("color: red"));
/// assert!(res.added_font_fallback);
/// ```
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

/// Sanitizes a CSS `style` string for email usage while recording removed properties and issues.
///
/// Parses the provided CSS declarations, removes properties considered dangerous for email clients,
/// records each removed property and a corresponding `SanitizeIssue` into `COLLECTED_ISSUES`,
/// ensures `font-family` values include a web-safe fallback (setting `added_font_fallback` when one
/// is appended), and returns a `StyleSanitizeResult` containing the cleaned style string and
/// metadata about removals and fallback additions.
///
/// # Examples
///
/// ```
/// let result = sanitize_style_attribute_with_tracking("color: red; position: absolute; font-family: FooBar;");
/// assert!(result.cleaned_style.contains("color: red"));
/// assert!(result.removed_properties.iter().any(|p| p == "position"));
/// assert!(result.added_font_fallback);
/// ```
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

/// Detects whether a CSS property name is considered dangerous for email sanitization.
///
/// Checks the property name case-insensitively against the configured list of dangerous
/// properties, disallows properties containing the keywords `expression` or `behavior`,
/// and treats vendor-prefixed variants as dangerous if their unprefixed form is dangerous.
///
/// # Returns
///
/// `true` if the property is considered dangerous, `false` otherwise.
///
/// # Examples
///
/// ```
/// assert!(is_dangerous_property("position"));
/// assert!(is_dangerous_property("-webkit-transform"));
/// assert!(is_dangerous_property("BEHAVIOR")); // keyword match is case-insensitive
/// assert!(!is_dangerous_property("color"));
/// ```
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

/// Maps common custom font family names to web-safe fallback stacks for email clients.
///
/// Strips surrounding quotes, lowercases the font name, and returns a safe font stack when a known
/// custom font is recognized; returns `None` if no mapping is available.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     map_custom_font_to_safe("Roboto"),
///     Some("Arial, Helvetica, sans-serif")
/// );
/// assert_eq!(map_custom_font_to_safe("\"Unknown Font\""), None);
/// ```
fn map_custom_font_to_safe(font: &str) -> Option<&'static str> {
    let clean = font.trim_matches(|c| c == '"' || c == '\'').to_lowercase();

    match clean.as_str() {
        // Serif fonts - map to Georgia
        "cormorant garamond" | "cormorant" | "garamond" | "playfair display" | "merriweather"
        | "libre baskerville" | "crimson text" | "eb garamond" | "pt serif" | "noto serif"
        | "source serif pro" | "alice" | "cardo" => Some("Georgia, 'Times New Roman', serif"),
        // Sans-serif fonts - map to Arial/Helvetica
        "inter" | "roboto" | "open sans" | "lato" | "montserrat" | "poppins" | "nunito"
        | "raleway" | "ubuntu" | "work sans" | "fira sans" | "source sans pro" | "pt sans"
        | "noto sans" => Some("Arial, Helvetica, sans-serif"),
        // Monospace fonts
        "fira code" | "jetbrains mono" | "source code pro" | "roboto mono" | "space mono"
        | "ubuntu mono" | "ibm plex mono" => Some("'Courier New', Courier, monospace"),
        _ => None,
    }
}

/// Ensure a `font-family` value includes a web-safe fallback or is mapped to a safe font stack.
///
/// If the first font name is recognized as a common non-standard font, returns a mapped, web-safe
/// font stack (e.g., maps certain serif/sans/monospace names to known safe stacks). If any font
/// in the provided list already matches a known web-safe font, returns the original value
/// unchanged. Otherwise, appends `, sans-serif` to the original value and returns that string.
///
/// # Examples
///
/// ```
/// // Known custom mapping (example)
/// let out = ensure_web_safe_font_fallback("Roboto");
/// assert_eq!(out, "Arial, Helvetica, sans-serif");
///
/// // Already contains a web-safe font -> unchanged
/// let out = ensure_web_safe_font_fallback("\"Times New Roman\", serif");
/// assert_eq!(out, "\"Times New Roman\", serif");
///
/// // No safe fonts -> append generic fallback
/// let out = ensure_web_safe_font_fallback("MyCustomFont, SomethingElse");
/// assert_eq!(out, "MyCustomFont, SomethingElse, sans-serif");
/// ```
fn ensure_web_safe_font_fallback(value: &str) -> String {
    let fonts: Vec<&str> = value.split(',').map(|f| f.trim()).collect();

    // Check if first font is a custom font that needs mapping
    if let Some(first) = fonts.first() {
        if let Some(mapped) = map_custom_font_to_safe(first) {
            return mapped.to_string();
        }
    }

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

/// Provide a human-readable reason and a severity level for removing or altering a CSS property.
///
/// # Parameters
///
/// - `prop`: the CSS property name to evaluate (e.g., `"position"`, `"font-family"`, `"animation-name"`).
///
/// # Returns
///
/// A tuple `(message, severity)` where `message` explains why the property was removed or modified and `severity` indicates the importance of the issue.
///
/// # Examples
///
/// ```
/// let (msg, sev) = get_issue_details("position");
/// assert_eq!(sev, IssueSeverity::Warning);
/// assert!(msg.contains("position"));
/// ```
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