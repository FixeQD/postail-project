//! html_transpiler - email-safe HTML transpiler
//!
//! Converts modern HTML (flexbox, grid, CSS variables, animations) into
//! table-based, email-client-compatible markup. Also provides an
//! Ammonia-based sanitization pass for incoming email content.
//!
//! ## Main API
//! - [`sanitize_email_html_with_details`] — sanitize with issue tracking
//! - [`auto_fix_email_html`] — auto-fix and return clean HTML
//! - [`sanitize_email_html`] — sanitize without issue tracking

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
