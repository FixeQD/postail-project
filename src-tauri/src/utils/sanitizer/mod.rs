//! HTML Sanitizer for email content
//!
//! This module provides email-safe HTML sanitization including:
//! - CSS variable resolution
//! - Pseudo-element expansion
//! - Positioned element conversion to table layout
//! - Ammonia-based sanitization
//! - Dead element removal

pub mod config;
pub mod css;
pub mod dom;
pub mod pipeline;
pub mod stages;
pub mod types;

// Re-export main public API
pub use pipeline::{auto_fix_email_html, sanitize_email_html, sanitize_email_html_with_details};
pub use types::{IssueSeverity, PositionInfo, SanitizeIssue, SanitizeResult, StyleSanitizeResult};

// Re-export commonly used utility functions
pub use config::sanitize_style_attribute;
pub use css::{parse_css_declarations, parse_css_value};
pub use dom::{cleanup_html_whitespace, extract_body_content, serialize_clean};
