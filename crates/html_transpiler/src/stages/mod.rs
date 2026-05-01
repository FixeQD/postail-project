//! Pipeline stages for HTML sanitization

pub mod inline;
pub mod pseudo;
pub mod scaling;
pub mod table;
pub mod preprocessing;

// Re-export all stage functions
pub use inline::{inline_css_styles, inline_css_styles_dom, FONT_FACE_REGEX, IMPORT_REGEX, find_matching_brace};
pub use pseudo::{expand_pseudo_elements, expand_pseudo_elements_dom};
pub use scaling::{scale_elements_for_email_dom, scale_elements_for_email};
pub use table::*;
pub use preprocessing::*;
