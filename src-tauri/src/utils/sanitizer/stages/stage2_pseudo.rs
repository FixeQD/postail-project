//! Stage 2: Pseudo-element expansion

use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::PseudoRule;
use regex::Regex;

pub fn expand_pseudo_elements(html: &str) -> String {
    let style_re = Regex::new(r"(?s)(<style[^>]*>)(.*?)(</style>)").unwrap();
    let style_caps = match style_re.captures(html) {
        Some(c) => c,
        None => return html.to_string(), // no <style> → nothing to do
    };

    let style_open = &style_caps[1];
    let style_body = &style_caps[2];
    let style_close = &style_caps[3];
    let style_full = style_caps.get(0).unwrap();

    let (rules, cleaned_css) = parse_pseudo_rules(style_body);
    if rules.is_empty() {
        return html.to_string();
    }

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
    let mut result = html[..style_full.start()].to_string();
    result.push_str(&new_style);
    result.push_str(&html[style_full.end()..]);

    for rule in &rules {
        let open_tag_re = Regex::new(&format!(
            r#"(?s)(<[a-zA-Z][a-zA-Z0-9]*\s[^>]*class=")([^"]*\b{}\b[^"]*)"#,
            regex::escape(&rule.class)
        ))
        .expect("invalid class-match regex");

        let span = if rule.content.is_empty() {
            format!(
                r#"<span class="{}" style="display: inline-block"></span>"#,
                rule.class_for_style
            )
        } else {
            format!(
                r#"<span class="{}">{}</span>"#,
                rule.class_for_style, rule.content
            )
        };

        if rule.is_before {
            result = open_tag_re
                .replace_all(&result, |caps: &regex::Captures| {
                    let _match_end = caps.get(0).unwrap().end();
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
                if let Some(close_offset) = result[open_end..].find(&closing) {
                    let insert_pos = open_end + close_offset;
                    insertions.push((insert_pos, span.clone()));
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
    // ── Step 1: split grouped selectors ─────────────────────────────────────
    let rule_re = Regex::new(r"(?s)([^{]+)\{([^}]*)\}").expect("invalid rule block regex");

    let mut expanded_css = String::new();
    let mut last_end = 0;

    for caps in rule_re.captures_iter(css) {
        let full = caps.get(0).unwrap();
        expanded_css.push_str(&css[last_end..full.start()]);
        last_end = full.end();

        let selector_part = &caps[1];
        let body = &caps[2];

        let has_pseudo = selector_part.contains("::before") || selector_part.contains("::after");
        if !has_pseudo {
            expanded_css.push_str(&caps[0]);
            continue;
        }

        for fragment in selector_part.split(',') {
            let fragment = fragment.trim();
            if fragment.is_empty() {
                continue;
            }
            expanded_css.push_str(&format!("{} {{\n{}\n}}\n", fragment, body));
        }
    }
    expanded_css.push_str(&css[last_end..]);

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
            if prop == "content" {
                has_content_decl = true;
                content = val
                    .trim_matches(|c: char| c == '"' || c == '\'')
                    .to_string();
            } else {
                style_parts.push(format!("{}: {}", prop, val));
                if prop == "display" {
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
