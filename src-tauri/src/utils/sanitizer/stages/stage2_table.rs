//! Stage 2: Table layout conversion

use html5ever::QualName;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use markup5ever::{namespace_url, ns};

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::css::parser::{parse_css_declarations, parse_css_value};
use crate::utils::sanitizer::types::{IssueSeverity, PositionInfo, SanitizeIssue};

const EMAIL_MAX_WIDTH: f32 = 600.0;

pub fn convert_to_table_layout(html: &str) -> String {
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

    let root_bg_style = extract_background_style(&target_root);

    let mut positioned_elements: Vec<(NodeRef, PositionInfo)> = Vec::new();
    let mut non_positioned_elements: Vec<NodeRef> = Vec::new();
    let mut flexbox_containers: Vec<NodeRef> = Vec::new();

    for child in target_root.children() {
        // Skip whitespace-only text nodes at the root level
        if let Some(text) = child.as_text() {
            if text.borrow().trim().is_empty() {
                continue;
            }
            non_positioned_elements.push(child.clone());
            continue;
        }

        if let Some(element) = child.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();
            if matches!(
                tag_name.as_str(),
                "table" | "tr" | "td" | "th" | "tbody" | "thead" | "tfoot"
            ) {
                non_positioned_elements.push(child.clone());
                continue;
            }

            let attrs = element.attributes.borrow();
            if let Some(style) = attrs.get("style") {
                let position_info = parse_position_style(style);
                let flex_info = parse_flex_style(style);

                if position_info.is_positioned {
                    drop(attrs);
                    positioned_elements.push((child.clone(), position_info));
                } else if flex_info.is_flex {
                    drop(attrs);
                    flexbox_containers.push(child.clone());
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

    if positioned_elements.is_empty()
        && non_positioned_elements.is_empty()
        && flexbox_containers.is_empty()
    {
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
                    has_top_elements = true;
                } else if (20.0..=50.0).contains(&info.vertical_value) {
                    has_top_corners = true;
                }
            }
            "bottom" => {
                if info.vertical_value < 0.0 {
                    has_bottom_elements = true;
                } else if (20.0..=50.0).contains(&info.vertical_value) {
                    has_bottom_corners = true;
                }
            }
            _ => {}
        }
    }

    let need_top_row = has_top_elements
        && positioned_elements.iter().any(|(_, info)| {
            info.vertical_pos == "top" && info.vertical_value < 0.0 && !info.is_overlay
        });
    let need_bottom_row = has_bottom_elements
        && positioned_elements.iter().any(|(_, info)| {
            info.vertical_pos == "bottom" && info.vertical_value < 0.0 && !info.is_overlay
        });

    // Build the outer wrapper table
    let wrapper_table = create_element(
        "table",
        &[
            ("width", "100%"),
            ("cellspacing", "0"),
            ("cellpadding", "0"),
            ("border", "0"),
            ("role", "presentation"),
        ],
    );
    if !root_bg_style.is_empty() {
        set_style(&wrapper_table, &root_bg_style);
    }

    let wrapper_row = create_element("tr", &[]);
    let wrapper_cell = create_element("td", &[("align", "center"), ("valign", "top")]);
    wrapper_table.append(wrapper_row.clone());
    wrapper_row.append(wrapper_cell.clone());

    // Inner content table with max-width for email
    let content_table = create_element(
        "table",
        &[
            ("width", "100%"),
            ("cellspacing", "0"),
            ("cellpadding", "0"),
            ("border", "0"),
            ("role", "presentation"),
        ],
    );
    set_style(
        &content_table,
        &format!("max-width: {}px; margin: 0 auto", EMAIL_MAX_WIDTH),
    );
    wrapper_cell.append(content_table.clone());

    let mut all_rows: Vec<NodeRef> = Vec::new();
    let mut top_row_cells: Option<(NodeRef, NodeRef)> = None;
    let mut bottom_row_cells: Option<(NodeRef, NodeRef)> = None;
    let mut top_corners_cells: Option<(NodeRef, NodeRef)> = None;
    let mut bottom_corners_cells: Option<(NodeRef, NodeRef)> = None;
    let center_cell: NodeRef;

    // Top positioned elements row
    if need_top_row {
        let row = create_table_row("top_row");
        let left = create_table_cell("top_left", Some("left"), None);
        let right = create_table_cell("top_right", Some("right"), None);
        row.append(left.clone());
        row.append(right.clone());
        content_table.append(row.clone());
        all_rows.push(row);
        top_row_cells = Some((left, right));
    }

    // Top corners row
    if has_top_corners {
        let row = create_table_row("top_corners_row");
        let left = create_table_cell("top_corner_left", Some("left"), None);
        let right = create_table_cell("top_corner_right", Some("right"), None);
        add_style_to_cell(&left, "padding-top: 28px;");
        add_style_to_cell(&right, "padding-top: 28px;");
        row.append(left.clone());
        row.append(right.clone());
        content_table.append(row.clone());
        all_rows.push(row);
        top_corners_cells = Some((left, right));
    }

    // Center content row
    {
        let row = create_table_row("center_row");
        let colspan = if need_top_row || need_bottom_row || has_top_corners || has_bottom_corners {
            2
        } else {
            1
        };
        let cell = create_table_cell(
            "center",
            Some("center"),
            if colspan > 1 { Some(colspan) } else { None },
        );
        // Vertical center alignment for the main content cell
        if let Some(elem) = cell.as_element() {
            let mut attrs = elem.attributes.borrow_mut();
            attrs.insert("valign", "middle".to_string());
        }
        row.append(cell.clone());
        content_table.append(row.clone());
        all_rows.push(row);
        center_cell = cell;
    }

    // Bottom corners row
    if has_bottom_corners {
        let row = create_table_row("bottom_corners_row");
        let left = create_table_cell("bottom_corner_left", Some("left"), None);
        let right = create_table_cell("bottom_corner_right", Some("right"), None);
        add_style_to_cell(&left, "padding-bottom: 28px;");
        add_style_to_cell(&right, "padding-bottom: 28px;");
        row.append(left.clone());
        row.append(right.clone());
        content_table.append(row.clone());
        all_rows.push(row);
        bottom_corners_cells = Some((left, right));
    }

    // Bottom positioned elements row
    if need_bottom_row {
        let row = create_table_row("bottom_row");
        let left = create_table_cell("bottom_left", Some("left"), None);
        let right = create_table_cell("bottom_right", Some("right"), None);
        row.append(left.clone());
        row.append(right.clone());
        content_table.append(row.clone());
        all_rows.push(row);
        bottom_row_cells = Some((left, right));
    }

    // Place positioned elements into their target cells
    for (element, info) in positioned_elements {
        if info.is_overlay {
            let element = process_overlay_element(element, &info);
            element.detach();
            center_cell.append(element);
            continue;
        }

        let element = process_positioned_element(element);

        let target_cell = match (
            info.vertical_pos.as_str(),
            info.vertical_value,
            info.horizontal_pos.as_str(),
        ) {
            ("top", val, "left") if val < 0.0 => {
                top_row_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("top", val, "right") if val < 0.0 => {
                top_row_cells.as_ref().map(|(_, right)| right.clone())
            }
            ("top", val, _) if val < 0.0 => top_row_cells.as_ref().map(|(left, _)| left.clone()),
            ("bottom", val, "left") if val < 0.0 => {
                bottom_row_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("bottom", val, "right") if val < 0.0 => {
                bottom_row_cells.as_ref().map(|(_, right)| right.clone())
            }
            ("bottom", val, _) if val < 0.0 => {
                bottom_row_cells.as_ref().map(|(_, right)| right.clone())
            }
            ("top", val, "left") if (20.0..=50.0).contains(&val) => {
                top_corners_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("top", val, "right") if (20.0..=50.0).contains(&val) => {
                top_corners_cells.as_ref().map(|(_, right)| right.clone())
            }
            ("bottom", val, "left") if (20.0..=50.0).contains(&val) => {
                bottom_corners_cells.as_ref().map(|(left, _)| left.clone())
            }
            ("bottom", val, "right") if (20.0..=50.0).contains(&val) => bottom_corners_cells
                .as_ref()
                .map(|(_, right)| right.clone()),
            _ => Some(center_cell.clone()),
        };

        if let Some(cell) = target_cell {
            element.detach();
            cell.append(element);
        }
    }

    // Convert flexbox containers into table-based centering
    for flex_node in flexbox_containers {
        let converted = convert_flex_to_table(&flex_node);
        converted.detach();
        center_cell.append(converted);
    }

    // Add remaining non-positioned elements to center
    for element in non_positioned_elements {
        element.detach();
        center_cell.append(element);
    }

    for child in target_root.children().collect::<Vec<_>>() {
        child.detach();
    }
    target_root.append(wrapper_table);

    document.to_string()
}

// --- Flex detection & conversion ---

struct FlexInfo {
    is_flex: bool,
    direction: FlexDirection,
    align_items: FlexAlign,
    justify_content: FlexAlign,
    gap: f32,
}

#[derive(PartialEq)]
enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy)]
enum FlexAlign {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

impl FlexAlign {
    fn to_html_align(self) -> &'static str {
        match self {
            FlexAlign::Start => "left",
            FlexAlign::Center => "center",
            FlexAlign::End => "right",
            FlexAlign::SpaceBetween | FlexAlign::SpaceAround => "center",
            FlexAlign::Stretch => "left",
        }
    }

    fn to_html_valign(self) -> &'static str {
        match self {
            FlexAlign::Start => "top",
            FlexAlign::Center => "middle",
            FlexAlign::End => "bottom",
            _ => "top",
        }
    }
}

fn parse_flex_align(value: &str) -> FlexAlign {
    match value.trim() {
        "center" => FlexAlign::Center,
        "flex-start" | "start" => FlexAlign::Start,
        "flex-end" | "end" => FlexAlign::End,
        "space-between" => FlexAlign::SpaceBetween,
        "space-around" => FlexAlign::SpaceAround,
        "stretch" => FlexAlign::Stretch,
        _ => FlexAlign::Start,
    }
}

fn parse_flex_style(style: &str) -> FlexInfo {
    let mut info = FlexInfo {
        is_flex: false,
        direction: FlexDirection::Row,
        align_items: FlexAlign::Stretch,
        justify_content: FlexAlign::Start,
        gap: 0.0,
    };

    let declarations = parse_css_declarations(style);
    for (prop, value) in &declarations {
        match prop.as_str() {
            "display" if value.contains("flex") => info.is_flex = true,
            "flex-direction" if value.contains("column") => info.direction = FlexDirection::Column,
            "align-items" => info.align_items = parse_flex_align(value),
            "justify-content" => info.justify_content = parse_flex_align(value),
            "gap" | "row-gap" | "column-gap" => info.gap = parse_css_value(value),
            _ => {}
        }
    }

    info
}

fn convert_flex_to_table(flex_node: &NodeRef) -> NodeRef {
    let flex_info = {
        let elem = flex_node.as_element().unwrap();
        let attrs = elem.attributes.borrow();
        let style = attrs.get("style").unwrap_or("").to_string();
        parse_flex_style(&style)
    };

    // Extract non-flex, non-position styles from the original container
    let container_style = extract_non_layout_style(flex_node);

    COLLECTED_ISSUES.with(|issues| {
        issues.borrow_mut().push(SanitizeIssue {
            property: "display: flex".to_string(),
            reason: "Flexbox converted to table layout for email compatibility".to_string(),
            severity: IssueSeverity::Info,
            count: 1,
        });
    });

    let table = create_element(
        "table",
        &[
            ("width", "100%"),
            ("cellspacing", "0"),
            ("cellpadding", "0"),
            ("border", "0"),
            ("role", "presentation"),
        ],
    );

    if !container_style.is_empty() {
        set_style(&table, &container_style);
    }

    // Collect only meaningful children - skip whitespace text nodes
    let children: Vec<NodeRef> = flex_node
        .children()
        .filter(|child| {
            if let Some(text) = child.as_text() {
                return !text.borrow().trim().is_empty();
            }
            true
        })
        .collect();

    let gap_px = if flex_info.gap > 0.0 {
        flex_info.gap as u32
    } else {
        0
    };

    if flex_info.direction == FlexDirection::Column {
        // Column flex -> each child gets its own row
        let halign = flex_info.align_items.to_html_align();
        let valign = flex_info.justify_content.to_html_valign();

        for (i, child) in children.iter().enumerate() {
            let actual_child = maybe_convert_nested_flex(child);

            let row = create_element("tr", &[]);
            let cell = create_element("td", &[("align", halign), ("valign", valign)]);

            if gap_px > 0 && i < children.len() - 1 {
                set_style(&cell, &format!("padding-bottom: {}px", gap_px));
            }

            actual_child.detach();
            cell.append(actual_child);
            row.append(cell);
            table.append(row);
        }
    } else {
        // Row flex -> all children in one row, each in their own cell
        let row = create_element("tr", &[]);
        let valign = flex_info.align_items.to_html_valign();
        let halign = flex_info.justify_content.to_html_align();

        for (i, child) in children.iter().enumerate() {
            let actual_child = maybe_convert_nested_flex(child);

            let cell = create_element("td", &[("align", halign), ("valign", valign)]);

            if gap_px > 0 && i < children.len() - 1 {
                set_style(&cell, &format!("padding-right: {}px", gap_px));
            }

            actual_child.detach();
            cell.append(actual_child);
            row.append(cell);
        }
        table.append(row);
    }

    table
}

/// If a child element itself has `display: flex`, convert it to a table
/// Also strips `display: flex` from inline styles of elements that aren't converted
fn maybe_convert_nested_flex(node: &NodeRef) -> NodeRef {
    if let Some(element) = node.as_element() {
        let attrs = element.attributes.borrow();
        let style = attrs.get("style").unwrap_or("").to_string();
        let flex_info = parse_flex_style(&style);

        if flex_info.is_flex {
            drop(attrs);

            let meaningful_children: Vec<NodeRef> = node
                .children()
                .filter(|c| {
                    if let Some(t) = c.as_text() {
                        return !t.borrow().trim().is_empty();
                    }
                    true
                })
                .collect();

            if meaningful_children.len() <= 2 {
                let halign = flex_info.align_items.to_html_align();
                let cleaned = strip_flex_from_style(&style, halign);
                let mut attrs_mut = element.attributes.borrow_mut();
                attrs_mut.insert("style", cleaned);
                drop(attrs_mut);
                return node.clone();
            }

            // Larger flex containers get full table conversion
            return convert_flex_to_table(node);
        }
    }
    node.clone()
}

/// Remove display:flex and related properties from an inline style, replacing with text-align for centering.
fn strip_flex_from_style(style: &str, halign: &str) -> String {
    let declarations = parse_css_declarations(style);
    let mut result: Vec<String> = Vec::new();

    for (prop, value) in &declarations {
        match prop.as_str() {
            "display" if value.contains("flex") => {
                // Replace flex with block
                result.push("display: block".to_string());
            }
            "flex-direction" | "flex-wrap" | "flex-flow" | "align-items" | "align-content"
            | "justify-content" | "justify-items" | "gap" | "row-gap" | "column-gap" => {
                // Skip flex-specific properties
                continue;
            }
            _ => {
                result.push(format!("{}: {}", prop, value));
            }
        }
    }

    // Add text-align for centering if it was a centered flex
    if halign == "center" && !declarations.iter().any(|(p, _)| p == "text-align") {
        result.push("text-align: center".to_string());
    }

    result.join("; ")
}

// --- Position parsing ---

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

fn parse_position_style(style: &str) -> PositionInfo {
    let mut info = PositionInfo {
        is_positioned: false,
        position_type: String::new(),
        vertical_pos: "none".to_string(),
        vertical_value: 0.0,
        horizontal_pos: "none".to_string(),
        horizontal_value: 0.0,
        width: None,
        height: None,
        is_overlay: false,
    };

    let declarations = parse_css_declarations(style);

    for (prop, value) in &declarations {
        match prop.as_str() {
            "position" => {
                if value == "fixed" || value == "absolute" {
                    info.is_positioned = true;
                    info.position_type = value.clone();
                }
            }
            "top" => {
                info.vertical_pos = "top".to_string();
                info.vertical_value = parse_css_value(value);
            }
            "bottom" => {
                info.vertical_pos = "bottom".to_string();
                info.vertical_value = parse_css_value(value);
            }
            "left" => {
                info.horizontal_pos = "left".to_string();
                info.horizontal_value = parse_css_value(value);
            }
            "right" => {
                info.horizontal_pos = "right".to_string();
                info.horizontal_value = parse_css_value(value);
            }
            "inset" => {
                // inset: 0 is shorthand for top:0 right:0 bottom:0 left:0
                let val = parse_css_value(value);
                if info.vertical_pos == "none" {
                    info.vertical_pos = "top".to_string();
                    info.vertical_value = val;
                }
                if info.horizontal_pos == "none" {
                    info.horizontal_pos = "left".to_string();
                    info.horizontal_value = val;
                }
            }
            "width" => {
                info.width = Some(parse_css_value(value));
            }
            "height" => {
                info.height = Some(parse_css_value(value));
            }
            _ => {}
        }
    }

    // Detect overlay elements (large decorative things like glows, gradients)
    if let (Some(w), Some(h)) = (info.width, info.height) {
        if w > 400.0 && h > 400.0 {
            info.is_overlay = true;
        }
    }

    // Also treat pointer-events: none elements as overlays if positioned
    if info.is_positioned {
        let has_pointer_events_none = declarations
            .iter()
            .any(|(p, v)| p == "pointer-events" && v == "none");
        if has_pointer_events_none {
            info.is_overlay = true;
        }
    }

    info
}

// --- Element processing ---

fn process_positioned_element(element: NodeRef) -> NodeRef {
    if let Some(elem) = element.as_element() {
        let mut attrs = elem.attributes.borrow_mut();

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

fn process_overlay_element(element: NodeRef, info: &PositionInfo) -> NodeRef {
    if let Some(elem) = element.as_element() {
        let mut attrs = elem.attributes.borrow_mut();

        let style = attrs
            .get("style")
            .map(|s| s.to_string())
            .unwrap_or_default();

        let declarations = parse_css_declarations(&style);
        let mut new_styles: Vec<String> = Vec::new();

        for (prop, value) in &declarations {
            match prop.as_str() {
                // Strip all positioning and unsupported overlay props
                "position" | "top" | "left" | "right" | "bottom" | "z-index" | "inset"
                | "pointer-events" => continue,
                _ => new_styles.push(format!("{}: {}", prop, value)),
            }
        }

        // Convert position offsets to margins so it roughly sits where intended
        if info.vertical_pos == "top" {
            let margin = if info.vertical_value < 0.0 {
                0.0
            } else {
                info.vertical_value
            };
            new_styles.push(format!("margin-top: {}px", margin));
        } else if info.vertical_pos == "bottom" {
            let margin = if info.vertical_value < 0.0 {
                0.0
            } else {
                info.vertical_value
            };
            new_styles.push(format!("margin-bottom: {}px", margin));
        }

        if info.horizontal_pos == "left" && info.horizontal_value > 0.0 {
            new_styles.push(format!("margin-left: {}px", info.horizontal_value));
        } else if info.horizontal_pos == "right" && info.horizontal_value > 0.0 {
            new_styles.push(format!("margin-right: {}px", info.horizontal_value));
        }

        // Overlays become display:block, overflow hidden to contain them
        new_styles.push("display: block".to_string());
        new_styles.push("overflow: hidden".to_string());

        attrs.insert("style", new_styles.join("; "));
    }

    element
}

fn clean_positioned_element_style(style: &str) -> String {
    let declarations = parse_css_declarations(style);
    let mut cleaned: Vec<String> = Vec::new();

    const POSITIONING_PROPS: &[&str] = &[
        "position",
        "z-index",
        "top",
        "left",
        "right",
        "bottom",
        "inset",
        "transform",
        "transform-origin",
    ];

    for (prop, value) in declarations {
        if POSITIONING_PROPS.iter().any(|&p| p == prop) {
            let (reason, severity) = get_positioning_issue_details(&prop);
            COLLECTED_ISSUES.with(|issues| {
                issues.borrow_mut().push(SanitizeIssue {
                    property: prop.clone(),
                    reason,
                    severity,
                    count: 1,
                });
            });
            continue;
        }

        cleaned.push(format!("{}: {}", prop, value));
    }

    if !cleaned.iter().any(|s| s.starts_with("display:")) {
        cleaned.push("display: block".to_string());
    }

    cleaned.join("; ")
}

fn get_positioning_issue_details(prop: &str) -> (String, IssueSeverity) {
    match prop {
        "position" => (
            "position property removed - converted to table layout for email compatibility"
                .to_string(),
            IssueSeverity::Warning,
        ),
        "z-index" => (
            "z-index removed - not supported in table layout".to_string(),
            IssueSeverity::Info,
        ),
        "transform" | "transform-origin" => (
            "transform removed - not supported in email clients".to_string(),
            IssueSeverity::Warning,
        ),
        "inset" => (
            "inset shorthand removed - converted to table positioning".to_string(),
            IssueSeverity::Info,
        ),
        _ => (
            format!("{} property removed during table layout conversion", prop),
            IssueSeverity::Info,
        ),
    }
}

// --- Style extraction helpers ---

fn extract_background_style(node: &NodeRef) -> String {
    if let Some(element) = node.as_element() {
        let attrs = element.attributes.borrow();
        if let Some(style) = attrs.get("style") {
            let declarations = parse_css_declarations(style);
            let bg_props: Vec<String> = declarations
                .iter()
                .filter(|(prop, _)| {
                    prop.starts_with("background") || prop == "color" || prop.starts_with("font")
                })
                .map(|(prop, val)| format!("{}: {}", prop, val))
                .collect();
            return bg_props.join("; ");
        }
    }
    String::new()
}

fn extract_non_layout_style(node: &NodeRef) -> String {
    const LAYOUT_PROPS: &[&str] = &[
        "display",
        "flex-direction",
        "flex-wrap",
        "flex-flow",
        "align-items",
        "align-content",
        "justify-content",
        "justify-items",
        "gap",
        "row-gap",
        "column-gap",
        "position",
        "top",
        "left",
        "right",
        "bottom",
        "inset",
        "z-index",
        "float",
        "clear",
        "overflow",
        "overflow-x",
        "overflow-y",
        "min-height",
    ];

    if let Some(element) = node.as_element() {
        let attrs = element.attributes.borrow();
        if let Some(style) = attrs.get("style") {
            let declarations = parse_css_declarations(style);
            let kept: Vec<String> = declarations
                .iter()
                .filter(|(prop, val)| {
                    // Skip layout props
                    if LAYOUT_PROPS.contains(&prop.as_str()) {
                        return false;
                    }
                    // Skip min-height with vh
                    if prop == "min-height" && val.contains("vh") {
                        return false;
                    }
                    true
                })
                .map(|(prop, val)| format!("{}: {}", prop, val))
                .collect();
            return kept.join("; ");
        }
    }
    String::new()
}

// --- DOM helpers ---

fn create_element(tag: &str, attrs: &[(&str, &str)]) -> NodeRef {
    let node = NodeRef::new_element(QualName::new(None, ns!(html), tag.into()), None);
    if let Some(elem) = node.as_element() {
        let mut elem_attrs = elem.attributes.borrow_mut();
        for (key, value) in attrs {
            elem_attrs.insert(*key, value.to_string());
        }
    }
    node
}

fn set_style(node: &NodeRef, style: &str) {
    if let Some(elem) = node.as_element() {
        let mut attrs = elem.attributes.borrow_mut();
        let existing = attrs.get("style").unwrap_or("").to_string();
        if existing.is_empty() {
            attrs.insert("style", style.to_string());
        } else {
            attrs.insert("style", format!("{}; {}", existing, style));
        }
    }
}

fn create_table_row(class: &str) -> NodeRef {
    let row = create_element("tr", &[]);
    if let Some(elem) = row.as_element() {
        let mut attrs = elem.attributes.borrow_mut();
        attrs.insert("class", class.to_string());
    }
    row
}

fn create_table_cell(class: &str, align: Option<&str>, colspan: Option<usize>) -> NodeRef {
    let cell = create_element("td", &[]);
    if let Some(elem) = cell.as_element() {
        let mut attrs = elem.attributes.borrow_mut();
        attrs.insert("class", class.to_string());

        if let Some(a) = align {
            attrs.insert("align", a.to_string());
        }
        attrs.insert("valign", "middle".to_string());

        if let Some(c) = colspan {
            attrs.insert("colspan", c.to_string());
        }

        if class.contains("left") {
            attrs.insert("width", "50%".to_string());
        } else if class.contains("right") {
            attrs.insert("width", "50%".to_string());
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
