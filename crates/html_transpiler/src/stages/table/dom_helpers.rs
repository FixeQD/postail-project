//! DOM creation and issue reporting helpers.

use kuchikiki::NodeRef;
use markup5ever::{namespace_url, ns, QualName};

use crate::config::COLLECTED_ISSUES;
use crate::types::{IssueSeverity, SanitizeIssue};

pub fn create_element(tag: &str, attrs: &[(&str, &str)]) -> NodeRef {
    let node = NodeRef::new_element(QualName::new(None, ns!(html), tag.into()), None);
    if let Some(el) = node.as_element() {
        let mut a = el.attributes.borrow_mut();
        for (k, v) in attrs {
            a.insert(*k, v.to_string());
        }
    }
    node
}

pub fn set_style(node: &NodeRef, style: &str) {
    if let Some(el) = node.as_element() {
        let mut attrs = el.attributes.borrow_mut();
        let existing = attrs.get("style").unwrap_or("").to_string();
        let new = if existing.is_empty() {
            style.to_string()
        } else {
            format!("{}; {}", existing, style)
        };
        attrs.insert("style", new);
    }
}

/// Create a two-cell left/right row, returns `(row, left_cell, right_cell)`.
pub fn make_lr_row(class: &str, cell_style: Option<&str>) -> (NodeRef, NodeRef, NodeRef) {
    let row = create_element("tr", &[("class", class)]);
    let left = create_element(
        "td",
        &[
            ("class", &format!("{}_left", class)),
            ("align", "left"),
            ("valign", "middle"),
            ("width", "50%"),
        ],
    );
    let right = create_element(
        "td",
        &[
            ("class", &format!("{}_right", class)),
            ("align", "right"),
            ("valign", "middle"),
            ("width", "50%"),
        ],
    );
    if let Some(s) = cell_style {
        set_style(&left, s);
        set_style(&right, s);
    }
    row.append(left.clone());
    row.append(right.clone());
    (row, left, right)
}

pub fn push_issue(property: &str, reason: &str, severity: IssueSeverity) {
    COLLECTED_ISSUES.with(|issues| {
        issues.borrow_mut().push(SanitizeIssue {
            property: property.to_string(),
            reason: reason.to_string(),
            severity,
            count: 1,
        });
    });
}
