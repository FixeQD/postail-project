use std::collections::HashMap;

use html5ever::QualName;
use kuchiki::NodeRef;
use markup5ever::{namespace_url, ns};

/// Resolve CSS custom property references (`var(--name[, fallback])`) in an HTML string.
///
/// This function parses CSS custom properties defined under `:root` and substitutes `var(...)` references
/// found inside `<style>` tags and inline `style="..."` attributes. If no custom properties are present,
/// the input HTML is returned unchanged. Nested references are resolved up to the implementation's depth,
/// and fallback values are used when a variable is not defined.
///
/// # Examples
///
/// ```
/// let html = r#"
/// <style>:root { --main: red; } p { color: var(--main); }</style>
/// <p style="background: var(--main, blue);">Hello</p>
/// "#;
/// let out = resolve_css_variables(html);
/// assert!(out.contains("color: red"));
/// assert!(out.contains(r#"style="background: red""#));
/// ```
pub fn resolve_css_variables(html: &str) -> String {
    let vars = parse_css_variables(html);
    if vars.is_empty() {
        return html.to_string();
    }

    let style_re = regex::Regex::new(r"(?s)(<style[^>]*>)(.*?)(</style>)").unwrap();
    let after_style = style_re
        .replace_all(html, |caps: &regex::Captures| {
            format!(
                "{}{}{}",
                &caps[1],
                resolve_var_refs(&caps[2], &vars),
                &caps[3]
            )
        })
        .to_string();

    let inline_style_re = regex::Regex::new(r#"style="([^"]*)"#).unwrap();
    inline_style_re
        .replace_all(&after_style, |caps: &regex::Captures| {
            let resolved = resolve_var_refs(&caps[1], &vars);
            format!(r#"style="{}""#, resolved)
        })
        .to_string()
}

/// Extracts CSS custom properties declared in a `:root` rule into a map.
///
/// Parses the first `:root { ... }` block found in `html` and returns a `HashMap` where
/// keys are the custom property names (including the leading `--`) and values are the
/// corresponding trimmed property values. Declarations without a colon or properties
/// that do not start with `--` are ignored.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// let html = r#"
/// :root {
///   --main-color: #ff0000;
///   --gap: 10px;
///   color: black;
/// }
/// "#;
///
/// let vars = parse_css_variables(html);
/// assert_eq!(vars.get("--main-color"), Some(&"#ff0000".to_string()));
/// assert_eq!(vars.get("--gap"), Some(&"10px".to_string()));
/// assert_eq!(vars.get("color"), None);
/// ```
fn parse_css_variables(html: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    let root_re = regex::Regex::new(r"(?s):root\s*\{([^}]*)\}").expect("invalid :root regex");
    if let Some(cap) = root_re.captures(html) {
        for decl in cap[1].split(';') {
            let decl = decl.trim();
            if let Some(colon) = decl.find(':') {
                let prop = decl[..colon].trim();
                let val = decl[colon + 1..].trim();
                if prop.starts_with("--") {
                    vars.insert(prop.to_string(), val.to_string());
                }
            }
        }
    }
    vars
}

/// Resolve CSS `var(--name[, fallback])` references inside a string using the provided custom-property map.
///
/// Resolves `var(...)` occurrences by replacing each with the corresponding value from `vars` when present,
/// or with the provided fallback when the variable is missing. Unresolved `var(...)` expressions are left unchanged.
///
/// # Returns
///
/// A `String` with `var(...)` references substituted: variable values are used when present, fallbacks are used when provided, and original `var(...)` text is preserved when neither is available.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("--main".to_string(), "red".to_string());
///
/// let input = "color: var(--main, blue); background: var(--bg, white);";
/// let out = resolve_var_refs(input, &vars);
/// assert_eq!(out, "color: red; background: white;");
/// ```
fn resolve_var_refs(value: &str, vars: &HashMap<String, String>) -> String {
    let var_re =
        regex::Regex::new(r"var\(\s*(--[a-zA-Z0-9_-]+)\s*(?:,\s*((?:[^()]*|\([^()]*\))*))?\)")
            .unwrap();
    let mut result = value.to_string();
    for _ in 0..8 {
        let next = var_re
            .replace_all(&result, |caps: &regex::Captures| {
                let name = &caps[1];
                if let Some(resolved) = vars.get(name) {
                    resolved.clone()
                } else if let Some(fallback) = caps.get(2) {
                    fallback.as_str().trim().to_string()
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();
        if next == result {
            break;
        }
        result = next;
    }
    result
}

/// Extracts and sanitizes CSS declarations for the document body from an HTML string.
///
/// Searches for a `body` (or `html, body`) rule and returns its declarations with
/// empty entries removed and any declarations containing `animation`, `transform`,
/// `filter`, or `position` excluded.
///
/// # Returns
///
/// A string of remaining declarations joined by `"; "`, or an empty string if no
/// body rule is found.
///
/// # Examples
///
/// ```
/// let html = "body { background: red; animation: slide 1s; color: blue; }";
/// assert_eq!(extract_body_styles_from_css(html), "background: red; color: blue");
/// ```
pub fn extract_body_styles_from_css(html: &str) -> String {
    let body_css_re = regex::Regex::new(r"(?s)(?:html,\s*)?body\s*\{([^}]+)\}").unwrap();

    if let Some(cap) = body_css_re.captures(html) {
        let declarations = &cap[1];
        let cleaned: Vec<String> = declarations
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter(|s| !s.contains("animation"))
            .filter(|s| !s.contains("transform"))
            .filter(|s| !s.contains("filter"))
            .filter(|s| !s.contains("position"))
            .map(|s| s.to_string())
            .collect();

        cleaned.join("; ")
    } else {
        String::new()
    }
}

/// Replaces the document's <body> element with a <div>, preserving attributes and children and merging provided body styles.
///
/// The supplied `body_styles` string is prepended to any existing inline `style` on the body (separated by `; `). All other attributes are copied from the original body to the new div. The body node is removed and replaced with the new div; the function stops after processing the first body element found.
///
/// - `document`: the DOM tree to modify.
/// - `body_styles`: CSS declarations to merge into the body's `style` attribute (e.g., `"margin: 0; background: white"`).
///
/// # Examples
///
/// ```
/// use kuchiki::traits::*;
/// use kuchiki::parse_html;
///
/// let html = r#"<html><head></head><body style="color: red" id="main">Hello<span>World</span></body></html>"#;
/// let document = parse_html().one(html);
///
/// // Merge additional body styles and replace body with a div
/// replace_body_with_div_dom(&document, "background: white".to_string());
///
/// let out = document.to_string();
/// assert!(out.contains("<div"));
/// assert!(out.contains(r#"id="main""#));
/// assert!(out.contains(r#"style="background: white; color: red""#) || out.contains(r#"style="background: white; color: red""#));
/// ```
pub fn replace_body_with_div_dom(document: &NodeRef, body_styles: String) {
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();
            if tag_name == "body" {
                let attrs = element.attributes.borrow();
                let mut style_attr = attrs
                    .get("style")
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                if !body_styles.is_empty() {
                    if !style_attr.is_empty() {
                        style_attr = format!("{}; {}", body_styles, style_attr);
                    } else {
                        style_attr = body_styles.clone();
                    }
                }

                let div = NodeRef::new_element(QualName::new(None, ns!(html), "div".into()), None);

                {
                    let div_element = div.as_element().unwrap();
                    let mut div_attrs = div_element.attributes.borrow_mut();

                    for (key, attr) in attrs.map.iter() {
                        let key_str = key.local.to_string();
                        if key_str != "style" {
                            div_attrs.insert(key.local.clone(), attr.value.clone());
                        }
                    }

                    if !style_attr.is_empty() {
                        div_attrs.insert("style", style_attr);
                    }
                }

                for child in node.children() {
                    div.append(child.clone());
                }

                node.insert_before(div.clone());
                node.detach();

                break;
            }
        }
    }
}