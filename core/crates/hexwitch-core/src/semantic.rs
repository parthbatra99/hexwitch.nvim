//! Semantic token rules, independent of editor transport.
//!
//! Matching uses the VS Code 1.95 selector score: exact type +100, each
//! modifier +100, language +10; ties use later source order per property.
//! Custom extension type hierarchies and fallback scope maps are not inferred.
use crate::color::Srgb;
use crate::ir::{
    Origin, Property, PropertyState, Role, SemanticStyle, SourceInfo, Style, SyntaxRole, Theme,
    UnderlineStyle,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const STANDARD_TYPES: &[&str] = &[
    "namespace",
    "type",
    "class",
    "enum",
    "interface",
    "struct",
    "typeParameter",
    "parameter",
    "variable",
    "property",
    "enumMember",
    "event",
    "function",
    "method",
    "macro",
    "keyword",
    "modifier",
    "comment",
    "string",
    "number",
    "regexp",
    "operator",
    "decorator",
    "label",
];

pub fn valid_id(id: &str) -> bool {
    id.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        && id.len() <= 128
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSelector {
    pub token_type: Option<String>,
    pub modifiers: BTreeSet<String>,
    pub language: Option<String>,
}
impl SemanticSelector {
    pub fn parse(input: &str) -> Result<Self, String> {
        let (head, language) = match input.split_once(':') {
            Some((head, language)) if valid_id(language) => (head, Some(language.to_string())),
            Some(_) => return Err(format!("invalid semantic selector {input}")),
            None => (input, None),
        };
        let mut parts = head.split('.');
        let token_type = match parts.next().unwrap_or("") {
            "*" => None,
            id if valid_id(id) => Some(id.to_string()),
            _ => return Err(format!("invalid semantic selector {input}")),
        };
        let mut modifiers = BTreeSet::new();
        for part in parts {
            if !valid_id(part) || !modifiers.insert(part.to_string()) {
                return Err(format!("invalid or repeated semantic modifier in {input}"));
            }
        }
        if modifiers.len() > 32 {
            return Err("semantic modifier count limit".into());
        }
        Ok(Self {
            token_type,
            modifiers,
            language,
        })
    }
}

pub fn portable_role(token_type: &str) -> Option<Role> {
    let syntax = match token_type {
        "namespace" => SyntaxRole::Namespace,
        "type" | "class" | "enum" | "interface" | "struct" | "typeParameter" => SyntaxRole::Type,
        "parameter" => SyntaxRole::Parameter,
        "variable" => SyntaxRole::Variable,
        "property" => SyntaxRole::Property,
        "enumMember" => SyntaxRole::Constant,
        "function" | "method" => SyntaxRole::Function,
        "macro" => SyntaxRole::Macro,
        "keyword" | "modifier" => SyntaxRole::Keyword,
        "comment" => SyntaxRole::Comment,
        "string" => SyntaxRole::String,
        "number" => SyntaxRole::Number,
        "regexp" => SyntaxRole::Regex,
        "operator" => SyntaxRole::Operator,
        "decorator" => SyntaxRole::Attribute,
        "event" | "label" => return Some(Role::Other(format!("semantic-{token_type}"))),
        _ => return None,
    };
    Some(Role::Syntax(syntax))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticContext {
    pub token_type: String,
    #[serde(default)]
    pub modifiers: BTreeSet<String>,
    #[serde(default)]
    pub language: Option<String>,
}
impl SemanticContext {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_id(&self.token_type)
            || self.modifiers.len() > 32
            || self.modifiers.iter().any(|m| !valid_id(m))
            || self.language.as_ref().is_some_and(|l| !valid_id(l))
        {
            return Err("invalid semantic token context".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticResolution {
    pub style: Style,
    pub matched_rules: Vec<usize>,
    pub property_rules: BTreeMap<Property, usize>,
    pub limitations: Vec<String>,
}

/// Resolve rules over an explicitly supplied lexical/base style. No grammar or
/// provider is fabricated when semantic highlighting enablement is unknown.
pub fn resolve_semantic(
    theme: &Theme,
    role: &Role,
    context: &SemanticContext,
    base: &Style,
) -> Result<SemanticResolution, String> {
    theme.validate()?;
    context.validate()?;
    if portable_role(&context.token_type).is_some_and(|expected| expected != *role) {
        return Err("semantic context does not match foreground role".into());
    }
    base.validate()?;
    if !theme.styles.contains_key(role) {
        return Err(format!("missing semantic base role {role}"));
    }
    let enabled = theme
        .extensions
        .get("vscode.semanticHighlighting")
        .and_then(Value::as_bool);
    let mut result = SemanticResolution {
        style: base.clone(),
        matched_rules: vec![],
        property_rules: BTreeMap::new(),
        limitations: vec![],
    };
    if enabled == Some(false) {
        return Ok(result);
    }
    if theme.extensions.contains_key("vscode.source") && theme.modifiers.is_empty() {
        result.limitations.push(
            "semantic default scope map not evaluated; supplied lexical base is provisional".into(),
        );
    }
    if enabled.is_none()
        && (theme.extensions.contains_key("vscode.source") || !theme.modifiers.is_empty())
    {
        result
            .limitations
            .push("semantic highlighting enablement depends on editor/language settings".into());
    }
    if portable_role(&context.token_type).is_none()
        && theme.extensions.contains_key("vscode.source")
    {
        result
            .limitations
            .push("custom semantic token type hierarchy is unavailable".into());
    }
    let mut scores = BTreeMap::new();
    for (index, rule) in theme.modifiers.iter().enumerate() {
        if rule.role != *role
            || !rule.modifiers.is_subset(&context.modifiers)
            || rule
                .language
                .as_ref()
                .is_some_and(|l| Some(l) != context.language.as_ref())
            || rule
                .token_type
                .as_ref()
                .is_some_and(|t| t != &context.token_type)
        {
            continue;
        }
        let score = rule.modifiers.len() * 100
            + usize::from(rule.token_type.is_some()) * 100
            + usize::from(rule.language.is_some()) * 10;
        result.matched_rules.push(index);
        for property in Property::ALL {
            if !rule.style.has_value(property) && !rule.style.states.contains_key(&property) {
                continue;
            }
            let rank = (score, rule.priority, index);
            if scores
                .get(&property)
                .is_none_or(|previous| rank >= *previous)
            {
                result.style.copy_property_from(&rule.style, property);
                result.property_rules.insert(property, index);
                scores.insert(property, rank);
            }
        }
    }
    if theme.extensions.contains_key("vscode.source")
        && result.matched_rules.is_empty()
        && !theme.modifiers.is_empty()
    {
        result.limitations.push(
            "no explicit semantic rule matched; extension/default scope fallback is unverified"
                .into(),
        );
    }
    result.style.validate()?;
    Ok(result)
}

/// Validated hex values shared by source importers (3/4/6/8 hexadecimal digits).
pub(crate) fn parse_color(value: &str) -> Result<(Srgb, Option<f64>), String> {
    let hex = value
        .strip_prefix('#')
        .ok_or_else(|| "expected #hex color".to_string())?;
    if !hex.is_ascii() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid hex color".into());
    }
    let expanded = match hex.len() {
        3 | 4 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 | 8 => hex.to_string(),
        _ => return Err("expected 3, 4, 6 or 8 hex digits".into()),
    };
    let rgb = Srgb::from_hex(&expanded[..6]).map_err(|e| e.to_string())?;
    let alpha = (expanded.len() == 8)
        .then(|| u8::from_str_radix(&expanded[6..], 16).unwrap() as f64 / 255.0);
    Ok((rgb, alpha))
}

pub(crate) fn font_style(value: &str) -> Result<Style, String> {
    let mut style = Style::new();
    style.bold = Some(false);
    style.italic = Some(false);
    style.strikethrough = Some(false);
    style
        .states
        .insert(Property::Underline, PropertyState::Clear);
    for token in value.split_whitespace() {
        match token {
            "bold" => style.bold = Some(true),
            "italic" => style.italic = Some(true),
            "underline" => {
                style.underline = Some(UnderlineStyle::Single);
                style.states.remove(&Property::Underline);
            }
            "strikethrough" => style.strikethrough = Some(true),
            _ => return Err(format!("invalid fontStyle token {token}")),
        }
    }
    Ok(style)
}

fn parse_settings(value: &Value, selector: &str, losses: &mut Vec<Value>) -> Result<Style, String> {
    let mut style = Style::new();
    let object = match value {
        Value::String(color) => {
            let (fg, alpha) = parse_color(color)?;
            style.fg = Some(fg);
            style.fg_opacity = Some(alpha.unwrap_or(1.0));
            return Ok(style);
        }
        Value::Object(object) => object,
        _ => return Err(format!("semantic rule {selector} must be color or object")),
    };
    for (key, value) in object {
        match key.as_str() {
            "foreground" => {let (fg,alpha) = parse_color(value.as_str().ok_or("semantic foreground must be a string")?)?; style.fg = Some(fg); style.fg_opacity = Some(alpha.unwrap_or(1.0));},
            "bold" | "italic" | "underline" | "strikethrough" => {
                let enabled = value.as_bool().ok_or_else(||format!("semantic {key} must be boolean"))?;
                match key.as_str() {
                    "bold" => style.bold = Some(enabled), "italic" => style.italic = Some(enabled),
                    "strikethrough" => style.strikethrough = Some(enabled),
                    _ if enabled => {style.underline = Some(UnderlineStyle::Single); style.states.remove(&Property::Underline);},
                    _ => {style.underline = None; style.states.insert(Property::Underline,PropertyState::Clear);},
                }
            },
            "fontStyle" => { if !value.is_string() {return Err("semantic fontStyle must be a string".into());} },
            _ => losses.push(serde_json::json!({"property":format!("semanticTokenColors.{selector}.{key}"),"reason":"unsupported semantic property; retained in source"})),
        }
    }
    // VS Code fontStyle overrides individual boolean settings regardless of key order.
    if let Some(value) = object.get("fontStyle") {
        let typography = font_style(value.as_str().unwrap())?;
        for p in [
            Property::Bold,
            Property::Italic,
            Property::Underline,
            Property::Strikethrough,
        ] {
            style.copy_property_from(&typography, p);
        }
    }
    style.validate()?;
    Ok(style)
}

pub(crate) fn import_semantic(
    theme: &mut Theme,
    root: &Value,
    losses: &mut Vec<Value>,
) -> Result<(), String> {
    if let Some(enablement) = root.get("semanticHighlighting") {
        if !enablement.is_boolean() {
            return Err("semanticHighlighting must be boolean".into());
        }
        theme
            .extensions
            .insert("vscode.semanticHighlighting".into(), enablement.clone());
    }
    let Some(rules) = root.get("semanticTokenColors").and_then(Value::as_object) else {
        return Ok(());
    };
    if rules.len() > 1024 {
        return Err("semantic rule count limit".into());
    }
    let offset = theme
        .modifiers
        .iter()
        .map(|r| r.priority)
        .max()
        .map_or(0, |p| p.saturating_add(1));
    for (priority, (selector, value)) in rules.iter().enumerate() {
        let parsed = SemanticSelector::parse(selector)?;
        let mut style = parse_settings(value, selector, losses)?;
        style.source = Some(SourceInfo {
            origin: Origin::Imported,
            reference: Some(format!("semanticTokenColors.{selector}")),
        });
        let types: Vec<_> = parsed
            .token_type
            .as_deref()
            .map_or_else(|| STANDARD_TYPES.to_vec(), |t| vec![t]);
        let mut emitted_roles = BTreeSet::new();
        for token_type in types {
            let Some(role) = portable_role(token_type) else {
                losses.push(serde_json::json!({"property":format!("semanticTokenColors.{selector}"),"reason":"custom semantic token type needs extension hierarchy; retained in source"}));
                continue;
            };
            if !emitted_roles.insert(role.clone()) {
                continue;
            }
            theme.styles.entry(role.clone()).or_default();
            theme.modifiers.push(SemanticStyle {
                role,
                modifiers: parsed.modifiers.clone(),
                language: parsed.language.clone(),
                style: style.clone(),
                token_type: parsed.token_type.clone(),
                priority: offset + priority as u32,
            });
            if theme.modifiers.len() > 4096 {
                return Err("expanded semantic rule count limit".into());
            }
        }
    }
    if !rules.is_empty() {
        losses.push(serde_json::json!({"property":"semanticTokenColors","reason":"standard explicit selectors resolved; extension type hierarchy and default scope mappings require editor context"}));
        if root
            .get("semanticHighlighting")
            .and_then(Value::as_bool)
            .is_none()
        {
            losses.push(serde_json::json!({"property":"semanticHighlighting","reason":"semantic enablement unknown; requires editor or language setting"}));
        }
    }
    Ok(())
}
