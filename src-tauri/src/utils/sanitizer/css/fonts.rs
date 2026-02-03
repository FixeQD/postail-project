//! Font mapping and fallback utilities

use crate::utils::sanitizer::config::WEB_SAFE_FONTS;

/// Map a custom font name to a corresponding web-safe fallback font stack.
///
/// The function recognizes common custom serif, sans-serif, and monospace font names (comparison
/// is case-insensitive and surrounding single/double quotes are ignored) and returns a suitable
/// web-safe fallback stack for known fonts.
///
/// # Returns
///
/// `Some` with a web-safe font stack if the font is recognized, `None` otherwise.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     map_custom_font_to_safe("\"Inter\""),
///     Some("Arial, Helvetica, sans-serif")
/// );
/// assert_eq!(map_custom_font_to_safe("Unknown Font"), None);
/// ```
pub fn map_custom_font_to_safe(font: &str) -> Option<&'static str> {
    let clean = font.trim_matches(|c| c == '"' || c == '\'').to_lowercase();

    match clean.as_str() {
        // Serif fonts - map to Georgia
        "cormorant garamond" | "cormorant" | "garamond" | "playfair display" | "merriweather"
        | "libre baskerville" | "crimson text" | "eb garamond" | "pt serif" | "noto serif"
        | "source serif pro" | "alice" | "cardo" => Some("Georgia, 'Times New Roman', serif"),
        // Sans-serif fonts - map to Arial/Helvetica
        "inter" | "roboto" | "open sans" | "lato" | "montserrat" | "poppins" | "nunito"
        | "raleway" | "ubuntu" | "work sans" | "fira sans" | "source sans pro" | "pt sans"
        | "noto sans" => Some("Arial, Helvetica, sans-serif"),
        // Monospace fonts
        "fira code" | "jetbrains mono" | "source code pro" | "roboto mono" | "space mono"
        | "ubuntu mono" | "ibm plex mono" => Some("'Courier New', Courier, monospace"),
        _ => None,
    }
}

/// Guarantees a web-safe fallback is present for a CSS `font-family` value.
///
/// If the first font is a recognized custom font, returns its mapped web-safe font stack.
/// If any font in the comma-separated list (quotes ignored, case-insensitive) matches a known web-safe font, returns the original `value` unchanged.
/// Otherwise returns `value` with ", sans-serif" appended.
///
/// # Returns
///
/// A `String` containing either the mapped web-safe stack, the original `value` (if it already includes a web-safe font), or `value` with ", sans-serif" appended.
///
/// # Examples
///
/// ```
/// // preserves original when a web-safe font is present
/// let v = ensure_web_safe_font_fallback("Arial, CustomFont");
/// assert_eq!(v, "Arial, CustomFont");
///
/// // appends sans-serif when no safe fallback is present
/// let v = ensure_web_safe_font_fallback("MyCustomFont");
/// assert_eq!(v, "MyCustomFont, sans-serif");
/// ```
pub fn ensure_web_safe_font_fallback(value: &str) -> String {
    let fonts: Vec<&str> = value.split(',').map(|f| f.trim()).collect();

    // Check if first font is a custom font that needs mapping
    if let Some(first) = fonts.first() {
        if let Some(mapped) = map_custom_font_to_safe(first) {
            return mapped.to_string();
        }
    }

    let has_safe_fallback = fonts.iter().any(|f| {
        let clean = f.trim_matches(|c| c == '"' || c == '\'').to_lowercase();
        WEB_SAFE_FONTS
            .iter()
            .any(|safe| safe.to_lowercase() == clean)
    });

    if has_safe_fallback {
        return value.to_string();
    }

    format!("{}, sans-serif", value)
}