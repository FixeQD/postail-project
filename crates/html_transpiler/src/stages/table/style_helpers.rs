//! Style extraction and manipulation helpers.

use kuchikiki::NodeRef;

use crate::css::parser::parse_css_declarations;

pub const EMAIL_MAX_WIDTH: f32 = 600.0;

pub const LAYOUT_PROPS: &[&str] = &[
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

pub fn extract_background_style(node: &NodeRef) -> String {
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

pub fn extract_non_layout_style(node: &NodeRef) -> String {
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

pub fn strip_display_from_style(style: &str) -> String {
    strip_display_from_style_with_align(style, "left")
}

pub fn strip_display_from_style_with_align(style: &str, halign: &str) -> String {
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
