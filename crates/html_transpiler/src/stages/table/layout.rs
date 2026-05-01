//! Main table layout conversion logic.

use kuchikiki::NodeRef;

use crate::stages::table::{
    convert_nested_in_place, create_element, extract_background_style, extract_non_layout_style,
    make_lr_row, parse_display_type, push_issue, set_style, strip_display_from_style, DisplayType,
    PositionInfo, EMAIL_MAX_WIDTH,
};
use crate::types::IssueSeverity;

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
                    .map(|s| super::position::parse_position_style(s).is_positioned)
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
            let pos = super::position::parse_position_style(&style);
            if pos.is_positioned {
                Some((i, c.clone(), pos))
            } else {
                None
            }
        })
        .collect();

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
            super::position::process_overlay_element(el.clone(), info)
        } else {
            super::position::process_positioned_element(el.clone())
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
            let converted = super::flex_grid::convert_flex_grid_to_table(child);
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

pub fn find_root_container(document: &NodeRef) -> Option<NodeRef> {
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
