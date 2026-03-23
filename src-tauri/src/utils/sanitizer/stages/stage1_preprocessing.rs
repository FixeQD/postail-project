//! Stage 1: Preprocessing — CSS variable resolution and body normalization.

use std::collections::HashMap;

use html5ever::QualName;
use kuchikiki::traits::TendrilSink;
use kuchikiki::NodeRef;
use markup5ever::{namespace_url, ns};

// ---------------------------------------------------------------------------
// CSS variable resolution
// ---------------------------------------------------------------------------

/// Resolve `var(--foo)` references in all `<style>` blocks and inline
/// `style=""` attributes using values declared in `:root { … }`.
pub fn resolve_css_variables_dom(document: &NodeRef) {
    let mut css_content = String::new();
    let mut style_nodes = Vec::new();

    for node in document.descendants() {
        if let Some(el) = node.as_element() {
            if el.name.local.as_ref().eq_ignore_ascii_case("style") {
                css_content.push_str(&node.text_contents());
                style_nodes.push(node.clone());
            }
        }
    }

    let vars = parse_css_variables(&css_content);
    if vars.is_empty() {
        return;
    }

    for style_node in style_nodes {
        let css = style_node.text_contents();
        let resolved = resolve_var_refs(&css, &vars);
        for child in style_node.children() {
            child.detach();
        }
        style_node.append(NodeRef::new_text(resolved));
    }

    for node in document.descendants() {
        if let Some(el) = node.as_element() {
            let mut attrs = el.attributes.borrow_mut();
            if let Some(style) = attrs.get("style").map(|s| s.to_string()) {
                let resolved = resolve_var_refs(&style, &vars);
                if resolved != style {
                    attrs.insert("style", resolved);
                }
            }
        }
    }
}

fn parse_css_variables(css: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    let re = regex::Regex::new(r"(?s):root\s*\{([^}]*)\}").expect("invalid :root regex");
    if let Some(cap) = re.captures(css) {
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

/// Substitute `var(--name, fallback)` references, up to 8 nesting levels.
fn resolve_var_refs(value: &str, vars: &HashMap<String, String>) -> String {
    let re =
        regex::Regex::new(r"var\(\s*(--[a-zA-Z0-9_-]+)\s*(?:,\s*((?:[^()]*|\([^()]*\))*))?\ ?\)")
            .unwrap();

    let mut result = value.to_string();
    for _ in 0..8 {
        let next = re
            .replace_all(&result, |caps: &regex::Captures| {
                let name = &caps[1];
                if let Some(v) = vars.get(name) {
                    v.clone()
                } else if let Some(fb) = caps.get(2) {
                    fb.as_str().trim().to_string()
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

// ---------------------------------------------------------------------------
// Body styles extraction
// ---------------------------------------------------------------------------

/// Extract inheritable styles from the CSS `body { … }` rule so they can be
/// applied to the replacement `<div>` wrapper.
pub fn extract_body_styles_dom(document: &NodeRef) -> String {
    let mut css = String::new();
    for node in document.descendants() {
        if let Some(el) = node.as_element() {
            if el.name.local.as_ref().eq_ignore_ascii_case("style") {
                css.push_str(&node.text_contents());
            }
        }
    }
    extract_body_styles_from_css(&css)
}

pub fn extract_body_styles_from_css(css: &str) -> String {
    let re = regex::Regex::new(r"(?s)(?:html,\s*)?body\s*\{([^}]+)\}").unwrap();
    let Some(cap) = re.captures(css) else {
        return String::new();
    };

    cap[1]
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|s| !s.contains("animation"))
        .filter(|s| !s.contains("transform"))
        .filter(|s| !s.contains("filter"))
        .filter(|s| !s.contains("position"))
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

// ---------------------------------------------------------------------------
// Body → div replacement
// ---------------------------------------------------------------------------

/// Replace `<body>` with a `<div>` carrying the same styles. Ammonia strips
/// `<body>` anyway, so this preserves any background/color declarations.
pub fn replace_body_with_div_dom(document: &NodeRef, body_styles: String) {
    for node in document.descendants() {
        if let Some(el) = node.as_element() {
            if !el.name.local.as_ref().eq_ignore_ascii_case("body") {
                continue;
            }

            let attrs = el.attributes.borrow();
            let mut style = attrs.get("style").unwrap_or("").to_string();
            if !body_styles.is_empty() {
                style = if style.is_empty() {
                    body_styles.clone()
                } else {
                    format!("{}; {}", body_styles, style)
                };
            }

            let div = NodeRef::new_element(QualName::new(None, ns!(html), "div".into()), None);
            if let Some(div_el) = div.as_element() {
                let mut div_attrs = div_el.attributes.borrow_mut();
                for (key, attr) in attrs.map.iter() {
                    if key.local.as_ref() != "style" {
                        div_attrs.insert(key.local.clone(), attr.value.clone());
                    }
                }
                if !style.is_empty() {
                    div_attrs.insert("style", style);
                }
            }

            for child in node.children() {
                div.append(child.clone());
            }
            node.insert_before(div);
            node.detach();
            break;
        }
    }
}

pub fn resolve_css_variables(html: &str) -> String {
    let doc = kuchikiki::parse_html().one(html);
    resolve_css_variables_dom(&doc);
    doc.to_string()
}
