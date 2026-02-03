//! Stage 2: Pseudo-element expansion

use crate::utils::sanitizer::css::parser::parse_css_declarations;
use crate::utils::sanitizer::types::PseudoRule;
use regex::Regex;

/// Expands CSS ::before and ::after pseudo-elements into explicit HTML <span> elements and adds corresponding CSS rules for those spans.
///
/// This function scans the first <style> block in the provided HTML for rules targeting `.class::before` and `.class::after`. For each pseudo rule that declares `content`, it:
/// - removes the pseudo rule from the original CSS and emits an equivalent CSS rule for a generated helper class,
/// - inserts a <span> carrying the helper class into the HTML either immediately after the element's opening tag for `::before` or immediately before the element's closing tag for `::after`.
///
/// If there is no <style> block or no pseudo rules with `content`, the original HTML is returned unchanged. Pseudo rules that omit `content` are ignored. When no `display` declaration is provided for a pseudo rule, the helper class defaults to `display: inline-block`.
///
/// # Examples
///
/// ```
/// let html = r#"<style>.badge::before { content: "*"; color: red; }</style><span class="badge">New</span>"#;
/// let out = expand_pseudo_elements(html);
/// // The output should contain an injected span representing the ::before content and a generated CSS rule for its helper class.
/// assert!(out.contains(r#"<span class="__pseudo_badge__before">*"</#) || out.contains("__pseudo_badge__before"));
/// ```
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
            if let Some(ph_pos) = result.find(&placeholder) {
                let after_ph = ph_pos + placeholder.len();
                // Find the next `>` after the placeholder.
                if let Some(gt_offset) = result[after_ph..].find('>') {
                    let insert_pos = after_ph + gt_offset + 1; // right after >
                    result = format!(
                        "{}{}{}",
                        &result[..ph_pos], // everything before placeholder
                        &result[after_ph..insert_pos], // rest of tag
                        &format!("{}{}", &span, &result[insert_pos..])  // span + rest
                    );
                }
            }
        } else {
            let open_full_re = Regex::new(&format!(
                r#"(?s)<([a-zA-Z][a-zA-Z0-9]*)\s[^>]*class="[^"]*\b{}\b[^"]*"[^>]*>"#,
                regex::escape(&rule.class)
            ))
            .expect("invalid full open tag regex");

            if let Some(caps) = open_full_re.captures(&result) {
                let tag_name = &caps[1];
                let open_end = caps.get(0).unwrap().end();
                let closing = format!("</{}>", tag_name);
                if let Some(close_offset) = result[open_end..].find(&closing) {
                    let insert_pos = open_end + close_offset;
                    result = format!("{}{}{}", &result[..insert_pos], span, &result[insert_pos..]);
                }
            }
        }
    }

    result
}

/// Parses CSS and extracts pseudo-element rules (::before and ::after), returning a list of
/// PseudoRule entries describing how to render those pseudo-elements and a cleaned CSS string
/// with all pseudo-rule blocks removed.
///
/// The returned `Vec<PseudoRule>` contains one entry per `.class::before` or `.class::after`
/// rule that includes a `content` declaration. Each `PseudoRule` carries:
/// - `class`: the target class name (without the leading dot),
/// - `is_before`: `true` for `::before`, `false` for `::after`,
/// - `content`: the unquoted `content` value,
/// - `style`: concatenated other declarations (guaranteed to include `display` if none was present),
/// - `class_for_style`: a generated helper class name used for emitting concrete CSS rules.
///
/// The cleaned CSS is the original CSS with all matched pseudo-element rule blocks removed.
/// Rules that lack a `content` declaration are ignored and not returned.
///
/// # Examples
///
/// ```
/// use crate::utils::sanitizer::stages::stage2_pseudo::parse_pseudo_rules;
/// use crate::utils::sanitizer::types::PseudoRule;
///
/// let css = r#"
/// .foo, .bar::before { color: red; }
/// .foo::before { content: "X"; color: blue; }
/// .baz::after { content: 'Y'; }
/// "#;
///
/// let (rules, cleaned) = parse_pseudo_rules(css);
/// assert_eq!(rules.len(), 2);
/// assert!(cleaned.contains(".foo, .bar::before") == false); // pseudo rules removed
/// assert!(!cleaned.contains("::before") && !cleaned.contains("::after"));
///
/// // Inspect one returned rule
/// let r = &rules[0];
/// assert!(r.content == "X" || r.content == "Y");
/// ```
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