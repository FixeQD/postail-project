//! Table layout conversion module.
//!
//! Converts modern CSS layout (flexbox, grid, positioned elements) into
//! table-based HTML that renders consistently across all major email clients.

use kuchikiki::traits::TendrilSink;

mod dom_helpers;
mod elements;
mod flex_grid;
mod layout;
mod position;
mod style_helpers;

// Re-exports
pub use crate::types::PositionInfo;
pub use dom_helpers::{create_element, set_style, push_issue, make_lr_row};
pub use elements::{process_positioned_element, process_overlay_element, find_root_container};
pub use flex_grid::{
    DisplayType, FlexDirection, FlexAlign, FlexGridInfo,
    parse_display_type, parse_flex_grid_style, count_grid_columns,
    convert_flex_grid_to_table, convert_nested_in_place, resolve_nested_flex,
    clean_nested_positioned_style,
};
pub use layout::convert_to_table_layout_dom;
pub use position::{parse_position_style, positioning_issue, POSITIONING_PROPS};
pub use style_helpers::{
    extract_background_style, extract_non_layout_style,
    strip_display_from_style, strip_display_from_style_with_align,
    LAYOUT_PROPS, EMAIL_MAX_WIDTH,
};

pub fn convert_to_table_layout(html: &str) -> String {
    let doc = kuchikiki::parse_html().one(html).document_node;
    doc.to_string()
}
