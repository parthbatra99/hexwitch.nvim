//! Explicit scope-stack probes for the documented VS Code import subset.
//! This is rule resolution over supplied scopes, not a TextMate tokenizer.
use crate::ir::{Property, Style};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct ScopeResolution {
    pub style: Style,
    pub property_rules: BTreeMap<Property, usize>,
    pub unsupported_selectors: Vec<String>,
    pub limitations: Vec<String>,
}

fn valid_scope(scope: &str) -> bool {
    !scope.is_empty()
        && scope != "-"
        && scope.len() <= 512
        && scope.split('.').all(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
}
fn prefix(selector: &str, scope: &str) -> bool {
    scope == selector
        || scope
            .strip_prefix(selector)
            .is_some_and(|tail| tail.starts_with('.'))
}
fn score(selector: &str, stack: &[String]) -> Option<usize> {
    let parts: Vec<_> = selector.split_whitespace().collect();
    if parts.is_empty() || !parts.iter().all(|p| valid_scope(p)) {
        return None;
    }
    let mut next = 0;
    for part in &parts {
        let found = (next..stack.len()).find(|i| prefix(part, &stack[*i]))?;
        next = found + 1;
    }
    Some(parts.len() * 0x10000 + parts.last().unwrap().len())
}

/// Probe the retained source document of an imported theme. Unsupported
/// selectors qualify the result: they may override these calculated values.
pub fn resolve_scope_stack(
    source: &Value,
    stack: &[String],
    base: &Style,
) -> Result<ScopeResolution, String> {
    base.validate()?;
    if stack.is_empty() || stack.len() > 64 || !stack.iter().all(|s| valid_scope(s)) {
        return Err("invalid scope stack".into());
    }
    let rules = source
        .get("tokenColors")
        .and_then(Value::as_array)
        .ok_or("source tokenColors must be an array")?;
    if rules.len() > 4096 {
        return Err("scope rule count limit".into());
    }
    let mut result = ScopeResolution {
        style: base.clone(),
        property_rules: BTreeMap::new(),
        unsupported_selectors: vec![],
        limitations: vec![],
    };
    let mut ranks = BTreeMap::new();
    for (index, rule) in rules.iter().enumerate() {
        let settings = rule
            .get("settings")
            .and_then(Value::as_object)
            .ok_or("scope settings must be an object")?;
        let mut patch = Style::new();
        for (field, value) in settings {
            match field.as_str() {
                "foreground" | "background" => {
                    let (color, alpha) = crate::semantic::parse_color(
                        value.as_str().ok_or("token color must be string")?,
                    )?;
                    if field == "foreground" {
                        patch.fg = Some(color);
                        patch.fg_opacity = Some(alpha.unwrap_or(1.0));
                    } else {
                        patch.bg = Some(color);
                        patch.bg_opacity = Some(alpha.unwrap_or(1.0));
                    }
                }
                "fontStyle" => {
                    let font = crate::semantic::font_style(
                        value.as_str().ok_or("fontStyle must be string")?,
                    )?;
                    for p in [
                        Property::Bold,
                        Property::Italic,
                        Property::Underline,
                        Property::Strikethrough,
                    ] {
                        patch.copy_property_from(&font, p);
                    }
                }
                _ => return Err(format!("unknown token settings property {field}")),
            }
        }
        let selectors: Vec<String> = match rule.get("scope") {
            None => continue,
            Some(Value::String(s)) => vec![s.clone()],
            Some(Value::Array(scopes)) => scopes
                .iter()
                .map(|s| {
                    s.as_str()
                        .map(str::to_string)
                        .ok_or("scope array must contain strings")
                })
                .collect::<Result<_, _>>()?,
            _ => return Err("invalid scope selector".into()),
        };
        let mut match_score = None;
        for selector in selectors.iter().flat_map(|s| s.split(',')).map(str::trim) {
            if selector.is_empty() {
                continue;
            }
            if !selector.split_whitespace().all(valid_scope) {
                result.unsupported_selectors.push(selector.into());
                continue;
            }
            if let Some(candidate) = score(selector, stack) {
                match_score = Some(match_score.map_or(candidate, |old: usize| old.max(candidate)));
            }
        }
        if let Some(score) = match_score {
            for p in Property::ALL {
                if patch.has_value(p) || patch.states.contains_key(&p) {
                    let rank = (score, index);
                    if ranks.get(&p).is_none_or(|previous| rank >= *previous) {
                        result.style.copy_property_from(&patch, p);
                        result.property_rules.insert(p, index);
                        ranks.insert(p, rank);
                    }
                }
            }
        }
    }
    if !result.unsupported_selectors.is_empty() {
        result
            .limitations
            .push("unsupported selectors may override probe values; not verified".into());
    }
    result.style.validate()?;
    Ok(result)
}
