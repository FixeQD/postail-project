//! Pseudo-element expansion module.

mod expander;
mod parser;

// Re-exports
pub use expander::{expand_pseudo_elements_dom, expand_pseudo_elements};
pub use parser::{parse_pseudo_rules, merge_pseudo_rules, html_escape, PSEUDO_RE, PSEUDO_STRIP_PROPS};
pub use crate::utils::brace_match::find_matching_brace;
