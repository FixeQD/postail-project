//! Sanitizer configuration module.

use std::sync::LazyLock;
use regex::Regex;

pub mod fonts;
pub mod properties;
pub mod sanitizer;

// Re-exports
pub use properties::{DANGEROUS_CSS_SET, DANGEROUS_CSS_PROPS, WEB_SAFE_FONTS};
pub use sanitizer::{
    ALLOWED_TAGS, create_email_sanitizer, create_sanitizer_with_tracking,
    sanitize_style_attribute,
};
pub use fonts::{map_custom_font_to_safe, ensure_web_safe_font_fallback};

thread_local! {
    /// Accumulates sanitization issues during a single pipeline run.
    pub static COLLECTED_ISSUES: std::cell::RefCell<Vec<crate::types::SanitizeIssue>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub static TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<([a-zA-Z][a-zA-Z0-9]*)[^>]*>").expect("invalid TAG_REGEX")
});
