//! Stage 2: Pseudo-element expansion

use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::PseudoRule;
use html5ever::QualName;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use markup5ever::{namespace_url, ns};
use regex::Regex;
use std::collections::HashSet;

/// HTML-escape content for safe interpolation
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn find_matching_brace(css: &str, start: usize) -> Option<usize> {
    let mut count = 1;
    let mut j = start;
    let mut in_string = false;
    let mut string_quote = '\0';
    let mut escaped = false;
    let mut in_comment = false;

    while j < css.len() && count > 0 {
        let ch = css.as_bytes()[j] as char;

        if escaped {
            escaped = false;
            j += 1;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            j += 1;
            continue;
        }

        if in_comment {
            if ch == '*' && j + 1 < css.len() && css.as_bytes()[j + 1] as char == '/' {
                in_comment = false;
                j += 2;
                continue;
            }
            j += 1;
            continue;
        }

        if in_string {
            if ch == string_quote {
                in_string = false;
            }
            j += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            string_quote = ch;
            j += 1;
            continue;
        }

        if ch == '/' && j + 1 < css.len() && css.as_bytes()[j + 1] as char == '*' {
            in_comment = true;
            j += 2;
            continue;
        }

        match ch {
            '{' => count += 1,
            '}' => count -= 1,
            _ => {}
        }
        j += 1;
    }

    if count == 0 {
        Some(j)
    } else {
        None
    }
}

pub fn expand_pseudo_elements_dom(document: &NodeRef) {
    let mut all_rules = Vec::new();
    let mut style_nodes = Vec::new();

    // First pass: collect all style nodes and their rules
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            if element.name.local.to_string().to_lowercase() == "style" {
                style_nodes.push(node.clone());
            }
        }
    }

    for style_node in &style_nodes {
        let css_body = style_node.text_contents();
        let (rules, cleaned_css) = parse_pseudo_rules(&css_body);

        let mut new_css_rules = String::new();
        for rule in &rules {
            if !rule.style.is_empty() {
                new_css_rules.push_str(&format!(
                    "\n.{} {{ {} }}\n",
                    rule.class_for_style, rule.style
                ));
            }
        }

        for child in style_node.children() {
            child.detach();
        }
        style_node.append(NodeRef::new_text(format!("{}{}", cleaned_css, new_css_rules)));

        all_rules.extend(rules);
    }

    if all_rules.is_empty() {
        return;
    }

    // Merge duplicate pseudo rules for the same class+pseudo kind.
    let all_rules = merge_pseudo_rules(all_rules);

    for rule in &all_rules {
        let span_class = rule.class_for_style.clone();
        let span_content = if rule.content.is_empty() {
            String::new()
        } else {
            html_escape(&rule.content)
        };

        let span = NodeRef::new_element(
            QualName::new(None, ns!(html), "span".into()),
            vec![(
                kuchiki::ExpandedName::new(ns!(), "class"),
                kuchiki::Attribute {
                    prefix: None,
                    value: span_class.to_string(),
                },
            )],
        );
        if !span_content.is_empty() {
            span.append(NodeRef::new_text(span_content));
        }

        for node in document.descendants() {
            if let Some(element) = node.as_element() {
                let attrs = element.attributes.borrow();
                if let Some(class_attr) = attrs.get("class") {
                    if class_attr.split_whitespace().any(|t| t == rule.class) {
                        drop(attrs);
                        if rule.is_before {
                            node.prepend(span.clone());
                        } else {
                            node.append(span.clone());
                        }
                    }
                }
            }
        }
    }
}

pub fn expand_pseudo_elements(html: &str) -> String {
    let document = kuchiki::parse_html().one(html);
    expand_pseudo_elements_dom(&document);
    document.to_string()
}

fn parse_pseudo_rules(css: &str) -> (Vec<PseudoRule>, String) {
    // ── Step 1: brace-counting parser to handle nested blocks and at-rules ─────────────────
    let mut expanded_css = String::new();
    let mut i = 0;

    let mut pseudo_selectors: Vec<(String, String, usize, usize)> = Vec::new();

    while i < css.len() {
        // Find the next opening brace
        if let Some(open_offset) = css[i..].find('{') {
            let selector_start = i;
            let brace_start = i + open_offset;
            let selector_part = css[selector_start..brace_start].trim();

            // Skip at-rules (like @media) - treat as opaque blocks
            if selector_part.starts_with("@") {
                if let Some(end) = find_matching_brace(css, brace_start + 1) {
                    // Keep the at-rule unchanged
                    expanded_css.push_str(&css[selector_start..end]);
                    expanded_css.push('\n');
                    i = end;
                } else {
                    // Malformed, skip selector
                    expanded_css.push_str(&css[selector_start..brace_start]);
                    i = brace_start;
                }
                continue;
            }

            if let Some(end) = find_matching_brace(css, brace_start + 1) {
                // Found complete rule block
                let body = css[brace_start + 1..end - 1].trim();
                let _expanded_start = expanded_css.len();

                let has_pseudo =
                    selector_part.contains("::before") || selector_part.contains("::after");
                if !has_pseudo {
                    // Keep unchanged
                    expanded_css.push_str(&css[selector_start..end]);
                    expanded_css.push('\n');
                } else {
                    // Expand grouped selectors
                    for fragment in selector_part.split(',') {
                        let fragment = fragment.trim();
                        if fragment.is_empty() {
                            continue;
                        }
                        let rule_start = expanded_css.len();
                        let rule_text = format!("{} {{\n{}\n}}\n", fragment, body);
                        expanded_css.push_str(&rule_text);
                        if fragment.contains("::before") || fragment.contains("::after") {
                            pseudo_selectors.push((
                                fragment.to_string(),
                                body.to_string(),
                                rule_start,
                                expanded_css.len(),
                            ));
                        }
                    }
                }
                i = end;
            } else {
                // Malformed - unmatched brace
                expanded_css.push_str(&css[selector_start..brace_start]);
                i = brace_start;
            }
        } else {
            // No more braces - copy rest
            expanded_css.push_str(&css[i..]);
            break;
        }
    }

    // ── Step 2: merge CSS bodies for same class+pseudo, THEN extract rules ─
    let pseudo_re = Regex::new(r"^\s*\.([\w-]+)::(before|after)\s*$").unwrap();

    // Group by (class, pseudo_kind) and merge CSS bodies
    let mut grouped: std::collections::HashMap<(String, bool), Vec<String>> =
        std::collections::HashMap::new();

    for (selector, body, _, _) in &pseudo_selectors {
        if let Some(caps) = pseudo_re.captures(selector) {
            let class = caps[1].to_string();
            let is_before = &caps[2] == "before";
            grouped
                .entry((class, is_before))
                .or_default()
                .push(body.to_string());
        }
    }

    let mut rules = Vec::new();

    // Positioning props that don't work in email - strip from pseudo spans
    const PSEUDO_STRIP_PROPS: &[&str] = &[
        "position",
        "top",
        "left",
        "right",
        "bottom",
        "inset",
        "z-index",
        "transform",
        "transform-origin",
    ];

    for ((class, is_before), bodies) in &grouped {
        // Merge all CSS declarations for this class+pseudo, later overrides earlier
        let mut merged_decls: Vec<(String, String)> = Vec::new();
        for body in bodies {
            for (prop, val) in parse_css_declarations(body) {
                if let Some(pos) = merged_decls.iter().position(|(p, _)| *p == prop) {
                    merged_decls[pos] = (prop, val);
                } else {
                    merged_decls.push((prop, val));
                }
            }
        }

        // Extract content and build style
        let mut content = String::new();
        let mut has_content_decl = false;
        let mut style_parts: Vec<String> = Vec::new();
        let mut has_display = false;

        for (prop, val) in &merged_decls {
            if prop.eq_ignore_ascii_case("content") {
                has_content_decl = true;
                content = val
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
            } else if PSEUDO_STRIP_PROPS.contains(&prop.as_str()) {
                // Skip positioning props - can't use them in email
                continue;
            } else {
                style_parts.push(format!("{}: {}", prop, val));
                if prop.eq_ignore_ascii_case("display") {
                    has_display = true;
                }
            }
        }

        if !has_display {
            style_parts.push("display: inline-block".to_string());
        }

        if !has_content_decl {
            continue;
        }

        let pseudo_kind = if *is_before { "before" } else { "after" };
        let class_for_style = format!("__pseudo_{}__{}", class, pseudo_kind);
        let style_body = style_parts.join("; ");

        rules.push(PseudoRule {
            class: class.clone(),
            is_before: *is_before,
            content,
            style: style_body,
            class_for_style,
        });
    }

    // Remove only the pseudo rules that we actually expanded into `rules`.
    let mut expanded_selectors: HashSet<String> = HashSet::new();
    for rule in &rules {
        let sel = format!(
            ".{}::{}",
            rule.class,
            if rule.is_before { "before" } else { "after" }
        );
        expanded_selectors.insert(sel);
    }

    let mut ranges_to_remove: Vec<(usize, usize)> = pseudo_selectors
        .into_iter()
        .filter(|(selector, _, _, _)| expanded_selectors.contains(selector))
        .map(|(_, _, start, end)| (start, end))
        .collect();
    ranges_to_remove.sort_by(|a, b| b.0.cmp(&a.0));

    let mut cleaned_css = expanded_css;
    for (start, end) in ranges_to_remove {
        if start < cleaned_css.len() && end <= cleaned_css.len() {
            cleaned_css.replace_range(start..end, "");
        }
    }

    (rules, cleaned_css)
}

/// Merge pseudo rules that target the same class + pseudo kind (before/after)
fn merge_pseudo_rules(rules: Vec<PseudoRule>) -> Vec<PseudoRule> {
    let mut merged: Vec<PseudoRule> = Vec::new();

    for rule in rules {
        let key_matches = merged.iter().position(|existing| {
            existing.class == rule.class && existing.is_before == rule.is_before
        });

        if let Some(idx) = key_matches {
            // Merge: parse both style strings, later properties override
            let existing_decls = parse_css_declarations(&merged[idx].style);
            let new_decls = parse_css_declarations(&rule.style);

            let mut combined: Vec<(String, String)> = existing_decls;
            for (prop, val) in new_decls {
                if let Some(pos) = combined.iter().position(|(p, _)| *p == prop) {
                    combined[pos] = (prop, val);
                } else {
                    combined.push((prop, val));
                }
            }

            merged[idx].style = combined
                .iter()
                .map(|(p, v)| format!("{}: {}", p, v))
                .collect::<Vec<_>>()
                .join("; ");

            // If the new rule has content and the existing one doesn't, take it
            if !rule.content.is_empty() && merged[idx].content.is_empty() {
                merged[idx].content = rule.content;
            }
        } else {
            merged.push(rule);
        }
    }

    merged
}
