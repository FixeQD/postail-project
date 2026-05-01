//! Brace matching utility for CSS parsing.

pub fn find_matching_brace(css: &str, start: usize) -> Option<usize> {
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
