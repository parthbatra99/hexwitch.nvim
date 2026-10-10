//! Small JSONC lexer: comments and trailing commas, never regex-strips strings.
pub fn parse(input: &str) -> Result<serde_json::Value, String> {
    if input.len() > 8 * 1024 * 1024 {
        return Err("JSONC exceeds 8 MiB".into());
    }
    let mut chars = input.chars().peekable();
    let mut clean = String::with_capacity(input.len());
    let mut string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if string {
            clean.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                string = false;
            }
            continue;
        }
        if c == '"' {
            string = true;
            clean.push(c);
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next in chars.by_ref() {
                if next == '\n' {
                    clean.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut closed = false;
            while let Some(next) = chars.next() {
                if next == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    closed = true;
                    break;
                }
                if next == '\n' {
                    clean.push('\n');
                }
            }
            if !closed {
                return Err("unterminated JSONC comment".into());
            }
            clean.push(' ');
        } else {
            clean.push(c);
        }
    }
    let mut out = String::with_capacity(clean.len());
    let mut iter = clean.chars().peekable();
    string = false;
    escape = false;
    while let Some(c) = iter.next() {
        if string {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                string = false;
            }
            continue;
        }
        if c == '"' {
            string = true;
        }
        if c == ',' {
            let rest = iter.clone().find(|c| !c.is_whitespace());
            if matches!(rest, Some('}') | Some(']')) {
                continue;
            }
        }
        out.push(c);
    }
    serde_json::from_str(&out).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strings_and_comments_do_not_collide() {
        let value = parse("{/* hi */\"url\":\"https://x//y\",\"list\":[1,],}//done").unwrap();
        assert_eq!(value["url"], "https://x//y");
        assert_eq!(value["list"], serde_json::json!([1]));
        assert!(parse("{/* unclosed").is_err());
    }
}
