//! Stage 2: Pseudo-element expansion

use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::PseudoRule;
use regex::Regex;

/// HTML-escape content for safe interpolation
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

    for rule in &all_rules {
        let open_tag_re = Regex::new(&format!(
            r#"(?s)(<[a-zA-Z][a-zA-Z0-9]*\s[^>]*class=")([^"]*\b{}\b[^"]*)"#,
            regex::escape(&rule.class)
        ))
        .expect("invalid class-match regex");

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
                    format!(
                        "{}\"{}__PSEUDO_BEFORE_{}__",
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
            let open_full_re = Regex::new(&format!(
                r#"(?s)<([a-zA-Z][a-zA-Z0-9]*)\s[^>]*class="[^"]*\b{}\b[^"]*"[^>]*>"#,
                regex::escape(&rule.class)
            ))
            .expect("invalid full open tag regex");

            let mut insertions = Vec::new();
            for caps in open_full_re.captures_iter(&result) {
                let tag_name = &caps[1];
                let open_end = caps.get(0).unwrap().end();
                let closing = format!("</{}>", tag_name);

                // Nesting-aware scan: count opening/closing tags to find the matching outer closing tag
                let mut depth = 1;
                let mut pos = open_end;
                let mut insert_pos = None;

                let open_tag_re =
                    Regex::new(&format!(r#"(?s)<{}(?:\s|>))"#, regex::escape(tag_name))).unwrap();

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

    while i < css.len() {
        // Find the next opening brace
        if let Some(open_offset) = css[i..].find('{') {
            let selector_start = i;
            let brace_start = i + open_offset;
            let selector_part = css[selector_start..brace_start].trim();

            // Skip at-rules (like @media) - treat as opaque blocks
            if selector_part.starts_with("@") {
                let mut count = 1;
                let mut j = brace_start + 1;
                while j < css.len() && count > 0 {
                    match css.as_bytes()[j] {
                        b'{' => count += 1,
                        b'}' => count -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                if count == 0 {
                    // Keep the at-rule unchanged
                    expanded_css.push_str(&css[selector_start..j]);
                    expanded_css.push('\n');
                    i = j;
                } else {
                    // Malformed, skip selector
                    expanded_css.push_str(&css[selector_start..brace_start]);
                    i = brace_start;
                }
                continue;
            }

            // Count braces to find the matching closing brace
            // Track context: strings, comments, and escaped characters
            let mut count = 1;
            let mut j = brace_start + 1;
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
                    // Check for comment end */
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

                // Not in string or comment
                if ch == '"' || ch == '\'' {
                    in_string = true;
                    string_quote = ch;
                    j += 1;
                    continue;
                }

                // Check for comment start /*
                if ch == '/' && j + 1 < css.len() && css.as_bytes()[j + 1] as char == '*' {
                    in_comment = true;
                    j += 2;
                    continue;
                }

                // Only count braces when not in string or comment
                match ch {
                    '{' => count += 1,
                    '}' => count -= 1,
                    _ => {}
                }
                j += 1;
            }

            if count == 0 {
                // Found complete rule block
                let body = css[brace_start + 1..j - 1].trim();

                let has_pseudo =
                    selector_part.contains("::before") || selector_part.contains("::after");
                if !has_pseudo {
                    // Keep unchanged
                    expanded_css.push_str(&css[selector_start..j]);
                    expanded_css.push('\n');
                } else {
                    // Expand grouped selectors
                    for fragment in selector_part.split(',') {
                        let fragment = fragment.trim();
                        if fragment.is_empty() {
                            continue;
                        }
                        expanded_css.push_str(&format!("{} {{\n{}\n}}\n", fragment, body));
                    }
                }
                i = j;
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

    // ── Step 2: parse individual .class::pseudo { … } rules ─────────────────
    let pseudo_re = Regex::new(r"(?s)\.([\w-]+)::(before|after)\s*\{([^}]*)\}")
        .expect("invalid pseudo rule regex");

    let mut rules = Vec::new();

    for caps in pseudo_re.captures_iter(&expanded_css) {
        let class = caps[1].to_string();
        let is_before = &caps[2] == "before";
        let body = &caps[3];

        let decls = parse_css_declarations(body);

        let mut content = String::new();
        let mut style_parts: Vec<String> = Vec::new();
        let mut has_content_decl = false;

        let mut has_display = false;

        for (prop, val) in &decls {
            if prop.eq_ignore_ascii_case("content") {
                has_content_decl = true;
                content = val
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
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

        let pseudo_kind = if is_before { "before" } else { "after" };
        let class_for_style = format!("__pseudo_{}__{}", class, pseudo_kind);
        let style_body = style_parts.join("; ");

        rules.push(PseudoRule {
            class,
            is_before,
            content,
            style: style_body,
            class_for_style,
        });
    }

    // Remove all pseudo rules from the CSS.
    let cleaned_css = pseudo_re.replace_all(&expanded_css, "").to_string();

    (rules, cleaned_css)
}
