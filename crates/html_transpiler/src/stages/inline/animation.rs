//! Animation property stripping and rule body patching.

use std::collections::HashMap;

use crate::css::parser::parse_css_declarations;
use crate::stages::inline::{STYLE_ATTR_RE, ANIM_PROP_RE, ANIM_NAME_RE, parse_keyframe_final_states};

/// Strip animation properties from inline styles in HTML.
pub fn strip_animation_from_inline_styles(html: &str) -> String {
    let keyframe_finals = parse_keyframe_final_states(html);

    let style_re = &*STYLE_ATTR_RE;
    let animation_prop_re = &*ANIM_PROP_RE;
    let anim_name_re = &*ANIM_NAME_RE;

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

/// Check if a CSS value is an initial/reset value.
pub fn is_initial_value(val: &str) -> bool {
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

/// Patch a single CSS rule body with final-state property overrides.
pub fn patch_rule_body(body: &str, final_props: &HashMap<String, String>) -> String {
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

    // Rebuild with newlines for readability
    patched
        .iter()
        .map(|(p, v)| format!("  {}: {}", p, v))
        .collect::<Vec<_>>()
        .join(";\n")
        + ";\n"
}
