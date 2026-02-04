//! Stage 4: Postprocessing
//!
//! This module wraps cleanup functions from dom::serialization

pub use crate::utils::sanitizer::dom::serialization::{
    cleanup_html_whitespace, extract_body_content, serialize_clean,
};
