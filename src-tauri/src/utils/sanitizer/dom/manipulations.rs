//! DOM manipulation utilities

use kuchiki::NodeRef;

use crate::utils::sanitizer::config::ALLOWED_TAGS;
use crate::utils::sanitizer::config::TAG_REGEX;
use crate::utils::sanitizer::css::parser::parse_css_declarations;

/// Remove content tags (head, script, style, title, noscript) from document
pub fn strip_content_tags_dom(document: &NodeRef) {
    let tags_to_remove: std::collections::HashSet<&str> =
        ["head", "script", "style", "title", "noscript"]
            .iter()
            .cloned()
            .collect();

    let mut nodes_to_remove: Vec<NodeRef> = Vec::new();

    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();
            if tags_to_remove.contains(tag_name.as_str()) {
                nodes_to_remove.push(node.clone());
            }
        }
    }

    for node in nodes_to_remove {
        node.detach();
    }
}

/// Detect unsupported HTML tags
pub fn detect_unsupported_tags(html: &str) -> Vec<(String, String)> {
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

/// Mark elements with position property for dead element removal
pub fn mark_positioned_elements_dom(document: &NodeRef) {
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let attrs = element.attributes.borrow();
            if let Some(style) = attrs.get("style") {
                let has_position = parse_css_declarations(style)
                    .iter()
                    .any(|(p, _)| p == "position");

                if has_position {
                    drop(attrs);
                    let mut attrs_mut = element.attributes.borrow_mut();
                    attrs_mut.insert("data-dead-if-empty", "".to_string());
                }
            }
        }
    }
}

/// Check if element has visual content (background, borders, dimensions)
pub fn has_visual_content_dom(element: &kuchiki::ElementData) -> bool {
    let attrs = element.attributes.borrow();

    if let Some(style) = attrs.get("style") {
        let styles = parse_css_declarations(style);
        let visual_props = [
            "background",
            "background-color",
            "background-image",
            "border",
            "border-color",
            "border-width",
            "border-top",
            "border-bottom",
            "border-left",
            "border-right",
            "box-shadow",
        ];

        let mut has_dimensions = false;
        let mut has_border_radius = false;

        for (prop, val) in &styles {
            let is_empty_val = val == "0"
                || val == "none"
                || val == "transparent"
                || val == "auto"
                || val == "0px"
                || val == "0%"
                || val.is_empty();

            if visual_props.contains(&prop.as_str()) && !is_empty_val {
                return true;
            }

            // Track dimensions separately
            if matches!(
                prop.as_str(),
                "width" | "height" | "min-width" | "min-height"
            ) && !is_empty_val
            {
                has_dimensions = true;
            }

            if prop == "border-radius" && !is_empty_val {
                has_border_radius = true;
            }
        }

        // A sized element with border-radius is likely a decorative shape
        if has_dimensions && has_border_radius {
            return true;
        }
    }

    if attrs.get("background").is_some() || attrs.get("bgcolor").is_some() {
        return true;
    }

    false
}

/// Remove dead (empty) elements from document
pub fn strip_dead_elements_dom(document: &NodeRef) {
    for _ in 0..3 {
        let mut nodes_to_remove: Vec<NodeRef> = Vec::new();
        let mut nodes_to_cleanup: Vec<NodeRef> = Vec::new();

        for node in document.descendants() {
            if let Some(element) = node.as_element() {
                let attrs = element.attributes.borrow();
                if attrs.get("data-dead-if-empty").is_some() {
                    drop(attrs);

                    let has_text = !node.text_contents().trim().is_empty();
                    // Only count actual element children, not whitespace text nodes
                    let has_element_children = node.children().any(|child| {
                        if child.as_element().is_some() {
                            return true;
                        }
                        if let Some(text) = child.as_text() {
                            return !text.borrow().trim().is_empty();
                        }
                        false
                    });
                    let has_visual = has_visual_content_dom(element);

                    if !has_text && !has_element_children && !has_visual {
                        nodes_to_remove.push(node.clone());
                    } else {
                        nodes_to_cleanup.push(node.clone());
                    }
                }
            }
        }

        if nodes_to_remove.is_empty() {
            for node in nodes_to_cleanup {
                if let Some(element) = node.as_element() {
                    let mut attrs = element.attributes.borrow_mut();
                    attrs.remove("data-dead-if-empty");
                }
            }
            break;
        }

        for node in nodes_to_remove {
            node.detach();
        }

        for node in nodes_to_cleanup {
            if let Some(element) = node.as_element() {
                let mut attrs = element.attributes.borrow_mut();
                attrs.remove("data-dead-if-empty");
            }
        }
    }
}
