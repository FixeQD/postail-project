//! DOM manipulation utilities

use kuchiki::NodeRef;

use crate::utils::sanitizer::config::ALLOWED_TAGS;
use crate::utils::sanitizer::config::TAG_REGEX;
use crate::utils::sanitizer::css::parser::parse_css_declarations;

/// Remove content-related elements (head, script, style, title, noscript) from the document.
///
/// This function detaches matching nodes from the DOM in place so they are removed from `document`.
///
/// # Examples
///
/// ```
/// use kuchiki::traits::*;
/// let html = "<html><head><title>x</title><script>console.log(1)</script></head><body><p>hi</p></body></html>";
/// let document = kuchiki::parse_html().one(html);
/// strip_content_tags_dom(&document);
/// let serialized = document.to_string();
/// assert!(!serialized.contains("<head"));
/// assert!(serialized.contains("<p>hi</p>"));
/// ```
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

/// Identify HTML tags that are not in the allowed set and provide a human-readable reason for each.
///
/// Scans the provided HTML text for tag names, filters out those present in `ALLOWED_TAGS`,
/// and returns a sorted, deduplicated list of (tag, reason) pairs describing why each tag is unsupported.
///
/// # Examples
///
/// ```
/// let html = "<html><body><script>alert('x')</script><meta charset=\"utf-8\"></body></html>";
/// let unsupported = detect_unsupported_tags(html);
/// assert!(unsupported.contains(&("script".to_string(), "<script> tags are removed for security".to_string())));
/// assert!(unsupported.contains(&("meta".to_string(), "<meta> tags are ignored in email HTML".to_string())));
/// ```
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

/// Mark elements that declare a CSS `position` by adding a `data-dead-if-empty` attribute.
///
/// Scans the provided document's element descendants and sets `data-dead-if-empty` on any element
/// whose inline `style` contains a `position` declaration. The document is modified in place.
///
/// # Examples
///
/// ```
/// use kuchiki::traits::*;
/// let html = r#"<div style="position: absolute;"><span>x</span></div><p></p>"#;
/// let document = kuchiki::parse_html().one(html);
/// crate::utils::sanitizer::dom::manipulations::mark_positioned_elements_dom(&document);
/// let div = document.select_first("div").unwrap().as_node().as_element().unwrap();
/// assert!(div.attributes.borrow().get("data-dead-if-empty").is_some());
/// let p = document.select_first("p").unwrap().as_node().as_element().unwrap();
/// assert!(p.attributes.borrow().get("data-dead-if-empty").is_none());
/// ```
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

/// Detects whether an element provides visible presentation via styles or attributes.
///
/// Inspects inline `style` declarations and common visual attributes (`background`, `bgcolor`)
/// to determine if the element contributes visible rendering (backgrounds, borders, non-zero
/// dimensions, shadows, etc.).
///
/// # Returns
///
/// `true` if the element has visual styling (for example a background, a border, a box-shadow,
/// or non-zero width/height), `false` otherwise.
///
/// # Examples
///
/// ```
/// // Given an element with inline style `background: red;`, this will return true:
/// // let result = has_visual_content_dom(&element);
/// // assert!(result);
/// ```
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
            "box-shadow",
            "width",
            "height",
            "min-width",
            "min-height",
        ];

        for (prop, val) in &styles {
            if visual_props.contains(&prop.as_str())
                && val != "0"
                && val != "none"
                && val != "transparent"
                && val != "auto"
                && val != "0px"
                && val != "0%"
                && !val.is_empty()
            {
                return true;
            }
        }
    }

    if attrs.get("background").is_some() || attrs.get("bgcolor").is_some() {
        return true;
    }

    false
}

/// Remove elements marked with `data-dead-if-empty` that contain no text, no child elements, and no visual content.
///
/// Elements that meet those criteria are detached from `document`. Elements marked with
/// `data-dead-if-empty` that are kept (because they contain text, children, or visual styling)
/// will have the `data-dead-if-empty` attribute removed. The `document` is modified in place.
///
/// # Examples
///
/// ```
/// use kuchiki::parse_html;
/// use kuchiki::traits::*;
///
/// let html = r#"
///   <div id="a" data-dead-if-empty></div>
///   <div id="b" data-dead-if-empty>kept</div>
/// "#;
/// let document = parse_html().one(html);
///
/// // call the function under test
/// strip_dead_elements_dom(&document);
///
/// let out = document.to_string();
/// assert!(!out.contains("id=\"a\"")); // removed
/// assert!(out.contains("id=\"b\""));  // kept
/// assert!(!out.contains("data-dead-if-empty")); // attribute cleaned from kept elements
/// ```
pub fn strip_dead_elements_dom(document: &NodeRef) {
    let mut nodes_to_remove: Vec<NodeRef> = Vec::new();
    let mut nodes_to_cleanup: Vec<kuchiki::ElementData> = Vec::new();

    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let attrs = element.attributes.borrow();
            if attrs.get("data-dead-if-empty").is_some() {
                drop(attrs);

                let has_text = !node.text_contents().trim().is_empty();
                let has_children = node.children().count() > 0;
                let has_visual = has_visual_content_dom(element);

                if !has_text && !has_children && !has_visual {
                    nodes_to_remove.push(node.clone());
                } else {
                    nodes_to_cleanup.push(element.clone());
                }
            }
        }
    }

    for node in nodes_to_remove {
        node.detach();
    }

    for element in nodes_to_cleanup {
        let mut attrs = element.attributes.borrow_mut();
        attrs.remove("data-dead-if-empty");
    }
}