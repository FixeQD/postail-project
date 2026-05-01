//! Pipeline stages for HTML sanitization

pub mod inline;
pub mod pseudo;
pub mod stage2_scaling;
pub mod table;
pub mod stage1_preprocessing;
pub mod stage3_sanitization;
pub mod stage4_postprocessing;

// Re-export all stage functions
pub use inline::{inline_css_styles, inline_css_styles_dom, FONT_FACE_REGEX, IMPORT_REGEX, find_matching_brace};
pub use pseudo::{expand_pseudo_elements, expand_pseudo_elements_dom};
pub use stage2_scaling::{scale_elements_for_email_dom, scale_elements_for_email};
pub use table::*;
pub use stage1_preprocessing::*;
pub use stage3_sanitization::*;
pub use stage4_postprocessing::*;
