//! CSS inlining entry point.

use kuchikiki::traits::*;
use kuchikiki::NodeRef;

use crate::stages::inline::{
    apply_final_states_to_css_rules, parse_keyframe_final_states, remove_keyframes,
    resolve_clamp_values, strip_animation_from_inline_styles,
};
use crate::utils::brace_match::find_matching_brace;

pub fn inline_css_styles_dom(document: &NodeRef) {
    let html = document.to_string();

    // Step 1: Parse @keyframes final states from the <style> blocks
    let keyframe_finals = parse_keyframe_final_states(&html);

    // Step 2: Apply final-state values to CSS rules in <style> blocks.
    let patched = apply_final_states_to_css_rules(&html, &keyframe_finals);

    // Step 3: Remove @keyframes blocks (css_inline chokes on them / they're useless in email)
    let without_keyframes = remove_keyframes(&patched);

    // Step 4: Resolve clamp() - not supported in any email client
    let without_clamp = resolve_clamp_values(&without_keyframes);

    // Step 5: Run css_inline to move <style> rules into inline style="" attributes
    let inlined = css_inline::inline(&without_clamp).unwrap_or_else(|_| without_clamp.clone());

    // Step 6: Final cleanup - strip any leftover animation props in inline styles
    let final_html = strip_animation_from_inline_styles(&inlined);

    // Parse back into the document
    let new_doc = kuchikiki::parse_html().one(final_html).document_node;
    for child in document.children().collect::<Vec<_>>() {
        child.detach();
    }
    for child in new_doc.children() {
        document.append(child.clone());
    }
}

pub fn inline_css_styles(html: &str) -> String {
    let document = kuchikiki::parse_html().one(html).document_node;
    inline_css_styles_dom(&document);
    document.to_string()
}
