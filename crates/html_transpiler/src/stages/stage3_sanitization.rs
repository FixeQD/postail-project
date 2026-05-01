//! Stage 3: HTML Sanitization
//!
//! This module wraps ammonia-based sanitization functions from config.

pub use crate::config::sanitizer::{
    create_email_sanitizer, create_sanitizer_with_tracking,
    sanitize_style_attribute,
};

pub use crate::dom::{
    detect_unsupported_tags, mark_positioned_elements_dom, strip_content_tags_dom,
    strip_dead_elements_dom, DiffTracker,
};
