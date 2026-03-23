//! Stage 2: Element scaling — clamp oversized elements to email-safe dimensions
//! and convert viewport units to fixed pixels.

use kuchikiki::traits::*;
use kuchikiki::NodeRef;

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::css::parser::{parse_css_declarations, parse_css_value};
use crate::utils::sanitizer::types::{IssueSeverity, SanitizeIssue};

/// Email max content width in pixels.
const MAX_EMAIL_WIDTH: f32 = 580.0;
/// Max height for purely decorative elements (glows, blobs, etc.).
const MAX_DECORATIVE_HEIGHT: f32 = 200.0;
/// Elements larger than this threshold get scaled.
const LARGE_ELEMENT_THRESHOLD: f32 = 400.0;
/// 1 vw / 1 vh ≈ 6 px at 600 px email width.
const VW_TO_PX: f32 = 6.0;

/// Walk the DOM and scale elements that exceed email-safe dimensions.
pub fn scale_elements_for_email_dom(document: &NodeRef) {
    for node in document.descendants() {
        let Some(el) = node.as_element() else {
            continue;
        };
        let tag = el.name.local.as_ref().to_ascii_lowercase();

        if matches!(
            tag.as_str(),
            "table" | "tr" | "td" | "th" | "tbody" | "thead" | "html" | "head" | "body"
        ) {
            continue;
        }

        let mut attrs = el.attributes.borrow_mut();
        let Some(style) = attrs.get("style").map(|s| s.to_string()) else {
            continue;
        };

        let new_style = scale_element_dimensions(&style);
        attrs.insert("style", new_style);
    }
}

pub fn scale_elements_for_email(html: &str) -> String {
    let doc = kuchikiki::parse_html().one(html);
    scale_elements_for_email_dom(&doc);
    doc.to_string()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn convert_viewport_units(prop: &str, value: &str) -> String {
    if !value.contains("vw") && !value.contains("vh") {
        return value.to_string();
    }
    let numeric = parse_css_value(value);
    if numeric > 0.0 {
        let px = (numeric * VW_TO_PX).round();
        push_issue(
            &format!("{}{}", prop, value),
            &format!("Viewport unit converted to fixed {}px", px),
            IssueSeverity::Info,
        );
        format!("{}px", px)
    } else {
        value.to_string()
    }
}

/// Scale an element's dimensions to fit within email-safe constraints.
/// Also handles `aspect-ratio` by inferring the missing dimension.
fn scale_element_dimensions(style: &str) -> String {
    let decls = parse_css_declarations(style);
    let mut out: Vec<String> = Vec::with_capacity(decls.len());

    let mut width: Option<f32> = None;
    let mut height: Option<f32> = None;
    let mut aspect_ratio: Option<f32> = None;
    let mut is_decorative = false;

    // First pass: gather sizing info
    for (prop, value) in &decls {
        match prop.as_str() {
            "width" => width = Some(parse_css_value(value)),
            "height" => height = Some(parse_css_value(value)),
            "aspect-ratio" => aspect_ratio = parse_aspect_ratio(value),
            "border-radius" if parse_css_value(value) > 100.0 => is_decorative = true,
            "filter" => is_decorative = true,
            _ => {}
        }
    }

    // If only one dimension known and aspect-ratio is set, infer the other
    if let Some(ratio) = aspect_ratio {
        if let (Some(w), None) = (width, height) {
            height = Some(w / ratio);
        } else if let (None, Some(h)) = (width, height) {
            width = Some(h * ratio);
        }
    }

    let scale = calculate_scale_factor(width, height, is_decorative);

    if scale < 1.0 {
        push_issue(
            "Element Dimensions",
            &format!(
                "Scaled down by {:.0}% to fit email width",
                (1.0 - scale) * 100.0
            ),
            IssueSeverity::Info,
        );
    }

    for (prop, value) in &decls {
        if prop == "min-height" && value.contains("vh") {
            continue;
        }
        if prop == "aspect-ratio" {
            // Drop aspect-ratio — email clients don't support it
            continue;
        }
        if prop == "filter" {
            push_issue(
                "filter",
                "CSS filters not supported in email clients",
                IssueSeverity::Warning,
            );
            continue;
        }

        let new_val = match prop.as_str() {
            "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height" => {
                // Convert vw/vh first — parse_css_value("100vw") returns 100 not 600
                if value.contains("vw") || value.contains("vh") {
                    convert_viewport_units(prop, value)
                } else {
                    let val = parse_css_value(value);
                    if val > 0.0 && scale < 1.0 {
                        format!("{}px", (val * scale).round())
                    } else if val > MAX_EMAIL_WIDTH && !is_decorative {
                        format!("{}px", MAX_EMAIL_WIDTH)
                    } else {
                        value.clone()
                    }
                }
            }
            m if is_decorative
                && matches!(
                    m,
                    "margin-top" | "margin-bottom" | "margin-left" | "margin-right"
                ) =>
            {
                format!("{}px", (parse_css_value(value) * scale).round())
            }
            _ if value.contains("vw") || value.contains("vh") => {
                convert_viewport_units(prop, value)
            }
            _ => value.clone(),
        };
        out.push(format!("{}: {}", prop, new_val));
    }

    // Clamp non-decorative large elements with max-width if not already set
    if width.map(|w| w > LARGE_ELEMENT_THRESHOLD).unwrap_or(false)
        && !is_decorative
        && !decls.iter().any(|(p, _)| p == "max-width")
    {
        out.push(format!("max-width: {}px", MAX_EMAIL_WIDTH));
    }

    out.join("; ")
}

fn calculate_scale_factor(width: Option<f32>, height: Option<f32>, is_decorative: bool) -> f32 {
    let max_h = if is_decorative {
        MAX_DECORATIVE_HEIGHT
    } else {
        800.0
    };
    let w = width.unwrap_or(0.0);
    let h = height.unwrap_or(0.0);

    if w < LARGE_ELEMENT_THRESHOLD && h < LARGE_ELEMENT_THRESHOLD {
        return 1.0;
    }

    let ws = if w > MAX_EMAIL_WIDTH {
        MAX_EMAIL_WIDTH / w
    } else {
        1.0
    };
    let hs = if h > max_h { max_h / h } else { 1.0 };
    ws.min(hs)
}

/// Parse `aspect-ratio: 16 / 9` or `aspect-ratio: 1.777` → scalar ratio.
fn parse_aspect_ratio(value: &str) -> Option<f32> {
    let v = value.trim();
    if let Some(slash) = v.find('/') {
        let num = v[..slash].trim().parse::<f32>().ok()?;
        let den = v[slash + 1..].trim().parse::<f32>().ok()?;
        if den != 0.0 {
            Some(num / den)
        } else {
            None
        }
    } else {
        v.parse::<f32>().ok()
    }
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
