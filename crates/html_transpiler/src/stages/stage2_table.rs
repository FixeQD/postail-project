//! Stage 2: Table layout conversion.
//!
//! Converts modern CSS layout (flexbox, grid, positioned elements) into
//! table-based HTML that renders consistently across all major email clients.

use kuchikiki::traits::*;
use kuchikiki::NodeRef;
use markup5ever::{namespace_url, ns, QualName};

use crate::config::COLLECTED_ISSUES;
use crate::css::parser::{parse_css_declarations, parse_css_value};
use crate::types::{IssueSeverity, PositionInfo, SanitizeIssue};

const EMAIL_MAX_WIDTH: f32 = 600.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Convert the document's layout to table-based HTML in place.
pub fn convert_to_table_layout_dom(document: &NodeRef) {
    let Some(root) = find_root_container(document) else {
        return;
    };

    // Unwrap a single non-positioned div wrapper — it's just noise
    let target = {
        let children: Vec<NodeRef> = root.children().collect();
        if children.len() == 1 {
            if let Some(el) = children[0].as_element() {
                let tag = el.name.local.as_ref().to_ascii_lowercase();
                let positioned = el
                    .attributes
                    .borrow()
                    .get("style")
                    .map(|s| parse_position_style(s).is_positioned)
                    .unwrap_or(false);
                if tag == "div" && !positioned {
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

    let root_bg = extract_background_style(&target);

    // First pass: classify children — only positioned elements need pre-collection
    let all_children: Vec<NodeRef> = target.children().collect();
    let positioned: Vec<(usize, NodeRef, PositionInfo)> = all_children
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let el = c.as_element()?;
            let style = el
                .attributes
                .borrow()
                .get("style")
                .unwrap_or("")
                .to_string();
            let pos = parse_position_style(&style);
            if pos.is_positioned {
                Some((i, c.clone(), pos))
            } else {
                None
            }
        })
        .collect();

    // Collect inline-block runs for grouping (consecutive siblings)
    let _has_inline_block = all_children.iter().any(|c| {
        c.as_element()
            .and_then(|el| el.attributes.borrow().get("style").map(|s| s.to_string()))
            .map(|s| parse_display_type(&s) == DisplayType::InlineBlock)
            .unwrap_or(false)
    });

    if all_children.is_empty() {
        return;
    }

    // --- Build outer wrapper table ---
    let wrapper = create_element(
        "table",
        &[
            ("width", "100%"),
            ("cellspacing", "0"),
            ("cellpadding", "0"),
            ("border", "0"),
            ("role", "presentation"),
        ],
    );
    if !root_bg.is_empty() {
        set_style(&wrapper, &root_bg);
    }

    let wr = create_element("tr", &[]);
    let wc = create_element("td", &[("align", "center"), ("valign", "top")]);
    wrapper.append(wr.clone());
    wr.append(wc.clone());

    let content = create_element(
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
        &content,
        &format!("max-width: {}px; margin: 0 auto", EMAIL_MAX_WIDTH),
    );
    wc.append(content.clone());

    // --- Determine which corner rows are needed ---
    let need_top = positioned
        .iter()
        .any(|(_, _, i)| i.vertical_pos == "top" && i.vertical_value < 0.0 && !i.is_overlay);
    let need_bottom = positioned
        .iter()
        .any(|(_, _, i)| i.vertical_pos == "bottom" && i.vertical_value < 0.0 && !i.is_overlay);
    let has_tc = positioned
        .iter()
        .any(|(_, _, i)| i.vertical_pos == "top" && (20.0..=50.0).contains(&i.vertical_value));
    let has_bc = positioned
        .iter()
        .any(|(_, _, i)| i.vertical_pos == "bottom" && (20.0..=50.0).contains(&i.vertical_value));
    let has_sides = need_top || need_bottom || has_tc || has_bc;
    let colspan = if has_sides { 2 } else { 1 };

    let mut top_cells = None;
    let mut tc_cells = None;
    let mut bottom_cells = None;
    let mut bc_cells = None;

    if need_top {
        let (r, l, r2) = make_lr_row("top_row", None);
        content.append(r);
        top_cells = Some((l, r2));
    }
    if has_tc {
        let (r, l, r2) = make_lr_row("top_corners_row", Some("padding-top: 28px"));
        content.append(r);
        tc_cells = Some((l, r2));
    }

    // Center content row
    let center_row = create_element("tr", &[]);
    let center_cell = create_element("td", &[("align", "center"), ("valign", "middle")]);
    if colspan > 1 {
        if let Some(el) = center_cell.as_element() {
            el.attributes
                .borrow_mut()
                .insert("colspan", colspan.to_string());
        }
    }
    center_row.append(center_cell.clone());
    content.append(center_row);

    if has_bc {
        let (r, l, r2) = make_lr_row("bottom_corners_row", Some("padding-bottom: 28px"));
        content.append(r);
        bc_cells = Some((l, r2));
    }
    if need_bottom {
        let (r, l, r2) = make_lr_row("bottom_row", None);
        content.append(r);
        bottom_cells = Some((l, r2));
    }

    // --- Place positioned elements into corner cells ---
    for (_, el, info) in &positioned {
        let el_clone = if info.is_overlay {
            process_overlay_element(el.clone(), info)
        } else {
            process_positioned_element(el.clone())
        };
        el_clone.detach();

        if info.is_overlay {
            center_cell.append(el_clone);
            continue;
        }

        let cell = match (
            info.vertical_pos.as_str(),
            info.vertical_value,
            info.horizontal_pos.as_str(),
        ) {
            ("top", v, "left") if v < 0.0 => top_cells.as_ref().map(|(l, _)| l.clone()),
            ("top", v, _) if v < 0.0 => top_cells.as_ref().map(|(l, _)| l.clone()),
            ("top", v, "right") if v < 0.0 => top_cells.as_ref().map(|(_, r)| r.clone()),
            ("bottom", v, "left") if v < 0.0 => bottom_cells.as_ref().map(|(l, _)| l.clone()),
            ("bottom", v, _) if v < 0.0 => bottom_cells.as_ref().map(|(_, r)| r.clone()),
            ("bottom", v, "right") if v < 0.0 => bottom_cells.as_ref().map(|(_, r)| r.clone()),
            ("top", v, "left") if (20.0..=50.0).contains(&v) => {
                tc_cells.as_ref().map(|(l, _)| l.clone())
            }
            ("top", v, "right") if (20.0..=50.0).contains(&v) => {
                tc_cells.as_ref().map(|(_, r)| r.clone())
            }
            ("bottom", v, "left") if (20.0..=50.0).contains(&v) => {
                bc_cells.as_ref().map(|(l, _)| l.clone())
            }
            ("bottom", v, "right") if (20.0..=50.0).contains(&v) => {
                bc_cells.as_ref().map(|(_, r)| r.clone())
            }
            _ => Some(center_cell.clone()),
        };
        if let Some(c) = cell {
            c.append(el_clone);
        }
    }

    let positioned_indices: std::collections::HashSet<usize> =
        positioned.iter().map(|(i, _, _)| *i).collect();

    // --- Process remaining children IN ORIGINAL DOM ORDER ---
    let mut pending_inline: Vec<NodeRef> = Vec::new();

    let flush_inline = |pending: &mut Vec<NodeRef>, center: &NodeRef| {
        if pending.is_empty() {
            return;
        }
        let tbl = create_element(
            "table",
            &[
                ("width", "100%"),
                ("cellspacing", "0"),
                ("cellpadding", "0"),
                ("border", "0"),
                ("role", "presentation"),
            ],
        );
        let tr = create_element("tr", &[]);
        for node in pending.drain(..) {
            let cell_style = extract_non_layout_style(&node);
            let td = create_element("td", &[("align", "left"), ("valign", "top")]);
            if !cell_style.is_empty() {
                set_style(&td, &cell_style);
            }
            node.detach();
            if let Some(el) = node.as_element() {
                let style = el
                    .attributes
                    .borrow()
                    .get("style")
                    .unwrap_or("")
                    .to_string();
                el.attributes
                    .borrow_mut()
                    .insert("style", strip_display_from_style(&style));
            }
            td.append(node);
            tr.append(td);
        }
        tbl.append(tr);
        push_issue(
            "display: inline-block",
            "inline-block siblings grouped into table row",
            IssueSeverity::Info,
        );
        center.append(tbl);
    };

    for (child_idx, child) in all_children.iter().enumerate() {
        // Skip positioned — already placed in corner cells
        if positioned_indices.contains(&child_idx) {
            child.detach();
            continue;
        }

        if let Some(text) = child.as_text() {
            if text.borrow().trim().is_empty() {
                child.detach();
                continue;
            }
            flush_inline(&mut pending_inline, &center_cell);
            child.detach();
            center_cell.append(child.clone());
            continue;
        }

        let Some(el) = child.as_element() else {
            flush_inline(&mut pending_inline, &center_cell);
            child.detach();
            center_cell.append(child.clone());
            continue;
        };

        let style = el
            .attributes
            .borrow()
            .get("style")
            .unwrap_or("")
            .to_string();
        let disp = parse_display_type(&style);

        if matches!(disp, DisplayType::Flex | DisplayType::Grid) {
            flush_inline(&mut pending_inline, &center_cell);
            let converted = convert_flex_grid_to_table(child);
            child.detach();
            center_cell.append(converted);
        } else if disp == DisplayType::InlineBlock {
            pending_inline.push(child.clone());
        } else {
            flush_inline(&mut pending_inline, &center_cell);
            // Recursively convert any flex/grid containers nested inside normal elements
            convert_nested_in_place(child);
            child.detach();
            center_cell.append(child.clone());
        }
    }
    flush_inline(&mut pending_inline, &center_cell);

    // legacy block kept for compat — already handled above
    // --- Detach all original children and attach wrapper ---
    for child in target.children().collect::<Vec<_>>() {
        child.detach();
    }
    target.append(wrapper);
}

pub fn convert_to_table_layout(html: &str) -> String {
    let doc = kuchikiki::parse_html().one(html).document_node;
    doc.to_string()
}

// ---------------------------------------------------------------------------
// Display type detection
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum DisplayType {
    Block,
    Flex,
    Grid,
    InlineBlock,
}

fn parse_display_type(style: &str) -> DisplayType {
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

// ---------------------------------------------------------------------------
// Flex / grid → table
// ---------------------------------------------------------------------------

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
    fn to_halign(self) -> &'static str {
        match self {
            FlexAlign::Center => "center",
            FlexAlign::End => "right",
            FlexAlign::SpaceBetween | FlexAlign::SpaceAround => "center",
            _ => "left",
        }
    }
    fn to_valign(self) -> &'static str {
        match self {
            FlexAlign::Center => "middle",
            FlexAlign::End => "bottom",
            _ => "top",
        }
    }
}

fn parse_flex_align(v: &str) -> FlexAlign {
    match v.trim() {
        "center" => FlexAlign::Center,
        "flex-end" | "end" => FlexAlign::End,
        "space-between" => FlexAlign::SpaceBetween,
        "space-around" => FlexAlign::SpaceAround,
        "stretch" => FlexAlign::Stretch,
        _ => FlexAlign::Start,
    }
}

struct FlexGridInfo {
    direction: FlexDirection,
    align_items: FlexAlign,
    justify: FlexAlign,
    gap: f32,
    is_grid: bool,
    grid_columns: usize, // 0 = unknown, >1 = explicit multi-column
}

fn parse_flex_grid_style(style: &str) -> FlexGridInfo {
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
                // Count columns: "1fr 1fr" → 2, "repeat(3, 1fr)" → 3
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
/// Handles `1fr 1fr 1fr`, `repeat(3, 1fr)`, `200px auto 1fr`, etc.
fn count_grid_columns(value: &str) -> usize {
    let v = value.trim();

    // Handle repeat(N, ...) — most common case
    if let Some(rest) = v.strip_prefix("repeat(") {
        if let Some(comma) = rest.find(',') {
            let count_str = rest[..comma].trim();
            // "auto-fill" / "auto-fit" → treat as multi-column (unknown, assume row)
            if count_str == "auto-fill" || count_str == "auto-fit" {
                return 2; // multi-column, force row
            }
            if let Ok(n) = count_str.parse::<usize>() {
                return n;
            }
        }
    }

    // Count space-separated tokens (ignoring nested parens like minmax(200px, 1fr))
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
fn convert_flex_grid_to_table(node: &NodeRef) -> NodeRef {
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
        // Column / grid: one row per child
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
        // Row: all children in one <tr>
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
/// flex/grid containers into tables in place. This handles cases like a wrapper
/// div that contains flex/grid children — the wrapper itself doesn't need
/// converting but its contents do.
fn convert_nested_in_place(node: &NodeRef) {
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
        let pos = parse_position_style(&style);
        let disp = parse_display_type(&style);

        if pos.is_positioned {
            if pos.is_overlay {
                // Decorative overlays (glow, blur, radial-gradient blobs) look terrible
                // as block elements in email — hide them entirely.
                el.attributes
                    .borrow_mut()
                    .insert("style", "display: none".to_string());
            } else {
                // Small UI elements (badges, labels): strip positioning,
                // float right if it was right-aligned, and move to front of parent.
                let cleaned = clean_nested_positioned_style(&style, &pos);
                el.attributes.borrow_mut().insert("style", cleaned);
                // Move to first child of parent so it appears at the top
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

fn clean_nested_positioned_style(style: &str, info: &PositionInfo) -> String {
    let decls = parse_css_declarations(style);
    let mut out: Vec<String> = Vec::new();

    for (prop, value) in &decls {
        match prop.as_str() {
            "position" | "z-index" | "inset" | "top" | "bottom" | "left" | "right"
            | "transform" | "transform-origin" => continue,
            _ => out.push(format!("{}: {}", prop, value)),
        }
    }

    // Float right if element was pinned to the right edge
    if info.horizontal_pos == "right" {
        out.push("float: right".to_string());
        out.push("display: inline-block".to_string());
    } else if !out.iter().any(|s| s.starts_with("display:")) {
        out.push("display: inline-block".to_string());
    }

    out.join("; ")
}

/// If a child itself has flex/grid, convert it; otherwise return as-is.
fn resolve_nested_flex(node: &NodeRef) -> NodeRef {
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

        // Small flex containers (≤2 children) just get display stripped + text-align added
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

// ---------------------------------------------------------------------------
// Position parsing
// ---------------------------------------------------------------------------

fn find_root_container(document: &NodeRef) -> Option<NodeRef> {
    for node in document.descendants() {
        if let Some(el) = node.as_element() {
            if el.name.local.as_ref().eq_ignore_ascii_case("body") {
                return Some(node);
            }
        }
    }
    document.descendants().find(|n| {
        n.as_element()
            .map(|e| e.name.local.as_ref().eq_ignore_ascii_case("div"))
            .unwrap_or(false)
    })
}

pub fn parse_position_style(style: &str) -> PositionInfo {
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

    let decls = parse_css_declarations(style);

    for (prop, value) in &decls {
        match prop.as_str() {
            "position" if value == "fixed" || value == "absolute" => {
                info.is_positioned = true;
                info.position_type = value.clone();
            }
            "top" => {
                info.vertical_pos = "top".into();
                info.vertical_value = parse_css_value(value);
            }
            "bottom" => {
                info.vertical_pos = "bottom".into();
                info.vertical_value = parse_css_value(value);
            }
            "left" => {
                info.horizontal_pos = "left".into();
                info.horizontal_value = parse_css_value(value);
            }
            "right" => {
                info.horizontal_pos = "right".into();
                info.horizontal_value = parse_css_value(value);
            }
            "inset" => {
                // Parse 1-4 value shorthand: top [right [bottom [left]]]
                let parts: Vec<f32> = value.split_whitespace().map(parse_css_value).collect();
                let (top, right, bottom, left) = match parts.as_slice() {
                    [a] => (*a, *a, *a, *a),
                    [a, b] => (*a, *b, *a, *b),
                    [a, b, c] => (*a, *b, *c, *b),
                    [a, b, c, d] => (*a, *b, *c, *d),
                    _ => (0.0, 0.0, 0.0, 0.0),
                };
                if info.vertical_pos == "none" {
                    info.vertical_pos = "top".into();
                    info.vertical_value = top;
                }
                if info.horizontal_pos == "none" {
                    info.horizontal_pos = "left".into();
                    info.horizontal_value = left;
                }
                let _ = (right, bottom); // used for full shorthand correctness
            }
            "width" => info.width = Some(parse_css_value(value)),
            "height" => info.height = Some(parse_css_value(value)),
            _ => {}
        }
    }

    if let (Some(w), Some(h)) = (info.width, info.height) {
        if w > 400.0 && h > 400.0 {
            info.is_overlay = true;
        }
    }
    // pointer-events: none = decorative overlay regardless of size
    if info.is_positioned
        && decls
            .iter()
            .any(|(p, v)| p == "pointer-events" && v == "none")
    {
        info.is_overlay = true;
    }
    // Also treat as overlay if it has a filter (blur glow) + position, even if small
    if info.is_positioned
        && decls
            .iter()
            .any(|(p, _)| p == "filter" || p == "backdrop-filter")
    {
        info.is_overlay = true;
    }

    info
}

// ---------------------------------------------------------------------------
// Element processing
// ---------------------------------------------------------------------------

fn process_positioned_element(element: NodeRef) -> NodeRef {
    if let Some(el) = element.as_element() {
        let mut attrs = el.attributes.borrow_mut();
        let style = attrs.get("style").unwrap_or("").to_string();
        let cleaned = clean_positioned_element_style(&style);
        if cleaned.is_empty() {
            attrs.remove("style");
        } else {
            attrs.insert("style", cleaned);
        }
    }
    element
}

fn process_overlay_element(element: NodeRef, info: &PositionInfo) -> NodeRef {
    if let Some(el) = element.as_element() {
        let mut attrs = el.attributes.borrow_mut();
        let style = attrs.get("style").unwrap_or("").to_string();
        let mut new: Vec<String> = Vec::new();

        for (prop, value) in parse_css_declarations(&style) {
            if matches!(
                prop.as_str(),
                "position"
                    | "top"
                    | "left"
                    | "right"
                    | "bottom"
                    | "z-index"
                    | "inset"
                    | "pointer-events"
            ) {
                continue;
            }
            new.push(format!("{}: {}", prop, value));
        }

        let margin_from = |val: f32| if val < 0.0 { 0.0 } else { val };

        if info.vertical_pos == "top" {
            new.push(format!(
                "margin-top: {}px",
                margin_from(info.vertical_value)
            ));
        } else if info.vertical_pos == "bottom" {
            new.push(format!(
                "margin-bottom: {}px",
                margin_from(info.vertical_value)
            ));
        }
        if info.horizontal_pos == "left" && info.horizontal_value > 0.0 {
            new.push(format!("margin-left: {}px", info.horizontal_value));
        } else if info.horizontal_pos == "right" && info.horizontal_value > 0.0 {
            new.push(format!("margin-right: {}px", info.horizontal_value));
        }

        new.push("display: block".to_string());
        new.push("overflow: hidden".to_string());
        attrs.insert("style", new.join("; "));
    }
    element
}

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

fn clean_positioned_element_style(style: &str) -> String {
    let mut out: Vec<String> = Vec::new();

    for (prop, value) in parse_css_declarations(style) {
        if POSITIONING_PROPS.iter().any(|&p| p == prop) {
            let (reason, severity) = positioning_issue(&prop);
            COLLECTED_ISSUES.with(|issues| {
                issues.borrow_mut().push(SanitizeIssue {
                    property: prop,
                    reason,
                    severity,
                    count: 1,
                });
            });
            continue;
        }
        out.push(format!("{}: {}", prop, value));
    }

    if !out.iter().any(|s| s.starts_with("display:")) {
        out.push("display: block".to_string());
    }
    out.join("; ")
}

fn positioning_issue(prop: &str) -> (String, IssueSeverity) {
    match prop {
        "position" => (
            "position removed — converted to table layout".into(),
            IssueSeverity::Warning,
        ),
        "z-index" => (
            "z-index removed — not supported in table layout".into(),
            IssueSeverity::Info,
        ),
        "transform" | "transform-origin" => (
            "transform removed — not supported in email".into(),
            IssueSeverity::Warning,
        ),
        "inset" => (
            "inset shorthand removed — converted to table positioning".into(),
            IssueSeverity::Info,
        ),
        _ => (
            format!("{} removed during table layout conversion", prop),
            IssueSeverity::Info,
        ),
    }
}

// ---------------------------------------------------------------------------
// Style extraction helpers
// ---------------------------------------------------------------------------

fn extract_background_style(node: &NodeRef) -> String {
    let Some(el) = node.as_element() else {
        return String::new();
    };
    let attrs = el.attributes.borrow();
    let Some(style) = attrs.get("style") else {
        return String::new();
    };

    parse_css_declarations(style)
        .iter()
        .filter(|(p, _)| p.starts_with("background") || p == "color" || p.starts_with("font"))
        .map(|(p, v)| format!("{}: {}", p, v))
        .collect::<Vec<_>>()
        .join("; ")
}

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
    "grid-gap",
    "grid-template-columns",
    "grid-template-rows",
    "grid-template",
    "grid-auto-flow",
    "grid-auto-columns",
    "grid-auto-rows",
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

fn extract_non_layout_style(node: &NodeRef) -> String {
    let Some(el) = node.as_element() else {
        return String::new();
    };
    let attrs = el.attributes.borrow();
    let Some(style) = attrs.get("style") else {
        return String::new();
    };

    parse_css_declarations(style)
        .iter()
        .filter(|(p, v)| {
            if LAYOUT_PROPS.contains(&p.as_str()) {
                return false;
            }
            if p == "min-height" && v.contains("vh") {
                return false;
            }
            true
        })
        .map(|(p, v)| format!("{}: {}", p, v))
        .collect::<Vec<_>>()
        .join("; ")
}

fn strip_display_from_style(style: &str) -> String {
    strip_display_from_style_with_align(style, "left")
}

fn strip_display_from_style_with_align(style: &str, halign: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut is_flex_center = false;

    for (prop, value) in parse_css_declarations(style) {
        match prop.as_str() {
            "display" if value.contains("flex") || value.contains("grid") => {
                out.push("display: block".to_string());
            }
            "flex-direction"
            | "flex-wrap"
            | "flex-flow"
            | "align-items"
            | "align-content"
            | "justify-content"
            | "justify-items"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "grid-template-columns"
            | "grid-template-rows"
            | "grid-template"
            | "grid-auto-flow"
            | "grid-auto-columns"
            | "grid-auto-rows"
            | "grid-gap" => {
                if prop == "justify-content" && value.contains("center") {
                    is_flex_center = true;
                }
            }
            _ => out.push(format!("{}: {}", prop, value)),
        }
    }

    if (halign == "center" || is_flex_center) && !out.iter().any(|s| s.starts_with("text-align")) {
        out.push("text-align: center".to_string());
    }
    out.join("; ")
}

// ---------------------------------------------------------------------------
// DOM creation helpers
// ---------------------------------------------------------------------------

fn create_element(tag: &str, attrs: &[(&str, &str)]) -> NodeRef {
    let node = NodeRef::new_element(QualName::new(None, ns!(html), tag.into()), None);
    if let Some(el) = node.as_element() {
        let mut a = el.attributes.borrow_mut();
        for (k, v) in attrs {
            a.insert(*k, v.to_string());
        }
    }
    node
}

fn set_style(node: &NodeRef, style: &str) {
    if let Some(el) = node.as_element() {
        let mut attrs = el.attributes.borrow_mut();
        let existing = attrs.get("style").unwrap_or("").to_string();
        let new = if existing.is_empty() {
            style.to_string()
        } else {
            format!("{}; {}", existing, style)
        };
        attrs.insert("style", new);
    }
}

/// Create a two-cell left/right row, returns `(row, left_cell, right_cell)`.
fn make_lr_row(class: &str, cell_style: Option<&str>) -> (NodeRef, NodeRef, NodeRef) {
    let row = create_element("tr", &[("class", class)]);
    let left = create_element(
        "td",
        &[
            ("class", &format!("{}_left", class)),
            ("align", "left"),
            ("valign", "middle"),
            ("width", "50%"),
        ],
    );
    let right = create_element(
        "td",
        &[
            ("class", &format!("{}_right", class)),
            ("align", "right"),
            ("valign", "middle"),
            ("width", "50%"),
        ],
    );
    if let Some(s) = cell_style {
        set_style(&left, s);
        set_style(&right, s);
    }
    row.append(left.clone());
    row.append(right.clone());
    (row, left, right)
}

fn push_issue(property: &str, reason: &str, severity: IssueSeverity) {
    COLLECTED_ISSUES.with(|issues| {
        issues.borrow_mut().push(SanitizeIssue {
            property: property.to_string(),
            reason: reason.to_string(),
            severity,
            count: 1,
        });
    });
}
