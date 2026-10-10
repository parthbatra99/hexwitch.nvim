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
    // Object order matters for equal-specificity semantic rules. Duplicate keys
    // would silently overwrite an earlier rule, so reject them at every depth.
    use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
    struct StrictValue;
    impl<'de> DeserializeSeed<'de> for StrictValue {
        type Value = serde_json::Value;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            d.deserialize_any(self)
        }
    }
    impl<'de> Visitor<'de> for StrictValue {
        type Value = serde_json::Value;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("JSON with unique object keys")
        }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(v.into())
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(v.into())
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v.into())
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
            serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .ok_or_else(|| E::custom("nonfinite JSON number"))
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.into())
        }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(v.into())
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(serde_json::Value::Null)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut values = vec![];
            while let Some(value) = a.next_element_seed(StrictValue)? {
                values.push(value);
            }
            Ok(values.into())
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut object = serde_json::Map::new();
            while let Some(key) = a.next_key::<String>()? {
                if object.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate JSON key {key}"
                    )));
                }
                object.insert(key, a.next_value_seed(StrictValue)?);
            }
            Ok(serde_json::Value::Object(object))
        }
    }
    let mut deserializer = serde_json::Deserializer::from_str(&out);
    let value =
        DeserializeSeed::deserialize(StrictValue, &mut deserializer).map_err(|e| e.to_string())?;
    deserializer.end().map_err(|e| e.to_string())?;
    Ok(value)
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
        assert!(parse(r#"{"a":1,"a":2}"#).unwrap_err().contains("duplicate"));
        assert!(parse(r#"{"nested":{"a":1,"a":2}}"#).is_err());
        assert!(parse("{} {}").is_err());
    }
}
