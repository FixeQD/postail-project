//! Flex/grid to table conversion logic.

use kuchikiki::NodeRef;

use crate::css::parser::{parse_css_declarations, parse_css_value};
use crate::stages::table::{
    create_element, extract_non_layout_style, push_issue, set_style,
    strip_display_from_style_with_align,
};
use crate::types::IssueSeverity;

#[derive(Debug, PartialEq)]
pub enum DisplayType {
    Block,
    Flex,
    Grid,
    InlineBlock,
}

pub fn parse_display_type(style: &str) -> DisplayType {
    for (prop, value) in parse_css_declarations(style) {
        if prop == "display" {
            return match value.trim() {
                v if v.contains("flex") => DisplayType::Flex,
                v if v.contains("grid") => DisplayType::Grid,
                v if v == "inline-block" => DisplayType::InlineBlock,
                _ => DisplayType::Block,
            };
        }
    }
    DisplayType::Block
}

#[derive(PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

impl FlexAlign {
    pub fn to_halign(self) -> &'static str {
        match self {
            FlexAlign::Center => "center",
            FlexAlign::End => "right",
            FlexAlign::SpaceBetween | FlexAlign::SpaceAround => "center",
            _ => "left",
        }
    }
    pub fn to_valign(self) -> &'static str {
        match self {
            FlexAlign::Center => "middle",
            FlexAlign::End => "bottom",
            _ => "top",
        }
    }
}

pub fn parse_flex_align(v: &str) -> FlexAlign {
    match v.trim() {
        "center" => FlexAlign::Center,
        "flex-end" | "end" => FlexAlign::End,
        "space-between" => FlexAlign::SpaceBetween,
        "space-around" => FlexAlign::SpaceAround,
        "stretch" => FlexAlign::Stretch,
        _ => FlexAlign::Start,
    }
}

pub struct FlexGridInfo {
    pub direction: FlexDirection,
    pub align_items: FlexAlign,
    pub justify: FlexAlign,
    pub gap: f32,
    pub is_grid: bool,
    pub grid_columns: usize,
}

pub fn parse_flex_grid_style(style: &str) -> FlexGridInfo {
    let mut info = FlexGridInfo {
        direction: FlexDirection::Row,
        align_items: FlexAlign::Stretch,
        justify: FlexAlign::Start,
        gap: 0.0,
        is_grid: false,
        grid_columns: 0,
    };

    for (prop, value) in parse_css_declarations(style) {
        match prop.as_str() {
            "display" if value.contains("grid") => info.is_grid = true,
            "flex-direction" | "grid-auto-flow" if value.contains("column") => {
                info.direction = FlexDirection::Column
            }
            "grid-template-columns" => {
                let cols = count_grid_columns(&value);
                info.grid_columns = cols;
                if cols > 1 {
                    info.direction = FlexDirection::Row;
                } else if cols == 1 {
                    info.direction = FlexDirection::Column;
                }
            }
            "align-items" => info.align_items = parse_flex_align(&value),
            "justify-content" => info.justify = parse_flex_align(&value),
            "gap" | "row-gap" | "column-gap" | "grid-gap" => info.gap = parse_css_value(&value),
            _ => {}
        }
    }

    info
}

/// Count columns from `grid-template-columns` value.
pub fn count_grid_columns(value: &str) -> usize {
    let v = value.trim();

    if let Some(rest) = v.strip_prefix("repeat(") {
        if let Some(comma) = rest.find(',') {
            let count_str = rest[..comma].trim();
            if count_str == "auto-fill" || count_str == "auto-fit" {
                return 2;
            }
            if let Ok(n) = count_str.parse::<usize>() {
                return n;
            }
        }
    }

    let mut count = 0;
    let mut depth = 0usize;
    let mut in_token = false;
    for ch in v.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ' ' | '\t' if depth == 0 => {
                in_token = false;
            }
            _ if depth == 0 => {
                if !in_token {
                    count += 1;
                    in_token = true;
                }
            }
            _ => {}
        }
    }
    count.max(1)
}

/// Convert `display: flex` or `display: grid` node to a table.
pub fn convert_flex_grid_to_table(node: &NodeRef) -> NodeRef {
    let style = node
        .as_element()
        .map(|el| {
            el.attributes
                .borrow()
                .get("style")
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_default();

    let info = parse_flex_grid_style(&style);
    let container_style = extract_non_layout_style(node);

    push_issue(
        if info.is_grid {
            "display: grid"
        } else {
            "display: flex"
        },
        if info.is_grid {
            "CSS grid converted to table layout for email compatibility"
        } else {
            "Flexbox converted to table layout for email compatibility"
        },
        IssueSeverity::Info,
    );

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

    let children: Vec<NodeRef> = node
        .children()
        .filter(|c| {
            c.as_text()
                .map(|t| !t.borrow().trim().is_empty())
                .unwrap_or(true)
        })
        .collect();

    let gap = info.gap as u32;
    let halign = info.justify.to_halign();
    let valign = info.align_items.to_valign();

    if info.direction == FlexDirection::Column {
        for (i, child) in children.iter().enumerate() {
            let actual = resolve_nested_flex(child);
            let tr = create_element("tr", &[]);
            let td = create_element("td", &[("align", halign), ("valign", valign)]);
            if gap > 0 && i < children.len() - 1 {
                set_style(&td, &format!("padding-bottom: {}px", gap));
            }
            actual.detach();
            td.append(actual);
            tr.append(td);
            table.append(tr);
        }
    } else {
        let tr = create_element("tr", &[]);
        for (i, child) in children.iter().enumerate() {
            let actual = resolve_nested_flex(child);
            let td = create_element("td", &[("align", halign), ("valign", valign)]);
            if gap > 0 && i < children.len() - 1 {
                set_style(&td, &format!("padding-right: {}px", gap));
            }
            actual.detach();
            td.append(actual);
            tr.append(td);
        }
        table.append(tr);
    }

    table
}

/// Walk a normal (non-flex/grid) element's descendants and convert any nested
/// flex/grid containers into tables in place.
pub fn convert_nested_in_place(node: &NodeRef) {
    let children: Vec<NodeRef> = node.children().collect();
    for child in children {
        let Some(el) = child.as_element() else {
            continue;
        };
        let style = el
            .attributes
            .borrow()
            .get("style")
            .unwrap_or("")
            .to_string();
        let pos = super::position::parse_position_style(&style);
        let disp = parse_display_type(&style);

        if pos.is_positioned {
            if pos.is_overlay {
                el.attributes
                    .borrow_mut()
                    .insert("style", "display: none".to_string());
            } else {
                let cleaned = clean_nested_positioned_style(&style, &pos);
                el.attributes.borrow_mut().insert("style", cleaned);
                if let Some(parent) = child.parent() {
                    if let Some(first) = parent.first_child() {
                        if first != child {
                            first.insert_before(child.clone());
                        }
                    }
                }
            }
        } else if matches!(disp, DisplayType::Flex | DisplayType::Grid) {
            let converted = convert_flex_grid_to_table(&child);
            child.insert_before(converted);
            child.detach();
        } else {
            convert_nested_in_place(&child);
        }
    }
}

pub fn clean_nested_positioned_style(style: &str, info: &crate::types::PositionInfo) -> String {
    let decls = parse_css_declarations(style);
    let mut out: Vec<String> = Vec::new();

    for (prop, value) in &decls {
        match prop.as_str() {
            "position" | "z-index" | "inset" | "top" | "bottom" | "left" | "right"
            | "transform" | "transform-origin" => continue,
            _ => out.push(format!("{}: {}", prop, value)),
        }
    }

    if info.horizontal_pos == "right" {
        out.push("float: right".to_string());
        out.push("display: inline-block".to_string());
    } else if !out.iter().any(|s| s.starts_with("display:")) {
        out.push("display: inline-block".to_string());
    }

    out.join("; ")
}

/// If a child itself has flex/grid, convert it; otherwise return as-is.
pub fn resolve_nested_flex(node: &NodeRef) -> NodeRef {
    let Some(el) = node.as_element() else {
        return node.clone();
    };
    let style = el
        .attributes
        .borrow()
        .get("style")
        .unwrap_or("")
        .to_string();
    let disp = parse_display_type(&style);

    if matches!(disp, DisplayType::Flex | DisplayType::Grid) {
        let children: Vec<_> = node
            .children()
            .filter(|c| {
                c.as_text()
                    .map(|t| !t.borrow().trim().is_empty())
                    .unwrap_or(true)
            })
            .collect();

        if children.len() <= 2 {
            let info = parse_flex_grid_style(&style);
            let cleaned = strip_display_from_style_with_align(&style, info.justify.to_halign());
            el.attributes.borrow_mut().insert("style", cleaned);
            return node.clone();
        }
        convert_flex_grid_to_table(node)
    } else {
        node.clone()
    }
}
