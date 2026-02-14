use std::collections::{HashMap, HashSet};
use kuchiki::NodeRef;
use crate::utils::sanitizer::types::HtmlDiff;

/// Track changes between two DOM trees (or states of the same tree)
pub struct DiffTracker {
    initial_tags: HashMap<String, usize>,
    initial_attrs: HashSet<(String, String)>,
}

impl DiffTracker {
    pub fn new(document: &NodeRef) -> Self {
        let mut initial_tags = HashMap::new();
        let mut initial_attrs = HashSet::new();

        for node in document.descendants() {
            if let Some(element) = node.as_element() {
                let tag = element.name.local.to_string().to_lowercase();
                *initial_tags.entry(tag.clone()).or_insert(0) += 1;

                let attrs = element.attributes.borrow();
                for (key, _) in attrs.map.iter() {
                    initial_attrs.insert((tag.clone(), key.local.to_string()));
                }
            }
        }

        Self {
            initial_tags,
            initial_attrs,
        }
    }

    pub fn calculate_diff(&self, final_document: &NodeRef) -> HtmlDiff {
        let mut final_tags = HashMap::new();
        let mut final_attrs = HashSet::new();

        for node in final_document.descendants() {
            if let Some(element) = node.as_element() {
                let tag = element.name.local.to_string().to_lowercase();
                *final_tags.entry(tag.clone()).or_insert(0) += 1;

                let attrs = element.attributes.borrow();
                for (key, _) in attrs.map.iter() {
                    final_attrs.insert((tag.clone(), key.local.to_string()));
                }
            }
        }

        let mut removed_tags = Vec::new();
        for (tag, initial_count) in &self.initial_tags {
            let final_count = final_tags.get(tag).copied().unwrap_or(0);
            if final_count < *initial_count {
                // If it's completely gone or decreased, we report it.
                // For simplicity, just report the tag name once if it's completely removed or reduced
                removed_tags.push(tag.clone());
            }
        }

        let mut removed_attributes = Vec::new();
        for (tag, attr) in &self.initial_attrs {
            if !final_attrs.contains(&(tag.clone(), attr.clone())) {
                removed_attributes.push((tag.clone(), attr.clone()));
            }
        }

        HtmlDiff {
            removed_tags,
            removed_attributes,
            modified_styles: Vec::new(), // TODO: Track specific style property removals if needed
        }
    }
}
