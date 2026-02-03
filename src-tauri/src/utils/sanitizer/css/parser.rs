/// Parse CSS declarations from a style string into (property, value) pairs.
///
/// Splits declarations on semicolons that are not inside quoted strings or
/// parentheses, correctly handling nested parentheses and quoted strings.
/// Property names are lowercased; declarations that cannot be split by a
/// colon (`:`) are ignored.
///
/// # Examples
///
/// ```
/// let s = r#"color: red; background: url("data:image/png;base64,AAA;BBB"); padding: 10px"#;
/// let decls = parse_css_declarations(s);
/// assert_eq!(decls.iter().find(|(k, _)| k == "color").map(|(_, v)| v.as_str()), Some("red"));
/// assert!(decls.iter().any(|(k, v)| k == "background" && v.contains("data:image/png")));
/// assert_eq!(decls.iter().find(|(k, _)| k == "padding").map(|(_, v)| v.as_str()), Some("10px"));
/// ```
pub fn parse_css_declarations(style: &str) -> Vec<(String, String)> {
    let mut declarations = Vec::new();
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
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    if let Some(colon) = trimmed.find(':') {
                        let prop = trimmed[..colon].trim().to_lowercase();
                        let val = trimmed[colon + 1..].trim().to_string();
                        declarations.push((prop, val));
                    }
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        if let Some(colon) = trimmed.find(':') {
            let prop = trimmed[..colon].trim().to_lowercase();
            let val = trimmed[colon + 1..].trim().to_string();
            declarations.push((prop, val));
        }
    }

    declarations
}

/// Extracts the leading numeric component from a CSS value string.
///
/// Stops parsing at the first character that is not a digit, a decimal point, or a leading minus sign.
/// Returns the parsed number, or `0.0` if no valid numeric prefix is found or parsing fails.
///
/// # Examples
///
/// ```
/// assert_eq!(parse_css_value("-120px"), -120.0);
/// assert_eq!(parse_css_value("50%"), 50.0);
/// assert_eq!(parse_css_value("12.5em"), 12.5);
/// assert_eq!(parse_css_value("none"), 0.0);
/// ```
pub fn parse_css_value(value: &str) -> f32 {
    let cleaned: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned.parse::<f32>().unwrap_or(0.0)
}