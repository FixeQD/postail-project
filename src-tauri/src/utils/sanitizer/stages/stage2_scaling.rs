//! Stage 2: Element scaling

use kuchiki::traits::*;

use crate::utils::sanitizer::css::parser::{parse_css_declarations, parse_css_value};

const MAX_EMAIL_WIDTH: f32 = 580.0;
const MAX_DECORATIVE_HEIGHT: f32 = 200.0;
const LARGE_ELEMENT_THRESHOLD: f32 = 400.0;

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::types::{IssueSeverity, SanitizeIssue};

pub fn scale_elements_for_email(html: &str) -> String {
    let document = kuchiki::parse_html().one(html);
    let mut is_first_div = true;

    for node in document.descendants() {
        if let Some(element) = node.as_element() {
            let tag_name = element.name.local.to_string().to_lowercase();

            if matches!(
                tag_name.as_str(),
                "table" | "tr" | "td" | "th" | "tbody" | "thead" | "html" | "head" | "body"
            ) {
                continue;
            }

            if tag_name == "div" && is_first_div {
                is_first_div = false;
                let mut attrs = element.attributes.borrow_mut();
                if let Some(style) = attrs.get("style").map(|s| s.to_string()) {
                    let fixed_style = fix_viewport_units(&style, true);
                    attrs.insert("style", fixed_style);
                }
                continue;
            }

            let mut attrs = element.attributes.borrow_mut();
            if let Some(style) = attrs.get("style").map(|s| s.to_string()) {
                let scaled_style = scale_element_dimensions(&style);
                attrs.insert("style", scaled_style);
            }
        }
    }

    document.to_string()
}

fn fix_viewport_units(style: &str, is_root: bool) -> String {
    let declarations = parse_css_declarations(style);
    let mut new_declarations: Vec<String> = Vec::new();

    for (prop, value) in &declarations {
        // Skip min-height: 100vh - doesn't work in email
        if prop == "min-height" && value.contains("vh") {
            if is_root {
                COLLECTED_ISSUES.with(|issues| {
                    issues.borrow_mut().push(SanitizeIssue {
                        property: "min-height: 100vh".to_string(),
                        reason: "Full-screen height is not supported in email clients. Content might be cut off.".to_string(),
                        severity: IssueSeverity::Warning,
                        count: 1,
                    });
                });
            }
            continue;
        }

        let new_value = if value.contains("vw") || value.contains("vh") {
            let numeric = parse_css_value(value);
            if numeric > 0.0 {
                let px_val = (numeric * 6.0).round();
                COLLECTED_ISSUES.with(|issues| {
                    issues.borrow_mut().push(SanitizeIssue {
                        property: format!("{}{}", prop, value),
                        reason: format!("Viewport units converted to fixed pixels ({}px) for email compatibility.", px_val),
                        severity: IssueSeverity::Info,
                        count: 1,
                    });
                });
                format!("{}px", px_val) // 100vw = 600px
            } else {
                value.clone()
            }
        } else {
            value.clone()
        };

        new_declarations.push(format!("{}: {}", prop, new_value));
    }

    new_declarations.join("; ")
}

fn scale_element_dimensions(style: &str) -> String {
    let declarations = parse_css_declarations(style);
    let mut new_declarations: Vec<String> = Vec::new();

    let mut width: Option<f32> = None;
    let mut height: Option<f32> = None;
    let mut is_decorative = false;

    for (prop, value) in &declarations {
        match prop.as_str() {
            "width" => width = Some(parse_css_value(value)),
            "height" => height = Some(parse_css_value(value)),
            "border-radius" => {
                // Large border-radius often means decorative circle/oval
                let radius = parse_css_value(value);
                if radius > 100.0 {
                    is_decorative = true;
                }
            }
            "filter" => {
                // Elements with filter are decorative
                is_decorative = true;
            }
            _ => {}
        }
    }

    let scale_factor = calculate_scale_factor(width, height, is_decorative);

    if scale_factor < 1.0 {
        COLLECTED_ISSUES.with(|issues| {
            issues.borrow_mut().push(SanitizeIssue {
                property: "Element Dimensions".to_string(),
                reason: format!(
                    "Large element scaled down by {:.0}% to fit email width.",
                    (1.0 - scale_factor) * 100.0
                ),
                severity: IssueSeverity::Info,
                count: 1,
            });
        });
    }

    for (prop, value) in &declarations {
        // Skip min-height with vh
        if prop == "min-height" && value.contains("vh") {
            continue;
        }

        let new_value = match prop.as_str() {
            "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height" => {
                let val = parse_css_value(value);
                if val > 0.0 && scale_factor < 1.0 {
                    format!("{}px", (val * scale_factor).round())
                } else if val > MAX_EMAIL_WIDTH && !is_decorative {
                    format!("{}px", MAX_EMAIL_WIDTH)
                } else {
                    value.clone()
                }
            }
            // Scale margins for decorative elements
            "margin-top" | "margin-bottom" | "margin-left" | "margin-right" if is_decorative => {
                let val = parse_css_value(value);
                format!("{}px", (val * scale_factor).round())
            }
            // Remove filter - not supported in email
            "filter" => {
                COLLECTED_ISSUES.with(|issues| {
                    issues.borrow_mut().push(SanitizeIssue {
                        property: "filter".to_string(),
                        reason:
                            "CSS filters (blur, drop-shadow) are not supported in email clients."
                                .to_string(),
                        severity: IssueSeverity::Warning,
                        count: 1,
                    });
                });
                continue;
            }
            // Fix vw/vh units
            _ if value.contains("vw") || value.contains("vh") => {
                let numeric = parse_css_value(value);
                if numeric > 0.0 {
                    let px_val = (numeric * 6.0).round();
                    format!("{}px", px_val)
                } else {
                    value.clone()
                }
            }
            // Everything else stays
            _ => value.clone(),
        };
        new_declarations.push(format!("{}: {}", prop, new_value));
    }

    // Only add max-width for non-decorative large elements (not root containers)
    let should_add_max_width = width.map(|w| w > LARGE_ELEMENT_THRESHOLD).unwrap_or(false)
        && !is_decorative
        && !declarations.iter().any(|(p, _)| p == "max-width");

    if should_add_max_width {
        new_declarations.push(format!("max-width: {}px", MAX_EMAIL_WIDTH));
    }

    new_declarations.join("; ")
}

fn calculate_scale_factor(width: Option<f32>, height: Option<f32>, is_decorative: bool) -> f32 {
    let max_height = if is_decorative {
        MAX_DECORATIVE_HEIGHT
    } else {
        800.0
    };

    let w = width.unwrap_or(0.0);
    let h = height.unwrap_or(0.0);

    // Only scale if element is "large"
    if w < LARGE_ELEMENT_THRESHOLD && h < LARGE_ELEMENT_THRESHOLD {
        return 1.0;
    }

    let width_scale = if w > MAX_EMAIL_WIDTH {
        MAX_EMAIL_WIDTH / w
    } else {
        1.0
    };
    let height_scale = if h > max_height { max_height / h } else { 1.0 };

    // Use the smaller scale to ensure both dimensions fit
    width_scale.min(height_scale)
}
