//! Stage 2: CSS Processing - Inline styles and animations

use crate::utils::sanitizer::css::parser::parse_css_declarations;
pub use crate::utils::sanitizer::types::{FONT_FACE_REGEX, IMPORT_REGEX};
use kuchikiki::traits::*;
use kuchikiki::NodeRef;
use regex::Regex;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

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
    let new_doc = kuchikiki::parse_html().one(final_html);
    for child in document.children().collect::<Vec<_>>() {
        child.detach();
    }
    for child in new_doc.children() {
        document.append(child.clone());
    }
}

pub fn inline_css_styles(html: &str) -> String {
    let document = kuchikiki::parse_html().one(html);
    inline_css_styles_dom(&document);
    document.to_string()
}

// ---------------------------------------------------------------------------
// Step 2: Patch CSS rules in <style> blocks with keyframe final states
// ---------------------------------------------------------------------------

fn apply_final_states_to_css_rules(
    html: &str,
    keyframe_finals: &HashMap<String, Vec<(String, String)>>,
) -> String {
    if keyframe_finals.is_empty() {
        return html.to_string();
    }

    let style_re = Regex::new(r"(?s)(<style[^>]*>)(.*?)(</style>)").unwrap();

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

fn patch_css_rules(css: &str, keyframe_finals: &HashMap<String, Vec<(String, String)>>) -> String {
    let anim_name_re = Regex::new(r"animation(?:-name)?\s*:\s*([a-zA-Z_][\w-]*)").unwrap();

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
                        let patched_body = patch_rule_body(body, &final_props);
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

fn patch_rule_body(body: &str, final_props: &HashMap<String, String>) -> String {
    let decls = parse_css_declarations(body);
    let mut patched: Vec<(String, String)> = Vec::new();

    for (prop, val) in &decls {
        // Strip animation properties entirely
        if prop.starts_with("animation") {
            continue;
        }

        if let Some(final_val) = final_props.get(prop.as_str()) {
            if is_initial_value(val) {
                // Skip transform - not supported in email
                if prop == "transform"
                    || prop == "transform-origin"
                    || prop.starts_with("-webkit-transform")
                {
                    continue;
                }
                patched.push((prop.clone(), final_val.clone()));
                continue;
            }
        }

        patched.push((prop.clone(), val.clone()));
    }

    if patched.is_empty() {
        return String::new();
    }

    // Rebuild with newlines for readability (css_inline will flatten later)
    patched
        .iter()
        .map(|(p, v)| format!("  {}: {}", p, v))
        .collect::<Vec<_>>()
        .join(";\n")
        + ";\n"
}

fn is_initial_value(val: &str) -> bool {
    let v = val.trim();
    v == "0"
        || v == "0px"
        || v == "0%"
        || v == "0.0"
        || v == "0.00"
        || v == "none"
        || v == "initial"
        || v == "unset"
}

// ---------------------------------------------------------------------------
// Step 6: Strip leftover animation properties from inline styles
// ---------------------------------------------------------------------------

fn strip_animation_from_inline_styles(html: &str) -> String {
    let keyframe_finals = parse_keyframe_final_states(html);

    let style_re = Regex::new(r#"style="([^"]*)"#).unwrap();
    let animation_prop_re = Regex::new(r"animation[^:]*:\s*[^;]+;?").unwrap();
    let anim_name_re = Regex::new(r"animation(?:-name)?\s*:\s*([a-zA-Z_][\w-]*)").unwrap();

    style_re
        .replace_all(html, |caps: &regex::Captures| {
            let style = &caps[1];

            // Fast path - no animation reference at all
            if !style.contains("animation") {
                return caps[0].to_string();
            }

            // Extract animation names before stripping
            let anim_names: Vec<String> = anim_name_re
                .captures_iter(style)
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .collect();

            // Collect final-state props from referenced keyframes
            let mut final_props: HashMap<String, String> = HashMap::new();
            for name in &anim_names {
                if let Some(props) = keyframe_finals.get(name.as_str()) {
                    for (p, v) in props {
                        final_props.insert(p.clone(), v.clone());
                    }
                }
            }

            // Strip animation properties
            let cleaned = animation_prop_re.replace_all(style, "").to_string();

            // Apply final-state overrides to initial values
            let mut decls = parse_css_declarations(&cleaned);
            if !final_props.is_empty() {
                for (final_prop, final_val) in &final_props {
                    if final_prop == "transform"
                        || final_prop == "transform-origin"
                        || final_prop.starts_with("-webkit-transform")
                    {
                        continue;
                    }
                    if let Some(pos) = decls.iter().position(|(p, _)| p == final_prop) {
                        if is_initial_value(&decls[pos].1) {
                            decls[pos] = (final_prop.clone(), final_val.clone());
                        }
                    }
                }
            } else {
                // No @keyframes found but animation was present
                for decl in decls.iter_mut() {
                    if decl.0 == "opacity" && is_initial_value(&decl.1) {
                        decl.1 = "1".to_string();
                    }
                }
            }

            let result = decls
                .iter()
                .map(|(p, v)| format!("{}: {}", p, v))
                .collect::<Vec<_>>()
                .join("; ");

            format!(r#"style="{}""#, result)
        })
        .to_string()
}

// ---------------------------------------------------------------------------
// @keyframes parser: extract `to { ... }` / `100% { ... }` final states
// ---------------------------------------------------------------------------

/// Returns a map of keyframe-name -> Vec<(property, value)> for the `to` / `100%` state.
fn parse_keyframe_final_states(html: &str) -> HashMap<String, Vec<(String, String)>> {
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

/// Inside a @keyframes body, find the `to { ... }` or `100% { ... }` block and return its parsed declarations.
fn extract_final_frame(keyframe_body: &str) -> Option<Vec<(String, String)>> {
    let to_re = Regex::new(r"(?:^|\s|;|\})(to|100\s*%)\s*\{").unwrap();

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

// ---------------------------------------------------------------------------
// @keyframes remover
// ---------------------------------------------------------------------------

fn remove_keyframes(html: &str) -> String {
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

fn find_matching_brace(s: &str, start: usize) -> Option<usize> {
    let mut count = 1;
    let mut j = start;
    while j < s.len() && count > 0 {
        match s.as_bytes()[j] {
            b'{' => count += 1,
            b'}' => count -= 1,
            _ => {}
        }
        j += 1;
    }
    if count == 0 {
        Some(j)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// clamp() resolver
// ---------------------------------------------------------------------------

fn resolve_clamp_values(style: &str) -> String {
    let clamp_re = Regex::new(r"clamp\(\s*([^,]+)\s*,\s*([^,]+)\s*,\s*([^)]+)\s*\)").unwrap();
    clamp_re
        .replace_all(style, |caps: &regex::Captures| {
            let min_val = caps[1].trim();
            let preferred = caps[2].trim();
            let max_val = caps[3].trim();

            let min_px = resolve_single_value(min_val);
            let preferred_px = resolve_single_value(preferred);
            let max_px = resolve_single_value(max_val);

            let result_px = if preferred_px > 0.0 {
                preferred_px
                    .max(min_px)
                    .min(if max_px > 0.0 { max_px } else { preferred_px })
            } else if max_px > 0.0 {
                max_px
            } else if min_px > 0.0 {
                min_px
            } else {
                16.0
            };

            format!("{}px", result_px.round())
        })
        .to_string()
}

/// Convert a single CSS length value to pixels.
fn resolve_single_value(value: &str) -> f32 {
    let trimmed = value.trim();
    let numeric = extract_leading_number(trimmed);
    if numeric == 0.0 && !trimmed.starts_with('0') {
        return 0.0;
    }

    if trimmed.contains("vw") || trimmed.contains("vh") {
        return numeric * 6.0;
    }
    if trimmed.contains("rem") || trimmed.contains("em") {
        return numeric * 16.0;
    }
    if trimmed.contains("px")
        || trimmed
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
    {
        return numeric;
    }
    0.0
}

fn extract_leading_number(value: &str) -> f32 {
    let cleaned: String = value
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned.parse::<f32>().unwrap_or(0.0)
}
