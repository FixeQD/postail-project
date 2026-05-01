//! Pseudo-element expansion module.

mod brace_match;
mod expander;
mod parser;

// Re-exports
pub use brace_match::find_matching_brace;
pub use expander::{expand_pseudo_elements_dom, expand_pseudo_elements};
pub use parser::{parse_pseudo_rules, merge_pseudo_rules, html_escape, PSEUDO_RE, PSEUDO_STRIP_PROPS};
