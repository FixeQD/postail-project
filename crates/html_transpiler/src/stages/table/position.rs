//! Position parsing and element processing.

use kuchikiki::NodeRef;

use crate::config::COLLECTED_ISSUES;
use crate::css::parser::{parse_css_declarations, parse_css_value};
use crate::types::{IssueSeverity, PositionInfo, SanitizeIssue};

pub const POSITIONING_PROPS: &[&str] = &[
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
                let _ = (right, bottom);
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
    if info.is_positioned
        && decls
            .iter()
            .any(|(p, v)| p == "pointer-events" && v == "none")
    {
        info.is_overlay = true;
    }
    if info.is_positioned
        && decls
            .iter()
            .any(|(p, _)| p == "filter" || p == "backdrop-filter")
    {
        info.is_overlay = true;
    }

    info
}

pub fn process_positioned_element(element: NodeRef) -> NodeRef {
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

pub fn process_overlay_element(element: NodeRef, info: &PositionInfo) -> NodeRef {
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

pub fn clean_positioned_element_style(style: &str) -> String {
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

pub fn positioning_issue(prop: &str) -> (String, IssueSeverity) {
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
