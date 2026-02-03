//! Stage 2: Table layout conversion

use html5ever::QualName;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use markup5ever::{namespace_url, ns};

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::css::parser::{parse_css_declarations, parse_css_value};
use crate::utils::sanitizer::types::{IssueSeverity, PositionInfo, SanitizeIssue};

/// Convert HTML containing positioned or overlay elements into a table-based layout suitable for email clients.
///
/// The function parses `html`, detects positioned and overlay elements (based on inline `style`), removes positioning
/// CSS that is incompatible with table layouts, and rewraps content into a constructed `<table>` with optional top,
/// center, bottom, and corner cells. Positioned elements are placed into appropriate table cells (top/bottom/corners/center)
/// and overlays are converted to relative elements with margin offsets and pointer-events disabled. Non-positioned children
/// are moved into the center cell. Issues for removed positioning properties are recorded via the module's issue collector.
///
/// Returns the transformed HTML as a `String`.
///
/// # Examples
///
/// ```rust
/// let html = r#"
/// <body>
///   <div style="position:absolute; top:-10px; left:0;">Top banner</div>
///   <div>Main content</div>
/// </body>
/// "#;
/// let out = convert_to_table_layout(html);
/// assert!(out.contains("<table"));
/// assert!(out.contains("Top banner"));
/// assert!(out.contains("Main content"));
/// ```
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

/// Finds a suitable root container within the parsed document.
///
/// Searches descendants for the first `<body>` element and returns it if found; if no `<body>`
/// exists, returns the first `<div>` descendant; returns `None` if neither is present.
///
/// # Examples
///
/// ```
/// use kuchiki::parse_html;
/// use kuchiki::traits::TendrilSink;
///
/// let html = "<html><body><div id=\"root\">content</div></body></html>";
/// let document = parse_html().one(html);
/// let root = crate::utils::sanitizer::stages::stage2_table::find_root_container(&document)
///     .expect("expected a root container");
/// let name = root.as_element().unwrap().name.local.to_string().to_lowercase();
/// assert_eq!(name, "body");
/// ```
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

/// Extracts positioning metadata from an inline CSS style string.
///
/// Parses the provided inline style declarations and returns a `PositionInfo` describing
/// whether the element is positioned (absolute or fixed), the detected vertical and
/// horizontal offsets (as numeric pixel values when parseable), any parsed width/height,
/// and whether the element appears to be a large overlay (both width and height > 400).
///
/// # Examples
///
/// ```
/// let info = parse_position_style("position:absolute; top:10px; left:20px; width:100px; height:200px;");
/// assert!(info.is_positioned);
/// assert_eq!(info.position_type, "absolute");
/// assert_eq!(info.vertical_pos, "top");
/// assert_eq!(info.vertical_value, 10.0);
/// assert_eq!(info.horizontal_pos, "left");
/// assert_eq!(info.horizontal_value, 20.0);
/// assert_eq!(info.width, Some(100.0));
/// assert_eq!(info.height, Some(200.0));
/// assert!(!info.is_overlay);
/// ```
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

/// Creates a table row (`<tr>`) element with the given CSS class.
///
/// The returned node is an element node representing a `<tr>` with its `class` attribute set
/// to the provided value.
///
/// # Examples
///
/// ```
/// let row = create_table_row("row-class");
/// let elem = row.as_element().expect("should be an element");
/// assert_eq!(elem.name.local.as_ref(), "tr");
/// assert_eq!(elem.attributes.borrow().get("class"), Some(&"row-class".to_string()));
/// ```
fn create_table_row(class: &str) -> NodeRef {
    let row = NodeRef::new_element(QualName::new(None, ns!(html), "tr".into()), None);
    {
        let elem = row.as_element().unwrap();
        let mut attrs = elem.attributes.borrow_mut();
        attrs.insert("class", class.to_string());
    }
    row
}

/// Creates a `<td>` element with the given class, optional horizontal alignment and colspan, and ensures vertical alignment is middle.
///
/// The returned cell will have a `class` attribute set to `class`, an optional `align` attribute when `align` is `Some`, an optional `colspan` when `colspan` is `Some`, and a `valign="middle"` attribute. If the class string contains `left` or `right`, the cell will also receive `width="50%"` to prevent collapse.
///
/// # Examples
///
/// ```
/// let cell = create_table_cell("center", Some("center"), Some(2));
/// let elem = cell.as_element().unwrap();
/// let attrs = elem.attributes.borrow();
/// assert_eq!(attrs.get("class"), Some(&"center".to_string()));
/// assert_eq!(attrs.get("align"), Some(&"center".to_string()));
/// assert_eq!(attrs.get("colspan"), Some(&"2".to_string()));
/// assert_eq!(attrs.get("valign"), Some(&"middle".to_string()));
/// ```
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

/// Appends CSS declarations to an existing `style` attribute on a table cell element, preserving any existing styles.
///
/// If the node is not an element node the call is a no-op.
///
/// # Examples
///
/// ```no_run
/// use kuchiki::traits::*;
/// use kuchiki::parse_html;
/// use kuchiki::NodeRef;
///
/// // Build a small table and grab its first <td>.
/// let document = parse_html().one("<table><tr><td style=\"color:red\"></td></tr></table>");
/// let td = document.select_first("td").unwrap().as_node().clone();
///
/// // Append additional declarations.
/// add_style_to_cell(&td, "padding:0;");
///
/// // Verify the style now contains both original and appended declarations.
/// let style = td.as_element().and_then(|e| e.attributes.borrow().get("style").map(|s| s.to_string())).unwrap();
/// assert!(style.contains("color:red"));
/// assert!(style.contains("padding:0;"));
/// ```
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

/// Cleans an element's inline style by removing positioning-related CSS properties while preserving other declarations.
///
/// The function updates the node's `style` attribute in place: removed positioning properties are omitted and, if no style declarations remain, the `style` attribute is removed.
///
/// # Returns
///
/// The same `NodeRef` that was passed in, with its `style` attribute possibly updated or removed.
///
/// # Examples
///
/// ```
/// // Construct a node with an inline style containing positioning properties,
/// // run the cleaner, and observe that positioning declarations are removed.
/// use kuchiki::NodeRef;
///
/// // NOTE: constructing attributes here is illustrative; adapt to your project's imports.
/// let mut attrs = kuchiki::Attributes::new();
/// attrs.insert("style", "position:absolute;top:10px;display:block;");
/// let node = NodeRef::new_element("div".into(), attrs);
///
/// let processed = process_positioned_element(node.clone());
/// // `processed` now has a style that contains `display:block` but not `position` or `top`.
/// ```
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

/// Adjusts an overlay element's inline style for insertion into the table layout.
///
/// This removes positioning-related CSS declarations (`position`, `top`, `left`, `right`, `bottom`, `z-index`),
/// preserves other visual declarations, sets `position: relative`, converts vertical/horizontal offsets from
/// PositionInfo into corresponding `margin-*` properties (using pixel units), and adds `pointer-events: none`.
/// If the provided NodeRef is not an element node, it is returned unchanged.
///
/// # Examples
///
/// ```
/// use kuchiki::NodeRef;
///
/// // Create a simple element with absolute positioning
/// let node = NodeRef::new_element("div", vec![("style", "position: absolute; top: 10px; left: 20px; background: red;")]);
/// let info = crate::PositionInfo {
///     position: "absolute".into(),
///     vertical_pos: "top".into(),
///     vertical_value: 10,
///     horizontal_pos: "left".into(),
///     horizontal_value: 20,
///     width: None,
///     height: None,
///     is_overlay: true,
/// };
///
/// let updated = crate::process_overlay_element(node.clone(), &info);
/// let style = updated.as_element().unwrap().attributes.borrow().get("style").unwrap().to_string();
/// assert!(style.contains("background: red"));
/// assert!(style.contains("position: relative"));
/// assert!(style.contains("margin-top: 10px"));
/// assert!(style.contains("margin-left: 20px"));
/// assert!(style.contains("pointer-events: none"));
/// ```
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

/// Map a CSS property name to a human-readable issue message and a severity level
/// for reporting when that property is removed during table-layout conversion.
///
/// The returned tuple contains a message explaining why the property was removed
/// and an `IssueSeverity` value representing the reported importance.
///
/// # Examples
///
/// ```
/// let (msg, sev) = get_positioning_issue_details("position");
/// assert!(msg.contains("position property removed"));
/// matches!(sev, IssueSeverity::Warning);
/// ```
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

/// Removes positioning-related CSS declarations from an inline `style` string, records each removed property as a sanitize issue, and ensures a `display` declaration exists.
///
/// Specifically removes the following properties when present: `position`, `z-index`, `top`, `left`, `right`, `bottom`, and `transform`. Each removal pushes a `SanitizeIssue` into `COLLECTED_ISSUES`. If the resulting declarations do not include a `display` property, `display: block` is appended.
///
/// # Examples
///
/// ```
/// let cleaned = clean_positioned_element_style("position: absolute; top: 10px; color: red;");
/// assert!(cleaned.contains("color: red"));
/// assert!(cleaned.contains("display: block"));
/// assert!(!cleaned.contains("position:"));
/// ```
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