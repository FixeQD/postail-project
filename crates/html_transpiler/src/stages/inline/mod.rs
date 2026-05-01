//! CSS inlining module.
//!
//! Handles CSS inlining, keyframe processing, animation stripping, and clamp() resolution.

mod animation;
mod clamp;
mod inliner;
mod keyframes;
mod regexes;

// Re-exports
pub use animation::{strip_animation_from_inline_styles, is_initial_value, patch_rule_body};
pub use clamp::{resolve_clamp_values, resolve_single_value, extract_leading_number};
pub use inliner::{inline_css_styles_dom, inline_css_styles, find_matching_brace};
pub use keyframes::{
    parse_keyframe_final_states, extract_final_frame, remove_keyframes,
    apply_final_states_to_css_rules, patch_css_rules,
};
pub use regexes::{STYLE_BLOCK_RE, ANIM_NAME_RE, STYLE_ATTR_RE, ANIM_PROP_RE, TO_RE, CLAMP_RE};
pub use crate::types::{FONT_FACE_REGEX, IMPORT_REGEX};
