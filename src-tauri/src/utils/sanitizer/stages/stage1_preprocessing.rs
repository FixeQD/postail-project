use std::collections::HashMap;

use html5ever::QualName;
use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;
use markup5ever::{namespace_url, ns};

pub fn resolve_css_variables_dom(document: &NodeRef) {
    let mut css_content = String::new();
    let mut style_nodes = Vec::new();

    // Collect all style blocks to find :root variables
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            if element.name.local.to_string().to_lowercase() == "style" {
                css_content.push_str(&node.text_contents());
                style_nodes.push(node.clone());
            }
        }
    }

    let vars = parse_css_variables(&css_content);
    if vars.is_empty() {
        return;
    }

    // Resolve variables in <style> blocks
    for style_node in style_nodes {
        let css = style_node.text_contents();
        let resolved = resolve_var_refs(&css, &vars);
        for child in style_node.children() {
            child.detach();
        }
        style_node.append(NodeRef::new_text(resolved));
    }

    // Resolve variables in style="" attributes
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let mut attrs = element.attributes.borrow_mut();
            if let Some(style) = attrs.get("style").map(|s| s.to_string()) {
                let resolved = resolve_var_refs(&style, &vars);
                if resolved != style {
                    attrs.insert("style", resolved);
                }
            }
        }
    }
}

pub fn resolve_css_variables(html: &str) -> String {
    let document = kuchiki::parse_html().one(html);
    resolve_css_variables_dom(&document);
    document.to_string()
}

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

pub fn extract_body_styles_dom(document: &NodeRef) -> String {
    let mut css_content = String::new();
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            if element.name.local.to_string().to_lowercase() == "style" {
                css_content.push_str(&node.text_contents());
            }
        }
    }
    extract_body_styles_from_css(&css_content)
}

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
