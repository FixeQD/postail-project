//! Stage 2: Table layout conversion

use html5ever::QualName;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use markup5ever::{namespace_url, ns};

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::css::parser::{parse_css_declarations, parse_css_value};
use crate::utils::sanitizer::types::{IssueSeverity, PositionInfo, SanitizeIssue};

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
        let left = create_table_cell("top_corner_left", Some("left"), None);
        let right = create_table_cell("top_corner_right", Some("right"), None);
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
        let left = create_table_cell("bottom_corner_left", Some("left"), None);
        let right = create_table_cell("bottom_corner_right", Some("right"), None);
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
        if info.is_overlay {
            let element = process_overlay_element(element, &info);
            if let Some(ref center) = center_cell {
                element.detach();
                center.append(element);
            }
            continue;
        }

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
            "width" => {
                info.width = Some(parse_css_value(value));
            }
            "height" => {
                info.height = Some(parse_css_value(value));
            }
            _ => {}
        }
    }

    // Detect overlay elements (large decorative elements like glows)
    // Overlays are typically > 400px in both dimensions
    if let (Some(w), Some(h)) = (info.width, info.height) {
        if w > 400.0 && h > 400.0 {
            info.is_overlay = true;
        }
    }

    info
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

        attrs.insert("valign", "middle".to_string());

        if let Some(c) = colspan {
            attrs.insert("colspan", c.to_string());
        }

        // Set explicit width for left/right cells to prevent collapse
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

fn process_overlay_element(element: NodeRef, info: &PositionInfo) -> NodeRef {
    if let Some(elem) = element.as_element() {
        let mut attrs = elem.attributes.borrow_mut();

        let style = attrs
            .get("style")
            .map(|s| s.to_string())
            .unwrap_or_default();

        let declarations = parse_css_declarations(&style);
        let mut new_styles: Vec<String> = Vec::new();

        // Keep visual properties, remove positioning
        for (prop, value) in &declarations {
            match prop.as_str() {
                "position" | "top" | "left" | "right" | "bottom" | "z-index" => continue,
                _ => new_styles.push(format!("{}: {}", prop, value)),
            }
        }

        // Convert to relative positioning with margin offsets
        new_styles.push("position: relative".to_string());

        // Convert position offsets to margins
        // Negative top becomes margin-top, etc.
        if info.vertical_pos == "top" {
            new_styles.push(format!("margin-top: {}px", info.vertical_value));
        } else if info.vertical_pos == "bottom" {
            new_styles.push(format!("margin-bottom: {}px", info.vertical_value));
        }

        if info.horizontal_pos == "left" {
            new_styles.push(format!("margin-left: {}px", info.horizontal_value));
        } else if info.horizontal_pos == "right" {
            new_styles.push(format!("margin-right: {}px", info.horizontal_value));
        }

        // Overlays should not affect layout flow
        new_styles.push("pointer-events: none".to_string());

        attrs.insert("style", new_styles.join("; "));
    }

    element
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
        "transform" => (
            "transform removed - not supported in email clients".to_string(),
            IssueSeverity::Warning,
        ),
        _ => (
            format!("{} property removed during table layout conversion", prop),
            IssueSeverity::Info,
        ),
    }
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
        "transform",
    ];

    for (prop, value) in declarations {
        // Skip all positioning-related properties
        if POSITIONING_PROPS.iter().any(|&p| p == prop) {
            // Track the removed property as an issue
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
