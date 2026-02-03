//! Stage 2: CSS Processing - Inline styles and animations

use regex::Regex;

fn remove_keyframes(html: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    while let Some(start) = html[i..].find("@keyframes") {
        let start = i + start;
        let after_keyframes = &html[start + 10..];
        if let Some(brace_start_offset) = after_keyframes.find('{') {
            let brace_start = start + 10 + brace_start_offset;
            let mut count = 1;
            let mut j = brace_start + 1;
            while j < html.len() && count > 0 {
                if html.as_bytes()[j] == b'{' {
                    count += 1;
                } else if html.as_bytes()[j] == b'}' {
                    count -= 1;
                }
                j += 1;
            }
            if count == 0 {
                result.push_str(&html[i..start]);
                i = j;
            } else {
                result.push_str(&html[i..start]);
                i = start + 1;
            }
        } else {
            result.push_str(&html[i..start]);
            i = start + 1;
        }
    }
    result.push_str(&html[i..]);
    result
}

use crate::utils::sanitizer::stages::stage1_preprocessing::resolve_css_variables;

// Re-export types constants used in CSS processing
pub use crate::utils::sanitizer::types::{FONT_FACE_REGEX, IMPORT_REGEX};

pub fn inline_css_styles(html: &str) -> String {
    let resolved = resolve_css_variables(html);
    let inlined = css_inline::inline(&resolved).unwrap_or(resolved);
    remove_animations_and_fix_opacity(&inlined)
}

pub fn remove_animations_and_fix_opacity(html: &str) -> String {
    // Step 1: Remove @keyframes
    let without_keyframes = remove_keyframes(html);

    // Step 2: Process inline styles
    let style_re = Regex::new(r#"style="([^"]*)"#).unwrap();
    let animation_re = Regex::new(r"animation\s*:\s*[^;]+;?").unwrap();

    style_re
        .replace_all(&without_keyframes, |caps: &regex::Captures| {
            let mut style = caps[1].to_string();

            // Check if element has an animation
            let has_fade_animation = style.contains("animation:")
                && (style.contains("fadeIn")
                    || style.contains("fade")
                    || style.contains("rise")
                    || style.contains("expand"));

            // Remove animation property
            style = animation_re.replace_all(&style, "").to_string();

            if has_fade_animation && style.contains("opacity: 0") {
                style = style.replace("opacity: 0", "opacity: 1");
            }

            // Clean up empty declarations
            style = style
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("; ");

            format!(r#"style="{}""#, style)
        })
        .to_string()
}
