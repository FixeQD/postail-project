//! clamp() CSS function resolver.

use crate::stages::inline::CLAMP_RE;

/// Resolve all clamp() occurrences to a single pixel value.
pub fn resolve_clamp_values(style: &str) -> String {
    let clamp_re = &*CLAMP_RE;
    clamp_re
        .replace_all(style, |caps: &regex::Captures| {
            let min_val = caps[1].trim();
            let preferred = caps[2].trim();
            let max_val = caps[3].trim();

            let min_px = resolve_single_value(min_val);
            let preferred_px = resolve_single_value(preferred);
            let max_px = resolve_single_value(max_val);

            let result_px = if preferred_px > 0.0 {
                preferred_px
                    .max(min_px)
                    .min(if max_px > 0.0 { max_px } else { preferred_px })
            } else if max_px > 0.0 {
                max_px
            } else if min_px > 0.0 {
                min_px
            } else {
                16.0
            };

            format!("{}px", result_px.round())
        })
        .to_string()
}

/// Convert a single CSS length value to pixels.
pub fn resolve_single_value(value: &str) -> f32 {
    let trimmed = value.trim();
    let numeric = extract_leading_number(trimmed);
    if numeric == 0.0 && !trimmed.starts_with('0') {
        return 0.0;
    }

    if trimmed.contains("vw") || trimmed.contains("vh") {
        return numeric * 6.0;
    }
    if trimmed.contains("rem") || trimmed.contains("em") {
        return numeric * 16.0;
    }
    if trimmed.contains("px")
        || trimmed
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
    {
        return numeric;
    }
    0.0
}

pub fn extract_leading_number(value: &str) -> f32 {
    let cleaned: String = value
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned.parse::<f32>().unwrap_or(0.0)
}
