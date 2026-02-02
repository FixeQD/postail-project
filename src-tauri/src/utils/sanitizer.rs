use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::LazyLock;

use ammonia::Builder;
use html5ever::QualName;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use maplit::hashset;
use markup5ever::{namespace_url, ns};

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

// ─── Main Flow Functions ──────────────────────────────────────────────

pub fn auto_fix_email_html(html: &str) -> String {
    // Stage 1: Parse to DOM and replace body with div
    let document = kuchiki::parse_html().one(html);

    // Extract body styles
    let body_styles = extract_body_styles_from_css(html);

    // Replace body with div
    replace_body_with_div_dom(&document, body_styles);

    // Serialize for CSS processing
    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    // Stage 2: Pseudo-elements expansion
    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    let without_imports = IMPORT_REGEX.replace_all(&expanded, "");
    let without_font_faces = FONT_FACE_REGEX.replace_all(&without_imports, "");
    let inlined = inline_css_styles(&without_font_faces);

    // Stage 4: Convert positioning to table layout
    let table_layout = convert_to_table_layout(&inlined);

    // Stage 5: Parse again and strip content tags
    let document2 = kuchiki::parse_html().one(table_layout);
    strip_content_tags_dom(&document2);

    // Stage 6: Mark positioned elements
    mark_positioned_elements_dom(&document2);

    // Stage 7: Ammonia sanitization
    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);
    let builder = create_email_sanitizer();
    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Stage 8: Strip dead elements using DOM
    let document3 = kuchiki::parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    // Stage 9: Aaaaaaand cleanup HTML whitespace
    cleanup_html_whitespace(&result)
}

pub fn sanitize_email_html(html: &str) -> String {
    let resolved = resolve_css_variables(html);
    let document = kuchiki::parse_html().one(resolved.clone());

    let body_styles = extract_body_styles_from_css(&resolved);
    replace_body_with_div_dom(&document, body_styles);

    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    // Stage 2: Pseudo-elements expansion
    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    let inlined = inline_css_styles(&expanded);

    // Stage 4: Convert positioning to table layout
    let table_layout = convert_to_table_layout(&inlined);

    // Stage 5: Parse again and strip content tags
    let document2 = kuchiki::parse_html().one(table_layout);
    strip_content_tags_dom(&document2);

    // Stage 6: Mark positioned elements
    mark_positioned_elements_dom(&document2);

    // Stage 7: Ammonia sanitization
    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);
    let builder = create_email_sanitizer();
    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Stage 8: Strip dead elements using DOM
    let document3 = kuchiki::parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    // Stage 9: Cleanup HTML whitespace
    cleanup_html_whitespace(&result)
}

pub fn sanitize_email_html_with_details(html: &str) -> SanitizeResult {
    COLLECTED_ISSUES.with(|issues| issues.borrow_mut().clear());

    let unsupported_tags = detect_unsupported_tags(html);

    let resolved = resolve_css_variables(html);
    let document = kuchiki::parse_html().one(resolved.clone());

    let body_styles = extract_body_styles_from_css(&resolved);
    replace_body_with_div_dom(&document, body_styles);

    let html_str = document.to_string();
    let html_content = extract_body_content(&html_str);

    // Stage 2: Pseudo-elements expansion
    let expanded = expand_pseudo_elements(&html_content);

    // Stage 3: Inline CSS styles
    let inlined = inline_css_styles(&expanded);

    // Stage 4: Convert positioning to table layout
    let table_layout = convert_to_table_layout(&inlined);

    // Stage 5: Parse again and strip content tags
    let document2 = kuchiki::parse_html().one(table_layout);
    strip_content_tags_dom(&document2);

    // Stage 6: Mark positioned elements
    mark_positioned_elements_dom(&document2);

    // Record unsupported tag issues
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

    // Stage 7: Ammonia sanitization
    let serialized = serialize_clean(&document2);
    let content_for_ammonia = extract_body_content(&serialized);
    let builder = create_sanitizer_with_tracking();
    let sanitized = builder.clean(&content_for_ammonia).to_string();

    // Stage 8: Strip dead elements using DOM
    let document3 = kuchiki::parse_html().one(sanitized);
    strip_dead_elements_dom(&document3);
    let final_serialized = serialize_clean(&document3);
    let result = extract_body_content(&final_serialized);

    // Stage 9: Cleanup HTML whitespace
    let cleaned = cleanup_html_whitespace(&result);

    let issues = COLLECTED_ISSUES.with(|issues| issues.borrow().clone());

    SanitizeResult {
        html: cleaned,
        issues,
    }
}

// ─── Stage 1: Preprocessing ───────────────────────────────────────────

fn resolve_css_variables(html: &str) -> String {
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

fn parse_css_variables(html: &str) -> std::collections::HashMap<String, String> {
    let mut vars = std::collections::HashMap::new();
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

fn resolve_var_refs(value: &str, vars: &std::collections::HashMap<String, String>) -> String {
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

fn extract_body_styles_from_css(html: &str) -> String {
    let body_css_re = regex::Regex::new(r"(?s)(?:html,\s*)?body\s*\{([^}]+)\}").unwrap();

    if let Some(cap) = body_css_re.captures(html) {
        let declarations = &cap[1];
        let cleaned: Vec<String> = declarations
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter(|s| !s.contains("animation")) // Skip animations
            .filter(|s| !s.contains("transform")) // Skip transforms
            .filter(|s| !s.contains("filter")) // Skip filters
            .filter(|s| !s.contains("position")) // Skip position
            .map(|s| s.to_string())
            .collect();

        cleaned.join("; ")
    } else {
        String::new()
    }
}

fn replace_body_with_div_dom(document: &NodeRef, body_styles: String) {
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

                    // Set the merged style
                    if !style_attr.is_empty() {
                        div_attrs.insert("style", style_attr);
                    }
                }

                for child in node.children() {
                    div.append(child.clone());
                }

                // Replace body with div
                node.insert_before(div.clone());
                node.detach();

                break; // Only one body
            }
        }
    }
}

fn convert_to_table_layout(html: &str) -> String {
    let document = kuchiki::parse_html().one(html);

    let root = find_root_container(&document);
    if root.is_none() {
        return document.to_string();
    }
    let root = root.unwrap();
    let target_root = {
        let children: Vec<NodeRef> = root.children().collect();
        if children.len() == 1 {
            if let Some(element) = children[0].as_element() {
                let tag_name = element.name.local.to_string().to_lowercase();
                let attrs = element.attributes.borrow();
                let has_position = attrs
                    .get("style")
                    .map(|s| parse_position_style(s).is_positioned)
                    .unwrap_or(false);
                drop(attrs);
                if tag_name == "div" && !has_position {
                    children[0].clone()
                } else {
                    root.clone()
                }
            } else {
                root.clone()
            }
        } else {
            root.clone()
        }
    };

    let mut positioned_elements: Vec<(NodeRef, PositionInfo)> = Vec::new();
    let mut non_positioned_elements: Vec<NodeRef> = Vec::new();

    for child in target_root.children() {
        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();
            if tag_name == "table"
                || tag_name == "tr"
                || tag_name == "td"
                || tag_name == "th"
                || tag_name == "tbody"
                || tag_name == "thead"
                || tag_name == "tfoot"
            {
                continue;
            }

            let attrs = element.attributes.borrow();
            if let Some(style) = attrs.get("style") {
                let position_info = parse_position_style(style);
                if position_info.is_positioned {
                    drop(attrs);
                    positioned_elements.push((child.clone(), position_info));
                } else {
                    drop(attrs);
                    non_positioned_elements.push(child.clone());
                }
            } else {
                drop(attrs);
                non_positioned_elements.push(child.clone());
            }
        }
    }

    if positioned_elements.is_empty() && non_positioned_elements.is_empty() {
        return document.to_string();
    }

    let mut has_top_elements = false;
    let mut has_bottom_elements = false;
    let mut has_top_corners = false;
    let mut has_bottom_corners = false;

    for (_, info) in &positioned_elements {
        match info.vertical_pos.as_str() {
            "top" => {
                if info.vertical_value < 0.0 {
                    // Negative top = top row element
                    has_top_elements = true;
                } else if (20.0..=50.0).contains(&info.vertical_value) {
                    // Small positive top = top corners
                    has_top_corners = true;
                }
            }
            "bottom" => {
                if info.vertical_value < 0.0 {
                    // Negative bottom = bottom row element
                    has_bottom_elements = true;
                } else if (20.0..=50.0).contains(&info.vertical_value) {
                    // Small positive bottom = bottom corners
                    has_bottom_corners = true;
                }
            }
            _ => {}
        }
    }

    let table = NodeRef::new_element(QualName::new(None, ns!(html), "table".into()), None);
    {
        let elem = table.as_element().unwrap();
        let mut attrs = elem.attributes.borrow_mut();
        attrs.insert("width", "100%".to_string());
        attrs.insert("cellspacing", "0".to_string());
        attrs.insert("cellpadding", "0".to_string());
        attrs.insert("border", "0".to_string());
    }

    let mut all_rows: Vec<NodeRef> = Vec::new();
    let mut top_row_cells: Option<(NodeRef, NodeRef)> = None;
    let mut bottom_row_cells: Option<(NodeRef, NodeRef)> = None;
    let mut top_corners_cells: Option<(NodeRef, NodeRef)> = None;
    let mut bottom_corners_cells: Option<(NodeRef, NodeRef)> = None;
    let center_cell: Option<NodeRef>;

    if has_top_elements {
        let row = create_table_row("top_row");
        let left = create_table_cell("top_left", Some("left"), None);
        let right = create_table_cell("top_right", Some("right"), None);
        row.append(left.clone());
        row.append(right.clone());
        table.append(row.clone());
        all_rows.push(row);
        top_row_cells = Some((left, right));
    }

    if has_top_corners {
        let row = create_table_row("top_corners_row");
        let left = create_table_cell("top_corner_left", Some("left"), Some(1));
        let right = create_table_cell("top_corner_right", Some("right"), Some(1));
        add_style_to_cell(&left, "padding-top: 28px;");
        add_style_to_cell(&right, "padding-top: 28px;");
        row.append(left.clone());
        row.append(right.clone());
        table.append(row.clone());
        all_rows.push(row);
        top_corners_cells = Some((left, right));
    }

    {
        let row = create_table_row("center_row");
        let colspan = if has_top_elements || has_bottom_elements {
            2
        } else {
            1
        };
        let center = create_table_cell(
            "center",
            Some("center"),
            if colspan > 1 { Some(colspan) } else { None },
        );
        row.append(center.clone());
        table.append(row.clone());
        all_rows.push(row);
        center_cell = Some(center);
    }

    if has_bottom_corners {
        let row = create_table_row("bottom_corners_row");
        let left = create_table_cell("bottom_corner_left", Some("left"), Some(1));
        let right = create_table_cell("bottom_corner_right", Some("right"), Some(1));
        add_style_to_cell(&left, "padding-bottom: 28px;");
        add_style_to_cell(&right, "padding-bottom: 28px;");
        row.append(left.clone());
        row.append(right.clone());
        table.append(row.clone());
        all_rows.push(row);
        bottom_corners_cells = Some((left, right));
    }

    if has_bottom_elements {
        let row = create_table_row("bottom_row");
        let left = create_table_cell("bottom_left", Some("left"), None);
        let right = create_table_cell("bottom_right", Some("right"), None);
        row.append(left.clone());
        row.append(right.clone());
        table.append(row.clone());
        all_rows.push(row);
        bottom_row_cells = Some((left, right));
    }

    for (element, info) in positioned_elements {
        let element = process_positioned_element(element);

        let target_cell = match (
            info.vertical_pos.as_str(),
            info.vertical_value,
            info.horizontal_pos.as_str(),
        ) {
            // Top row (negative top values)
            ("top", val, "left") if val < 0.0 => {
                top_row_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("top", val, "right") if val < 0.0 => {
                top_row_cells.as_ref().map(|(_, right)| right.clone())
            }
            ("top", val, _) if val < 0.0 => top_row_cells.as_ref().map(|(left, _)| left.clone()),
            // Bottom row (negative bottom values)
            ("bottom", val, "left") if val < 0.0 => {
                bottom_row_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("bottom", val, "right") if val < 0.0 => {
                bottom_row_cells.as_ref().map(|(_, right)| right.clone())
            }
            ("bottom", val, _) if val < 0.0 => {
                bottom_row_cells.as_ref().map(|(_, right)| right.clone())
            }
            // Top corners (small positive top ~20-50px)
            ("top", val, "left") if (20.0..=50.0).contains(&val) => {
                top_corners_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("top", val, "right") if (20.0..=50.0).contains(&val) => {
                top_corners_cells.as_ref().map(|(_, right)| right.clone())
            }
            // Bottom corners (small positive bottom ~20-50px)
            ("bottom", val, "left") if (20.0..=50.0).contains(&val) => {
                bottom_corners_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("bottom", val, "right") if (20.0..=50.0).contains(&val) => bottom_corners_cells
                .as_ref()
                .map(|(_, right)| right.clone()),
            // Everything else goes to center
            _ => center_cell.clone(),
        };

        if let Some(cell) = target_cell {
            element.detach();
            cell.append(element);
        }
    }

    // Add non-positioned elements to center
    if let Some(ref center) = center_cell {
        for element in non_positioned_elements {
            element.detach();
            center.append(element);
        }
    }

    for child in target_root.children().collect::<Vec<_>>() {
        child.detach();
    }
    target_root.append(table);

    document.to_string()
}

fn find_root_container(document: &NodeRef) -> Option<NodeRef> {
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();
            if tag_name == "body" {
                return Some(node);
            }
        }
    }
    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();
            if tag_name == "div" {
                return Some(node);
            }
        }
    }
    None
}

#[derive(Debug)]
struct PositionInfo {
    is_positioned: bool,
    position_type: String,
    vertical_pos: String, // "top", "bottom", "none"
    vertical_value: f32,
    horizontal_pos: String, // "left", "right", "none"
    horizontal_value: f32,
}

fn parse_position_style(style: &str) -> PositionInfo {
    let mut info = PositionInfo {
        is_positioned: false,
        position_type: String::new(),
        vertical_pos: "none".to_string(),
        vertical_value: 0.0,
        horizontal_pos: "none".to_string(),
        horizontal_value: 0.0,
    };

    let declarations = parse_css_declarations(style);

    for (prop, value) in declarations {
        match prop.as_str() {
            "position" => {
                if value == "fixed" || value == "absolute" {
                    info.is_positioned = true;
                    info.position_type = value;
                }
            }
            "top" => {
                info.vertical_pos = "top".to_string();
                info.vertical_value = parse_css_value(&value);
            }
            "bottom" => {
                info.vertical_pos = "bottom".to_string();
                info.vertical_value = parse_css_value(&value);
            }
            "left" => {
                info.horizontal_pos = "left".to_string();
                info.horizontal_value = parse_css_value(&value);
            }
            "right" => {
                info.horizontal_pos = "right".to_string();
                info.horizontal_value = parse_css_value(&value);
            }
            _ => {}
        }
    }

    info
}

fn parse_css_value(value: &str) -> f32 {
    let cleaned: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned.parse::<f32>().unwrap_or(0.0)
}

fn create_table_row(class: &str) -> NodeRef {
    let row = NodeRef::new_element(QualName::new(None, ns!(html), "tr".into()), None);
    {
        let elem = row.as_element().unwrap();
        let mut attrs = elem.attributes.borrow_mut();
        attrs.insert("class", class.to_string());
    }
    row
}

fn create_table_cell(class: &str, align: Option<&str>, colspan: Option<usize>) -> NodeRef {
    let cell = NodeRef::new_element(QualName::new(None, ns!(html), "td".into()), None);
    {
        let elem = cell.as_element().unwrap();
        let mut attrs = elem.attributes.borrow_mut();
        attrs.insert("class", class.to_string());

        if let Some(a) = align {
            attrs.insert("align", a.to_string());
        }

        if let Some(c) = colspan {
            attrs.insert("colspan", c.to_string());
        }
    }
    cell
}

fn add_style_to_cell(cell: &NodeRef, additional_style: &str) {
    if let Some(element) = cell.as_element() {
        let mut attrs = element.attributes.borrow_mut();
        let existing = attrs
            .get("style")
            .map(|s| s.to_string())
            .unwrap_or_default();
        let new_style = if existing.is_empty() {
            additional_style.to_string()
        } else {
            format!("{} {}", existing, additional_style)
        };
        attrs.insert("style", new_style);
    }
}

fn process_positioned_element(element: NodeRef) -> NodeRef {
    if let Some(elem) = element.as_element() {
        let mut attrs = elem.attributes.borrow_mut();

        // Get and clean up the style
        let style = attrs
            .get("style")
            .map(|s| s.to_string())
            .unwrap_or_default();
        let cleaned_style = clean_positioned_element_style(&style);

        if cleaned_style.is_empty() {
            attrs.remove("style");
        } else {
            attrs.insert("style", cleaned_style);
        }
    }

    element
}

fn clean_positioned_element_style(style: &str) -> String {
    let declarations = parse_css_declarations(style);
    let mut cleaned: Vec<String> = Vec::new();

    // CSS properties to remove from positioned elements
    const POSITIONING_PROPS: &[&str] = &[
        "position",
        "z-index",
        "top",
        "left",
        "right",
        "bottom",
        "transform",
    ];

    for (prop, value) in declarations {
        // Skip all positioning-related properties
        if POSITIONING_PROPS.iter().any(|&p| p == prop) {
            continue;
        }

        cleaned.push(format!("{}: {}", prop, value));
    }

    if !cleaned.iter().any(|s| s.starts_with("display:")) {
        cleaned.push("display: block".to_string());
    }

    cleaned.join("; ")
}

fn sort_elements_by_z_index(html: &str) -> String {
    use std::collections::HashMap;

    let document = kuchiki::parse_html().one(html);
    let mut parent_children: HashMap<*const kuchiki::Node, Vec<(i32, NodeRef)>> = HashMap::new();

    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let style = element
                .attributes
                .borrow()
                .get("style")
                .map(|s| s.to_string());

            if let Some(style_str) = style {
                if let Some(z_index) = parse_z_index(&style_str) {
                    if let Some(parent) = node.parent() {
                        let parent_ptr = std::rc::Rc::as_ptr(&parent.0);
                        parent_children
                            .entry(parent_ptr)
                            .or_default()
                            .push((z_index, node.clone()));
                    }
                }
            }
        }
    }

    for (_, mut children) in parent_children {
        // Sort by z-index
        children.sort_by_key(|(z, _)| *z);

        // Get the sorted z-index nodes
        let z_nodes: Vec<NodeRef> = children.into_iter().map(|(_, node)| node).collect();

        for (new_pos, node) in z_nodes.iter().enumerate().rev() {
            if let Some(parent) = node.parent() {
                node.detach();

                let parent_children: Vec<NodeRef> = parent.children().collect();

                if new_pos >= parent_children.len() {
                    parent.append(node.clone());
                } else {
                    let target = &parent_children[new_pos];
                    target.insert_before(node.clone());
                }
            }
        }
    }

    // Serialize back to HTML
    document.to_string()
}

fn parse_z_index(style: &str) -> Option<i32> {
    for (prop, value) in parse_css_declarations(style) {
        if prop == "z-index" {
            let cleaned = value.trim();
            if cleaned != "auto" {
                return cleaned.parse::<i32>().ok();
            }
        }
    }
    None
}

fn expand_pseudo_elements(html: &str) -> String {
    let style_re = regex::Regex::new(r"(?s)(<style[^>]*>)(.*?)(</style>)").unwrap();
    let style_caps = match style_re.captures(html) {
        Some(c) => c,
        None => return html.to_string(), // no <style> → nothing to do
    };

    let style_open = &style_caps[1];
    let style_body = &style_caps[2];
    let style_close = &style_caps[3];
    let style_full = style_caps.get(0).unwrap();

    let (rules, cleaned_css) = parse_pseudo_rules(style_body);
    if rules.is_empty() {
        return html.to_string();
    }

    let mut new_css_rules = String::new();
    for rule in &rules {
        if !rule.style.is_empty() {
            new_css_rules.push_str(&format!(
                "\n.{} {{ {} }}\n",
                rule.class_for_style, rule.style
            ));
        }
    }

    let new_style = format!(
        "{}{}{}{}",
        style_open, cleaned_css, new_css_rules, style_close
    );
    let mut result = html[..style_full.start()].to_string();
    result.push_str(&new_style);
    result.push_str(&html[style_full.end()..]);

    for rule in &rules {
        let open_tag_re = regex::Regex::new(&format!(
            r#"(?s)(<[a-zA-Z][a-zA-Z0-9]*\s[^>]*class=")([^"]*\b{}\b[^"]*)"#,
            regex::escape(&rule.class)
        ))
        .expect("invalid class-match regex");

        let span = if rule.content.is_empty() {
            format!(
                r#"<span class="{}" style="display: inline-block"></span>"#,
                rule.class_for_style
            )
        } else {
            format!(
                r#"<span class="{}">{}</span>"#,
                rule.class_for_style, rule.content
            )
        };

        if rule.is_before {
            result = open_tag_re
                .replace_all(&result, |caps: &regex::Captures| {
                    let _match_end = caps.get(0).unwrap().end();
                    format!(
                        "{}{}\"__PSEUDO_BEFORE_{}__",
                        &caps[1], &caps[2], rule.class_for_style
                    )
                })
                .to_string();

            let placeholder = format!("__PSEUDO_BEFORE_{}__", rule.class_for_style);
            if let Some(ph_pos) = result.find(&placeholder) {
                let after_ph = ph_pos + placeholder.len();
                // Find the next `>` after the placeholder.
                if let Some(gt_offset) = result[after_ph..].find('>') {
                    let insert_pos = after_ph + gt_offset + 1; // right after >
                    result = format!(
                        "{}{}{}",
                        &result[..ph_pos], // everything before placeholder
                        &result[after_ph..insert_pos], // rest of tag
                        &format!("{}{}", &span, &result[insert_pos..])  // span + rest
                    );
                }
            }
        } else {
            let open_full_re = regex::Regex::new(&format!(
                r#"(?s)<([a-zA-Z][a-zA-Z0-9]*)\s[^>]*class="[^"]*\b{}\b[^"]*"[^>]*>"#,
                regex::escape(&rule.class)
            ))
            .expect("invalid full open tag regex");

            if let Some(caps) = open_full_re.captures(&result) {
                let tag_name = &caps[1];
                let open_end = caps.get(0).unwrap().end();
                let closing = format!("</{}>", tag_name);
                if let Some(close_offset) = result[open_end..].find(&closing) {
                    let insert_pos = open_end + close_offset;
                    result = format!("{}{}{}", &result[..insert_pos], span, &result[insert_pos..]);
                }
            }
        }
    }

    result
}

struct PseudoRule {
    class: String,           // e.g. "checkmark"
    is_before: bool,         // true = ::before, false = ::after
    content: String,         // literal text from content:"..."
    style: String,           // remaining declarations as a CSS rule body
    class_for_style: String, // the generated class name
}

fn parse_pseudo_rules(css: &str) -> (Vec<PseudoRule>, String) {
    // ── Step 1: split grouped selectors ─────────────────────────────────────
    let rule_re = regex::Regex::new(r"(?s)([^{]+)\{([^}]*)\}").expect("invalid rule block regex");

    let mut expanded_css = String::new();
    let mut last_end = 0;

    for caps in rule_re.captures_iter(css) {
        let full = caps.get(0).unwrap();
        expanded_css.push_str(&css[last_end..full.start()]);
        last_end = full.end();

        let selector_part = &caps[1];
        let body = &caps[2];

        let has_pseudo = selector_part.contains("::before") || selector_part.contains("::after");
        if !has_pseudo {
            expanded_css.push_str(&caps[0]);
            continue;
        }

        for fragment in selector_part.split(',') {
            let fragment = fragment.trim();
            if fragment.is_empty() {
                continue;
            }
            expanded_css.push_str(&format!("{} {{\n{}\n}}\n", fragment, body));
        }
    }
    expanded_css.push_str(&css[last_end..]);

    // ── Step 2: parse individual .class::pseudo { … } rules ─────────────────
    let pseudo_re = regex::Regex::new(r"(?s)\.([\w-]+)::(before|after)\s*\{([^}]*)\}")
        .expect("invalid pseudo rule regex");

    let mut rules = Vec::new();

    for caps in pseudo_re.captures_iter(&expanded_css) {
        let class = caps[1].to_string();
        let is_before = &caps[2] == "before";
        let body = &caps[3];

        let decls = parse_css_declarations(body);

        let mut content = String::new();
        let mut style_parts: Vec<String> = Vec::new();
        let mut has_content_decl = false;

        let mut has_display = false;

        for (prop, val) in &decls {
            if prop == "content" {
                has_content_decl = true;
                content = val
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
            } else {
                style_parts.push(format!("{}: {}", prop, val));
                if prop == "display" {
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

        let pseudo_kind = if is_before { "before" } else { "after" };
        let class_for_style = format!("__pseudo_{}__{}", class, pseudo_kind);
        let style_body = style_parts.join("; ");

        rules.push(PseudoRule {
            class,
            is_before,
            content,
            style: style_body,
            class_for_style,
        });
    }

    // Remove all pseudo rules from the CSS.
    let cleaned_css = pseudo_re.replace_all(&expanded_css, "").to_string();

    (rules, cleaned_css)
}

fn strip_content_tags_dom(document: &NodeRef) {
    // Tags to remove: head, script, style, title, noscript
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

    // Remove collected nodes
    for node in nodes_to_remove {
        node.detach();
    }
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

fn mark_positioned_elements_dom(document: &NodeRef) {
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

// ─── Stage 2: CSS Processing ──────────────────────────────────────────

fn inline_css_styles(html: &str) -> String {
    let resolved = resolve_css_variables(html);
    let inlined = css_inline::inline(&resolved).unwrap_or(resolved);
    remove_animations_and_fix_opacity(&inlined)
}

fn remove_animations_and_fix_opacity(html: &str) -> String {
    // Step 1: Remove @keyframes
    let keyframes_re = regex::Regex::new(r"(?s)@keyframes\s+\w+\s*\{[^}]*\}").unwrap();
    let without_keyframes = keyframes_re.replace_all(html, "").to_string();

    // Step 2: Process inline styles
    let style_re = regex::Regex::new(r#"style="([^"]*)"#).unwrap();
    let animation_re = regex::Regex::new(r"animation\s*:\s*[^;]+;?").unwrap();

    style_re
        .replace_all(&without_keyframes, |caps: &regex::Captures| {
            let mut style = caps[1].to_string();

            // Check if element has an animation
            let has_fade_animation = style.contains("animation:")
                && (style.contains("fadeIn")
                    || style.contains("fade")
                    || style.contains("rise")
                    || style.contains("expand"));

            // Remove animation property
            style = animation_re.replace_all(&style, "").to_string();

            if has_fade_animation && style.contains("opacity: 0") {
                style = style.replace("opacity: 0", "opacity: 1");
            }

            // Clean up empty declarations
            style = style
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("; ");

            format!(r#"style="{}""#, style)
        })
        .to_string()
}

// ─── Stage 3: HTML Sanitization ───────────────────────────────────────

pub fn create_email_sanitizer<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);

    builder.tag_attributes(maplit::hashmap! [
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

fn create_sanitizer_with_tracking<'a>() -> Builder<'a> {
    let mut builder = Builder::default();

    let allowed_tags: std::collections::HashSet<&str> = ALLOWED_TAGS.iter().cloned().collect();
    builder.tags(allowed_tags);

    builder.tag_attributes(maplit::hashmap! [
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

// ─── Stage 4: Postprocessing ──────────────────────────────────────────

fn serialize_clean(document: &NodeRef) -> String {
    let html = document.to_string();
    let clean_re = regex::Regex::new(r#"(\s+[a-zA-Z-]+)="""#).expect("invalid clean regex");
    clean_re.replace_all(&html, "$1").to_string()
}

fn extract_body_content(html: &str) -> String {
    let body_re = regex::Regex::new(r"(?s)<body[^>]*>(.*)</body>").unwrap();

    if let Some(caps) = body_re.captures(html) {
        caps[1].trim().to_string()
    } else {
        html.to_string()
    }
}

fn has_visual_content_dom(element: &kuchiki::ElementData) -> bool {
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

fn strip_dead_elements_dom(document: &NodeRef) {
    let mut nodes_to_remove: Vec<NodeRef> = Vec::new();
    let mut nodes_to_cleanup: Vec<kuchiki::ElementData> = Vec::new();

    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let attrs = element.attributes.borrow();
            if attrs.get("data-dead-if-empty").is_some() {
                drop(attrs);

                let has_text = !node.text_contents().trim().is_empty();
                let has_children = node.children().count() > 0;
                let has_visual = has_visual_content_dom(&element);

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

fn cleanup_html_whitespace(html: &str) -> String {
    html.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── Types & Results ──────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct StyleSanitizeResult {
    pub cleaned_style: String,
    pub removed_properties: Vec<String>,
    pub added_font_fallback: bool,
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

// ─── Thread-Local State ───────────────────────────────────────────────

thread_local! {
    static COLLECTED_ISSUES: RefCell<Vec<SanitizeIssue>> = const { RefCell::new(Vec::new()) };
}

static IMPORT_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"@import\s+[^;]+;?"#).expect("Invalid @import regex pattern")
});

static FONT_FACE_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"@font-face\s*\{[^}]*\}").expect("Invalid @font-face regex pattern")
});
