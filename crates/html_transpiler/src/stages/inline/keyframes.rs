//! Keyframe parsing, patching, and removal.

use std::collections::HashMap;

use crate::css::parser::parse_css_declarations;
use crate::stages::inline::{
    STYLE_BLOCK_RE, ANIM_NAME_RE, TO_RE,
    find_matching_brace,
};

/// Returns a map of keyframe-name -> Vec<(property, value)> for the `to` / `100%` state.
pub fn parse_keyframe_final_states(html: &str) -> HashMap<String, Vec<(String, String)>> {
    let mut result: HashMap<String, Vec<(String, String)>> = HashMap::new();

    let mut i = 0;
    while let Some(kf_offset) = html[i..].find("@keyframes") {
        let kf_start = i + kf_offset;
        let after_kw = kf_start + "@keyframes".len();

        let rest = &html[after_kw..];
        let Some(brace_offset) = rest.find('{') else {
            i = after_kw;
            continue;
        };
        let name = rest[..brace_offset].trim().to_string();
        if name.is_empty() {
            i = after_kw + brace_offset + 1;
            continue;
        }

        let outer_brace_start = after_kw + brace_offset;

        let Some(outer_end) = find_matching_brace(html, outer_brace_start + 1) else {
            i = outer_brace_start + 1;
            continue;
        };

        let keyframe_body = &html[outer_brace_start + 1..outer_end - 1];

        if let Some(final_decls) = extract_final_frame(keyframe_body) {
            result.insert(name, final_decls);
        }

        i = outer_end;
    }

    result
}

/// Inside a @keyframes body, find the `to { ... }` or `100% { ... }` block
/// and return its parsed declarations.
pub fn extract_final_frame(keyframe_body: &str) -> Option<Vec<(String, String)>> {
    let to_re = &*TO_RE;

    let mut last_match = None;
    for m in to_re.find_iter(keyframe_body) {
        last_match = Some(m);
    }

    let m = last_match?;

    let brace_pos = keyframe_body[m.start()..].find('{')? + m.start();
    let end = find_matching_brace(keyframe_body, brace_pos + 1)?;

    let body = keyframe_body[brace_pos + 1..end - 1].trim();
    if body.is_empty() {
        return None;
    }

    Some(parse_css_declarations(body))
}

/// Remove all @keyframes blocks from CSS/HTML string.
pub fn remove_keyframes(html: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    while let Some(start_offset) = html[i..].find("@keyframes") {
        let start = i + start_offset;
        let after_keyframes = &html[start + 10..];
        if let Some(brace_start_offset) = after_keyframes.find('{') {
            let brace_start = start + 10 + brace_start_offset;
            if let Some(end) = find_matching_brace(html, brace_start + 1) {
                result.push_str(&html[i..start]);
                i = end;
            } else {
                result.push_str(&html[i..start]);
                i = start + 1;
            }
        } else {
            result.push_str(&html[i..start]);
            i = start + 1;
        }
    }
    result.push_str(&html[i..]);
    result
}

/// Apply keyframe final states to CSS rules in `<style>` blocks.
pub fn apply_final_states_to_css_rules(
    html: &str,
    keyframe_finals: &HashMap<String, Vec<(String, String)>>,
) -> String {
    if keyframe_finals.is_empty() {
        return html.to_string();
    }

    let style_re = &*STYLE_BLOCK_RE;

    style_re
        .replace_all(html, |caps: &regex::Captures| {
            let open = &caps[1];
            let css_body = &caps[2];
            let close = &caps[3];

            let patched_css = patch_css_rules(css_body, keyframe_finals);
            format!("{}{}{}", open, patched_css, close)
        })
        .to_string()
}

/// Patch CSS rules: replace initial values with keyframe final states.
pub fn patch_css_rules(
    css: &str,
    keyframe_finals: &HashMap<String, Vec<(String, String)>>,
) -> String {
    let anim_name_re = &*ANIM_NAME_RE;

    let mut result = String::new();
    let mut i = 0;

    while i < css.len() {
        // Skip @-rules
        if css[i..].starts_with("@keyframes") || css[i..].starts_with("@media") {
            if let Some(brace_pos) = css[i..].find('{') {
                let abs_brace = i + brace_pos;
                if let Some(end) = find_matching_brace(css, abs_brace + 1) {
                    result.push_str(&css[i..end]);
                    i = end;
                    continue;
                }
            }
            result.push(css.as_bytes()[i] as char);
            i += 1;
            continue;
        }

        // Look for the next `{`
        if let Some(brace_offset) = css[i..].find('{') {
            let selector_end = i + brace_offset;
            let selector = &css[i..selector_end];

            if let Some(end) = find_matching_brace(css, selector_end + 1) {
                let body = &css[selector_end + 1..end - 1];

                // Check if this rule has an animation property
                let anim_names: Vec<String> = anim_name_re
                    .captures_iter(body)
                    .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                    .collect();

                if anim_names.is_empty() {
                    // No animation - keep rule unchanged
                    result.push_str(&css[i..end]);
                } else {
                    let mut final_props: HashMap<String, String> = HashMap::new();
                    for name in &anim_names {
                        if let Some(props) = keyframe_finals.get(name.as_str()) {
                            for (p, v) in props {
                                final_props.insert(p.clone(), v.clone());
                            }
                        }
                    }

                    if final_props.is_empty() {
                        result.push_str(&css[i..end]);
                    } else {
                        let patched_body = super::animation::patch_rule_body(body, &final_props);
                        result.push_str(selector);
                        result.push('{');
                        result.push_str(&patched_body);
                        result.push('}');
                    }
                }

                i = end;
            } else {
                // Malformed - copy rest
                result.push_str(&css[i..]);
                break;
            }
        } else {
            // No more braces - copy rest
            result.push_str(&css[i..]);
            break;
        }
    }

    result
}
