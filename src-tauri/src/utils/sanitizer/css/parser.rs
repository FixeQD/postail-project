//! CSS parsing utilities used throughout the sanitizer pipeline.

/// Parse CSS declarations from a `style=""` string into `(property, value)` pairs.
/// Handles nested parentheses (e.g. `url(...)`, `calc(...)`) and quoted strings.
pub fn parse_css_declarations(style: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut paren_depth: i32 = 0;
    let mut in_string = false;
    let mut string_char = '"';

    for ch in style.chars() {
        match ch {
            '"' | '\'' if !in_string => {
                in_string = true;
                string_char = ch;
                current.push(ch);
            }
            c if in_string && c == string_char => {
                in_string = false;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(ch);
            }
            ';' if !in_string && paren_depth == 0 => {
                push_decl(&current, &mut out);
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    push_decl(&current, &mut out);
    out
}

fn push_decl(raw: &str, out: &mut Vec<(String, String)>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Some(colon) = trimmed.find(':') {
        let prop = trimmed[..colon].trim().to_lowercase();
        let val = trimmed[colon + 1..].trim().to_string();
        if !prop.is_empty() {
            out.push((prop, val));
        }
    }
}

/// Extract the leading numeric component from a CSS value string.
///
/// Handles `px`, `%`, `em`, `rem`, `vw`, `vh` units.
/// `rem`/`em` are converted at 16 px per unit. `calc()` returns the first
/// numeric value found as a rough approximation.
pub fn parse_css_value(value: &str) -> f32 {
    let s = value.trim();
    if s.starts_with("calc(") {
        let inner = &s[5..s.len().saturating_sub(1)];
        return parse_css_value(inner);
    }

    let digits: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    let n = digits.parse::<f32>().unwrap_or(0.0);

    if s.contains("rem") || (s.contains("em") && !s.contains("rem")) {
        n * 16.0
    } else {
        n
    }
}
