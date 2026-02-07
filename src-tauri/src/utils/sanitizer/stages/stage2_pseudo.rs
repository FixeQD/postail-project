//! Stage 2: Pseudo-element expansion

use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::PseudoRule;
use regex::Regex;
use std::collections::HashSet;

/// HTML-escape content for safe interpolation
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn find_matching_brace(css: &str, start: usize) -> Option<usize> {
    let mut count = 1;
    let mut j = start;
    let mut in_string = false;
    let mut string_quote = '\0';
    let mut escaped = false;
    let mut in_comment = false;

    while j < css.len() && count > 0 {
        let ch = css.as_bytes()[j] as char;

        if escaped {
            escaped = false;
            j += 1;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            j += 1;
            continue;
        }

        if in_comment {
            if ch == '*' && j + 1 < css.len() && css.as_bytes()[j + 1] as char == '/' {
                in_comment = false;
                j += 2;
                continue;
            }
            j += 1;
            continue;
        }

        if in_string {
            if ch == string_quote {
                in_string = false;
            }
            j += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = true;
            string_quote = ch;
            j += 1;
            continue;
        }

        if ch == '/' && j + 1 < css.len() && css.as_bytes()[j + 1] as char == '*' {
            in_comment = true;
            j += 2;
            continue;
        }

        match ch {
            '{' => count += 1,
            '}' => count -= 1,
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

pub fn expand_pseudo_elements(html: &str) -> String {
    let style_re = Regex::new(r"(?s)(<style[^>]*>)(.*?)(</style>)").unwrap();
    let mut all_rules = Vec::new();
    let mut style_replacements = Vec::new();

    // First pass: collect all style blocks and their rules
    for style_caps in style_re.captures_iter(html) {
        let style_open = &style_caps[1];
        let style_body = &style_caps[2];
        let style_close = &style_caps[3];
        let style_full = style_caps.get(0).unwrap();

        let (rules, cleaned_css) = parse_pseudo_rules(style_body);

        let mut new_css_rules = String::new();
        for rule in &rules {
            if !rule.style.is_empty() {
                new_css_rules.push_str(&format!(
                    "\n.{} {{ {} }}\n",
                    rule.class_for_style, rule.style
                ));
            }
        }

        let new_style = format!(
            "{}{}{}{}",
            style_open, cleaned_css, new_css_rules, style_close
        );

        style_replacements.push((style_full.start(), style_full.end(), new_style));
        all_rules.extend(rules);
    }

    if all_rules.is_empty() {
        return html.to_string();
    }

    // Apply all style replacements in reverse order to preserve positions
    let mut result = html.to_string();
    for (start, end, new_style) in style_replacements.iter().rev() {
        result.replace_range(*start..*end, new_style);
    }

    // Cache for compiled regexes to avoid repeated compilation
    let mut regex_cache: std::collections::HashMap<String, Regex> =
        std::collections::HashMap::new();

    // Merge duplicate pseudo rules for the same class+pseudo kind.
    let all_rules = merge_pseudo_rules(all_rules);

    for rule in &all_rules {
        let open_tag_pattern = format!(
            r#"(?s)(<[a-zA-Z][a-zA-Z0-9]*\s[^>]*class=")([^"]*\b{class}\b[^"]*)"#,
            class = regex::escape(&rule.class)
        );
        let open_tag_re = regex_cache
            .entry(open_tag_pattern.clone())
            .or_insert_with(|| Regex::new(&open_tag_pattern).expect("invalid class-match regex"));

        let span = if rule.content.is_empty() {
            format!(r#"<span class="{}"></span>"#, rule.class_for_style)
        } else {
            let escaped_content = html_escape(&rule.content);
            format!(
                r#"<span class="{}">{}</span>"#,
                rule.class_for_style, escaped_content
            )
        };

        if rule.is_before {
            result = open_tag_re
                .replace_all(&result, |caps: &regex::Captures| {
                    // Verify class is a complete token, not part of a hyphenated name
                    let class_value = &caps[2];
                    if !class_value.split_whitespace().any(|t| t == rule.class) {
                        return caps[0].to_string();
                    }
                    format!(
                        "{}{}__PSEUDO_BEFORE_{}__",
                        &caps[1], &caps[2], rule.class_for_style
                    )
                })
                .to_string();

            let placeholder = format!("__PSEUDO_BEFORE_{}__", rule.class_for_style);
            let mut pos = 0;
            while let Some(ph_offset) = result[pos..].find(&placeholder) {
                let ph_pos = pos + ph_offset;
                let after_ph = ph_pos + placeholder.len();
                if let Some(gt_offset) = result[after_ph..].find('>') {
                    let insert_pos = after_ph + gt_offset + 1;
                    result = format!(
                        "{}{}{}{}",
                        &result[..ph_pos],
                        &result[after_ph..insert_pos],
                        &span,
                        &result[insert_pos..]
                    );
                    pos = ph_pos + span.len();
                } else {
                    pos = ph_pos + 1;
                }
            }
        } else {
            let open_full_pattern = format!(
                r#"(?s)<([a-zA-Z][a-zA-Z0-9]*)\s[^>]*class="([^"]*\b{class}\b[^"]*)"[^>]*>"#,
                class = regex::escape(&rule.class)
            );
            let open_full_re = regex_cache
                .entry(open_full_pattern.clone())
                .or_insert_with(|| {
                    Regex::new(&open_full_pattern).expect("invalid full open tag regex")
                });

            let mut insertions = Vec::new();
            for caps in open_full_re.captures_iter(&result) {
                let class_value = &caps[2];
                if !class_value.split_whitespace().any(|t| t == rule.class) {
                    continue;
                }
                let tag_name = &caps[1];
                let open_end = caps.get(0).unwrap().end();
                let closing = format!("</{}>", tag_name);

                // Nesting-aware scan: count opening/closing tags to find the matching outer closing tag
                let mut depth = 1;
                let mut pos = open_end;
                let mut insert_pos = None;

                let open_tag_re =
                    Regex::new(&format!(r#"(?s)<{}(?:\s|>)"#, regex::escape(tag_name))).unwrap();

                while pos < result.len() && insert_pos.is_none() {
                    // Look for next opening tag of the same type
                    if let Some(open_match) = open_tag_re.find_at(&result, pos) {
                        let open_start = open_match.start();
                        // Look for closing tag
                        if let Some(close_offset) = result[pos..].find(&closing) {
                            let close_start = pos + close_offset;
                            if open_start < close_start {
                                // Opening tag comes first - check if it's self-closing
                                let tag_end = result[open_start..].find('>');
                                if let Some(end) = tag_end {
                                    let tag_content = &result[open_start..open_start + end + 1];
                                    if tag_content.ends_with("/>") {
                                        // Self-closing tag, skip it
                                        pos = open_start + end + 1;
                                    } else {
                                        // Nested opening tag
                                        depth += 1;
                                        pos = open_start + end + 1;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                // Closing tag comes first
                                depth -= 1;
                                if depth == 0 {
                                    insert_pos = Some(close_start);
                                }
                                pos = close_start + closing.len();
                            }
                        } else {
                            break;
                        }
                    } else {
                        // No more opening tags, find the closing tag
                        if let Some(close_offset) = result[pos..].find(&closing) {
                            depth -= 1;
                            if depth == 0 {
                                insert_pos = Some(pos + close_offset);
                            }
                        }
                        break;
                    }
                }

                if let Some(pos) = insert_pos {
                    insertions.push((pos, span.clone()));
                }
            }
            insertions.sort_by(|a, b| b.0.cmp(&a.0)); // reverse order
            for (insert_pos, span) in insertions {
                result = format!("{}{}{}", &result[..insert_pos], span, &result[insert_pos..]);
            }
        }
    }

    result
}

fn parse_pseudo_rules(css: &str) -> (Vec<PseudoRule>, String) {
    // ── Step 1: brace-counting parser to handle nested blocks and at-rules ─────────────────
    let mut expanded_css = String::new();
    let mut i = 0;

    let mut pseudo_selectors: Vec<(String, String, usize, usize)> = Vec::new();

    while i < css.len() {
        // Find the next opening brace
        if let Some(open_offset) = css[i..].find('{') {
            let selector_start = i;
            let brace_start = i + open_offset;
            let selector_part = css[selector_start..brace_start].trim();

            // Skip at-rules (like @media) - treat as opaque blocks
            if selector_part.starts_with("@") {
                if let Some(end) = find_matching_brace(css, brace_start + 1) {
                    // Keep the at-rule unchanged
                    expanded_css.push_str(&css[selector_start..end]);
                    expanded_css.push('\n');
                    i = end;
                } else {
                    // Malformed, skip selector
                    expanded_css.push_str(&css[selector_start..brace_start]);
                    i = brace_start;
                }
                continue;
            }

            if let Some(end) = find_matching_brace(css, brace_start + 1) {
                // Found complete rule block
                let body = css[brace_start + 1..end - 1].trim();
                let _expanded_start = expanded_css.len();

                let has_pseudo =
                    selector_part.contains("::before") || selector_part.contains("::after");
                if !has_pseudo {
                    // Keep unchanged
                    expanded_css.push_str(&css[selector_start..end]);
                    expanded_css.push('\n');
                } else {
                    // Expand grouped selectors
                    for fragment in selector_part.split(',') {
                        let fragment = fragment.trim();
                        if fragment.is_empty() {
                            continue;
                        }
                        let rule_start = expanded_css.len();
                        let rule_text = format!("{} {{\n{}\n}}\n", fragment, body);
                        expanded_css.push_str(&rule_text);
                        if fragment.contains("::before") || fragment.contains("::after") {
                            pseudo_selectors.push((
                                fragment.to_string(),
                                body.to_string(),
                                rule_start,
                                expanded_css.len(),
                            ));
                        }
                    }
                }
                i = end;
            } else {
                // Malformed - unmatched brace
                expanded_css.push_str(&css[selector_start..brace_start]);
                i = brace_start;
            }
        } else {
            // No more braces - copy rest
            expanded_css.push_str(&css[i..]);
            break;
        }
    }

    // ── Step 2: merge CSS bodies for same class+pseudo, THEN extract rules ─
    let pseudo_re = Regex::new(r"^\s*\.([\w-]+)::(before|after)\s*$").unwrap();

    // Group by (class, pseudo_kind) and merge CSS bodies
    let mut grouped: std::collections::HashMap<(String, bool), Vec<String>> =
        std::collections::HashMap::new();

    for (selector, body, _, _) in &pseudo_selectors {
        if let Some(caps) = pseudo_re.captures(selector) {
            let class = caps[1].to_string();
            let is_before = &caps[2] == "before";
            grouped
                .entry((class, is_before))
                .or_default()
                .push(body.to_string());
        }
    }

    let mut rules = Vec::new();

    // Positioning props that don't work in email - strip from pseudo spans
    const PSEUDO_STRIP_PROPS: &[&str] = &[
        "position",
        "top",
        "left",
        "right",
        "bottom",
        "inset",
        "z-index",
        "transform",
        "transform-origin",
    ];

    for ((class, is_before), bodies) in &grouped {
        // Merge all CSS declarations for this class+pseudo, later overrides earlier
        let mut merged_decls: Vec<(String, String)> = Vec::new();
        for body in bodies {
            for (prop, val) in parse_css_declarations(body) {
                if let Some(pos) = merged_decls.iter().position(|(p, _)| *p == prop) {
                    merged_decls[pos] = (prop, val);
                } else {
                    merged_decls.push((prop, val));
                }
            }
        }

        // Extract content and build style
        let mut content = String::new();
        let mut has_content_decl = false;
        let mut style_parts: Vec<String> = Vec::new();
        let mut has_display = false;

        for (prop, val) in &merged_decls {
            if prop.eq_ignore_ascii_case("content") {
                has_content_decl = true;
                content = val
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
            } else if PSEUDO_STRIP_PROPS.iter().any(|&p| p == prop.as_str()) {
                // Skip positioning props - can't use them in email
                continue;
            } else {
                style_parts.push(format!("{}: {}", prop, val));
                if prop.eq_ignore_ascii_case("display") {
                    has_display = true;
                }
            }
        }

        if !has_display {
            style_parts.push("display: inline-block".to_string());
        }

        if !has_content_decl {
            continue;
        }

        let pseudo_kind = if *is_before { "before" } else { "after" };
        let class_for_style = format!("__pseudo_{}__{}", class, pseudo_kind);
        let style_body = style_parts.join("; ");

        rules.push(PseudoRule {
            class: class.clone(),
            is_before: *is_before,
            content,
            style: style_body,
            class_for_style,
        });
    }

    // Remove only the pseudo rules that we actually expanded into `rules`.
    let mut expanded_selectors: HashSet<String> = HashSet::new();
    for rule in &rules {
        let sel = format!(
            ".{}::{}",
            rule.class,
            if rule.is_before { "before" } else { "after" }
        );
        expanded_selectors.insert(sel);
    }

    let mut ranges_to_remove: Vec<(usize, usize)> = pseudo_selectors
        .into_iter()
        .filter(|(selector, _, _, _)| expanded_selectors.contains(selector))
        .map(|(_, _, start, end)| (start, end))
        .collect();
    ranges_to_remove.sort_by(|a, b| b.0.cmp(&a.0));

    let mut cleaned_css = expanded_css;
    for (start, end) in ranges_to_remove {
        if start < cleaned_css.len() && end <= cleaned_css.len() {
            cleaned_css.replace_range(start..end, "");
        }
    }

    (rules, cleaned_css)
}

/// Merge pseudo rules that target the same class + pseudo kind (before/after)
fn merge_pseudo_rules(rules: Vec<PseudoRule>) -> Vec<PseudoRule> {
    let mut merged: Vec<PseudoRule> = Vec::new();

    for rule in rules {
        let key_matches = merged.iter().position(|existing| {
            existing.class == rule.class && existing.is_before == rule.is_before
        });

        if let Some(idx) = key_matches {
            // Merge: parse both style strings, later properties override
            let existing_decls = parse_css_declarations(&merged[idx].style);
            let new_decls = parse_css_declarations(&rule.style);

            let mut combined: Vec<(String, String)> = existing_decls;
            for (prop, val) in new_decls {
                if let Some(pos) = combined.iter().position(|(p, _)| *p == prop) {
                    combined[pos] = (prop, val);
                } else {
                    combined.push((prop, val));
                }
            }

            merged[idx].style = combined
                .iter()
                .map(|(p, v)| format!("{}: {}", p, v))
                .collect::<Vec<_>>()
                .join("; ");

            // If the new rule has content and the existing one doesn't, take it
            if !rule.content.is_empty() && merged[idx].content.is_empty() {
                merged[idx].content = rule.content;
            }
        } else {
            merged.push(rule);
        }
    }

    merged
}
