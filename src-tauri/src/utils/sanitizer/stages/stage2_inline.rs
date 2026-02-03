//! Stage 2: CSS Processing - Inline styles and animations

use regex::Regex;

use crate::utils::sanitizer::stages::stage1_preprocessing::resolve_css_variables;

// Re-export types constants used in CSS processing
pub use crate::utils::sanitizer::types::{FONT_FACE_REGEX, IMPORT_REGEX};

/// Resolve CSS variables, inline styles into the HTML, and post-process animations and opacity.
///
/// The returned HTML has CSS variables resolved and stylesheet rules inlined into element
/// style attributes; `@keyframes` blocks are removed, inline `animation` declarations are
/// stripped, and opacity values set to `0` for fade-like animations are changed to `1`.
///
/// # Examples
///
/// ```
/// let html = r#"<html><head><style>.fade{animation:fadeIn 1s;opacity:0;}</style></head><body><div class="fade">Hi</div></body></html>"#;
/// let result = inline_css_styles(html);
/// assert!(result.contains("style=\"")); // styles were inlined
/// assert!(!result.contains("@keyframes")); // keyframes removed
/// ```
pub fn inline_css_styles(html: &str) -> String {
    let resolved = resolve_css_variables(html);
    let inlined = css_inline::inline(&resolved).unwrap_or(resolved);
    remove_animations_and_fix_opacity(&inlined)
}

/// Removes CSS animations from the provided HTML and adjusts opacity for fade-like animations.
///
/// This function removes all `@keyframes` blocks, strips `animation` properties from inline
/// `style` attributes, and if an element referenced a fade-like animation (e.g., `fadeIn`,
/// `fade`, `rise`, `expand`) and contained `opacity: 0`, it replaces that declaration with
/// `opacity: 1`. The resulting HTML preserves other style declarations and cleans up empty
/// style entries.
///
/// # Returns
///
/// The transformed HTML string with animation rules removed and opacity adjusted where applicable.
///
/// # Examples
///
/// ```
/// let html = r#"<div style="animation: fadeIn 1s; opacity: 0; color: red">@content</div>
/// @keyframes fadeIn { from { opacity: 0 } to { opacity: 1 } }"#;
///
/// let out = remove_animations_and_fix_opacity(html);
/// assert!(!out.contains("@keyframes"));
/// assert!(out.contains(r#"style="opacity: 1; color: red""#));
/// ```
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