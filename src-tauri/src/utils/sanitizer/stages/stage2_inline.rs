//! Stage 2: CSS Processing - Inline styles and animations

use regex::Regex;

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
    let keyframes_re = Regex::new(r"(?s)@keyframes\s+\w+\s*\{[^}]*\}").unwrap();
    let without_keyframes = keyframes_re.replace_all(html, "").to_string();

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
