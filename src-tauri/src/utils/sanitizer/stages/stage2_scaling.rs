//! Stage 2: Element scaling

use kuchiki::traits::*;

use crate::utils::sanitizer::css::parser::{parse_css_declarations, parse_css_value};

const MAX_EMAIL_WIDTH: f32 = 580.0;
const MAX_DECORATIVE_HEIGHT: f32 = 200.0;
const LARGE_ELEMENT_THRESHOLD: f32 = 400.0;

use crate::utils::sanitizer::config::COLLECTED_ISSUES;
use crate::utils::sanitizer::types::{IssueSeverity, SanitizeIssue};

/// Adjust inline styles so HTML elements fit common email client constraints.
///
/// Parses the provided HTML and rewrites inline `style` attributes to improve email compatibility:
/// converting viewport units to pixels for the root viewport, scaling or clamping large dimensions,
/// removing unsupported properties (e.g., CSS filters), and preserving non-table structural elements.
/// Returns the modified HTML as a `String`.
///
/// # Examples
///
/// ```
/// let html = r#"<div style="height:100vh; width:1200px;"><p style="width:600px">Hello</p></div>"#;
/// let out = scale_elements_for_email(html);
/// assert!(out.contains("px"));
/// ```
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

/// Convert viewport units (vw/vh) in an inline style string to fixed pixel values for email compatibility.
///
/// Parses CSS declarations from `style`, converts any `vw`/`vh` values to pixel values (multiplied by 6 and rounded),
/// and returns a new style string with the updated declarations. If `is_root` is true and a `min-height` using `vh` is encountered,
/// a warning about full-screen height not supported in email clients will be recorded and that declaration will be omitted. Conversions
/// and encountered unsupported properties are recorded in the global `COLLECTED_ISSUES` registry.
///
/// # Arguments
///
/// * `style` - A semicolon-separated CSS declaration string (e.g., `"width:100vw; height:50vh"`).
/// * `is_root` - If true, treat `min-height` using `vh` as a root-level full-screen height and record a warning.
///
/// # Returns
///
/// A new CSS declaration string with viewport units converted to pixel values where applicable and unsupported root `min-height` removed.
///
/// # Examples
///
/// ```
/// let input = "width:100vw; min-height:100vh; color:red";
/// let out = fix_viewport_units(input, true);
/// // 100vw -> 600px (100 * 6), min-height:100vh is removed when is_root == true
/// assert!(out.contains("width: 600px"));
/// assert!(!out.contains("min-height"));
/// assert!(out.contains("color: red"));
/// ```
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

/// Scale and adjust inline CSS declarations so an element's styles fit typical email client constraints.
///
/// Parses a semicolon-separated style string and returns a transformed style string where large
/// dimensions are downscaled or clamped, viewport units (vw/vh) are converted to pixel values,
/// decorative elements (large border-radius or use of `filter`) are treated specially, and unsupported
/// properties (e.g., `filter`) are removed. The function may record informational or warning issues
/// in the global `COLLECTED_ISSUES` registry for conversions, removals, or scaling actions.
///
/// The returned string contains the adjusted declarations joined by "; " suitable for writing back
/// to an element's `style` attribute.
///
/// # Examples
///
/// ```
/// let out = scale_element_dimensions("width: 1000px; height: 100px;");
/// // Very wide widths are clamped to the email max width (e.g., 580px).
/// assert!(out.contains("width: 580px"));
/// ```
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

/// Compute a uniform scale factor that ensures an element's width and height fit within email constraints.
///
/// The factor is <= 1.0; values less than 1.0 indicate the element should be downscaled.
/// It considers a smaller maximum height for decorative elements.
///
/// # Returns
///
/// The scale factor to apply to element dimensions. `1.0` means no scaling is required.
///
/// # Examples
///
/// ```
/// // No scaling when both dimensions are small
/// assert_eq!(calculate_scale_factor(Some(100.0), Some(100.0), false), 1.0);
///
/// // Width exceeds max email width -> scale down
/// let s = calculate_scale_factor(Some(1200.0), Some(600.0), false);
/// assert!(s < 1.0 && s <= 580.0 / 1200.0);
///
/// // Decorative elements use a lower max height
/// let s2 = calculate_scale_factor(Some(300.0), Some(400.0), true);
/// assert!(s2 < 1.0 || s2 == 1.0);
/// ```
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