//! CSS parsing utilities

/// Parse CSS declarations from a style string
/// Handles nested parentheses and quoted strings properly
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

/// Parse a CSS value to extract numeric component.
/// Handles px, %, rem, em, vw, vh units.
/// e.g., "-120px" -> -120.0, "50%" -> 50.0, "5rem" -> 80.0
pub fn parse_css_value(value: &str) -> f32 {
    let trimmed = value.trim();

    // Handle calc() - just grab the first numeric value as a rough approximation
    if trimmed.starts_with("calc(") {
        let inner = &trimmed[5..trimmed.len().saturating_sub(1)];
        return parse_css_value(inner);
    }

    let cleaned: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    let numeric = cleaned.parse::<f32>().unwrap_or(0.0);

    // Convert rem/em to px (1rem ~= 16px)
    if trimmed.contains("rem") || trimmed.contains("em") {
        return numeric * 16.0;
    }

    numeric
}
