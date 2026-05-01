//! Pseudo-element DOM expansion logic.

use kuchikiki::traits::*;
use kuchikiki::NodeRef;
use markup5ever::{namespace_url, ns, QualName};

use crate::stages::pseudo::{html_escape, merge_pseudo_rules, parse_pseudo_rules};

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
        style_node.append(NodeRef::new_text(format!(
            "{}{}",
            cleaned_css, new_css_rules
        )));

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
                kuchikiki::ExpandedName::new(ns!(), "class"),
                kuchikiki::Attribute {
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
    let document = kuchikiki::parse_html().one(html).document_node;
    expand_pseudo_elements_dom(&document);
    document.to_string()
}
