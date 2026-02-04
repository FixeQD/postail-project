//! Pipeline stages for HTML sanitization

pub mod stage1_preprocessing;
pub mod stage2_inline;
pub mod stage2_pseudo;
pub mod stage2_scaling;
pub mod stage2_table;
pub mod stage3_sanitization;
pub mod stage4_postprocessing;

// Re-export all stage functions
pub use stage1_preprocessing::*;
pub use stage2_inline::*;
pub use stage2_pseudo::*;
pub use stage2_scaling::*;
pub use stage2_table::*;
pub use stage3_sanitization::*;
pub use stage4_postprocessing::*;
