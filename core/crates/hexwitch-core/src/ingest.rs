//! Ingest: turn reference themes into IR.
//!
//! First (and most useful) source format: VS Code theme JSON — it is plain
//! data, widely available for nearly every famous theme, and exercises both
//! halves of what a theme *is*: workbench `colors` (UI surfaces/states) and
//! `tokenColors` (TextMate scopes → syntax styles).
//!
//! Deliberate limits: selected workbench mappings and a restricted dotted
//! selector/ancestor grammar, not full TextMate. Source rules and opacity are
//! preserved; unsupported semantics are recorded in import losses.
//! `semanticTokenColors` is retained but not yet mapped to portable modifiers.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::color::Srgb;
use crate::ir::role::{Role, Surface, TextRole};
use crate::ir::style::{Style, UnderlineStyle};
use crate::ir::theme::{Appearance, Theme};
use crate::IR_VERSION;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid color for `{key}`: {value}")]
    BadColor { key: String, value: String },
    #[error("invalid source: {0}")]
    InvalidSource(String),
}

/// VS Code workbench key -> (role, slot) mapping lives in
/// [`workbench_role`]; kept as a function so the table stays private and
/// testable.

#[derive(Debug, Deserialize)]
struct TokenColorEntry {
    #[serde(default)]
    scope: ScopeSpec,
    settings: TokenSettings,
}

#[derive(Debug, Deserialize, Default)]
#[serde(untagged)]
enum ScopeSpec {
    #[default]
    None,
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize, Default, Clone)]
struct TokenSettings {
    #[serde(default)]
    foreground: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default, rename = "fontStyle")]
    font_style: Option<String>,
}

fn parse_hex_maybe_alpha(s: &str) -> Result<Srgb, String> {
    let h = s.strip_prefix('#').unwrap_or(s);
    if h.len() == 8 {
        if !h.is_ascii() || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid hex color".into());
        }
        Srgb::from_hex(&h[..6]).map_err(|e| e.to_string())
    } else {
        Srgb::from_hex(&format!("#{h}")).map_err(|e| e.to_string())
    }
}

/// Ordered selector -> role table lives in [`scope_rules`]. First rule
/// whose selector is `scope` or a dot-boundary prefix of it wins; specific
/// selectors precede general ones ("keyword.operator" before "keyword").
fn selector_matches(selector: &str, scope: &str) -> bool {
    scope == selector
        || scope
            .strip_prefix(selector)
            .is_some_and(|r| r.starts_with('.'))
}

fn simple_selector(selector: &str) -> bool {
    selector.is_empty()
        || selector.split_whitespace().all(|part| {
            !part.is_empty()
                && part.split('.').all(|segment| {
                    !segment.is_empty()
                        && segment
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                })
        })
}
fn stack_matches(selector: &str, stack: &str) -> bool {
    if selector.is_empty() {
        return true;
    }
    let scopes: Vec<_> = stack.split_whitespace().collect();
    let parts: Vec<_> = selector.split_whitespace().collect();
    let mut index = scopes.len();
    for part in parts.iter().rev() {
        let Some(found) = (0..index)
            .rev()
            .find(|i| selector_matches(part, scopes[*i]))
        else {
            return false;
        };
        index = found;
    }
    true
}

fn depth(s: &str) -> usize {
    s.split('.').count()
}

fn ingest_token_colors(
    theme: &mut Theme,
    token_colors: &[TokenColorEntry],
) -> Result<(), IngestError> {
    // Resolve a deliberately limited selector grammar: dotted scopes, comma
    // alternatives and ordered ancestor stacks. This is not full TextMate.
    let mut rules: Vec<(String, &TokenSettings, usize)> = vec![];
    let mut representatives: BTreeMap<Role, (String, usize)> = BTreeMap::new();
    for (order, entry) in token_colors.iter().enumerate() {
        let scopes = match &entry.scope {
            ScopeSpec::None => vec![String::new()],
            ScopeSpec::One(s) => vec![s.clone()],
            ScopeSpec::Many(v) => v.clone(),
        };
        for scope in scopes {
            for selector in scope.split(',').map(str::trim) {
                if !simple_selector(selector) {
                    continue;
                }
                rules.push((selector.to_string(), &entry.settings, order));
                let leaf = selector.split_whitespace().last().unwrap_or("");
                let best = scope_rules()
                    .into_iter()
                    .filter(|(sel, _)| selector_matches(sel, leaf))
                    .max_by_key(|(sel, _)| depth(sel));
                if let Some((_, role)) = best {
                    let replace =
                        representatives
                            .get(&role)
                            .is_none_or(|(previous, previous_order)| {
                                (depth(leaf), order)
                                    >= (
                                        depth(previous.split_whitespace().last().unwrap_or("")),
                                        *previous_order,
                                    )
                            });
                    if replace {
                        representatives.insert(role, (selector.to_string(), order));
                    }
                }
            }
        }
    }
    for (role, (scope, _)) in representatives {
        let mut applicable: Vec<_> = rules
            .iter()
            .filter(|(selector, _, _)| stack_matches(selector, &scope))
            .collect();
        applicable.sort_by_key(|(selector, _, order)| {
            (selector.split_whitespace().count(), depth(selector), *order)
        });
        let mut settings = TokenSettings::default();
        for (_, rule, _) in applicable {
            if rule.foreground.is_some() {
                settings.foreground = rule.foreground.clone();
            }
            if rule.background.is_some() {
                settings.background = rule.background.clone();
            }
            if rule.font_style.is_some() {
                settings.font_style = rule.font_style.clone();
            }
        }
        let mut style = theme.get(&role).cloned().unwrap_or_default();
        if let Some(fg) = &settings.foreground {
            style.fg = Some(
                parse_hex_maybe_alpha(fg).map_err(|_| IngestError::BadColor {
                    key: scope.clone(),
                    value: fg.clone(),
                })?,
            );
        }
        if let Some(bg) = &settings.background {
            style.bg = Some(
                parse_hex_maybe_alpha(bg).map_err(|_| IngestError::BadColor {
                    key: scope.clone(),
                    value: bg.clone(),
                })?,
            );
        }
        let alpha = |value: &str| {
            value
                .strip_prefix('#')
                .filter(|h| h.len() == 8 && h.is_ascii())
                .and_then(|h| u8::from_str_radix(&h[6..], 16).ok())
                .map(|a| a as f64 / 255.0)
        };
        style.fg_opacity = settings.foreground.as_deref().and_then(alpha);
        style.bg_opacity = settings.background.as_deref().and_then(alpha);
        style.source = Some(crate::ir::SourceInfo {
            origin: crate::ir::Origin::Imported,
            reference: Some(scope.clone()),
        });
        apply_font_style(&mut style, settings.font_style.as_deref());
        if style.fg.is_some()
            || style.bg.is_some()
            || style.bold.is_some()
            || style.italic.is_some()
            || style.underline.is_some()
        {
            theme.set(role, style);
        }
    }
    Ok(())
}

fn apply_font_style(style: &mut Style, font_style: Option<&str>) {
    let Some(fs) = font_style else { return };
    // A specified fontStyle replaces the typography family, including empty resets.
    style.bold = Some(false);
    style.italic = Some(false);
    style.underline = None;
    style.states.insert(
        crate::ir::Property::Underline,
        crate::ir::PropertyState::Clear,
    );
    for token in fs.split_whitespace() {
        match token {
            "bold" => style.bold = Some(true),
            "italic" => style.italic = Some(true),
            "underline" => style.underline = Some(UnderlineStyle::Single),
            "strikethrough" => style.underline = Some(UnderlineStyle::Strikethrough),
            _ => {}
        }
    }
    if style.underline.is_some() {
        style.states.remove(&crate::ir::Property::Underline);
    }
}

fn scope_rules() -> Vec<(&'static str, Role)> {
    use crate::ir::role::SyntaxRole;
    let syntax: Vec<(&str, Role)> = vec![
        ("constant.numeric", Role::Syntax(SyntaxRole::Number)),
        ("constant.language", Role::Syntax(SyntaxRole::Boolean)),
        (
            "constant.character.escape",
            Role::Syntax(SyntaxRole::Escape),
        ),
        ("constant", Role::Syntax(SyntaxRole::Constant)),
        ("string", Role::Syntax(SyntaxRole::String)),
        ("keyword.operator", Role::Syntax(SyntaxRole::Operator)),
        ("keyword", Role::Syntax(SyntaxRole::Keyword)),
        ("storage", Role::Syntax(SyntaxRole::Keyword)),
        ("entity.name.function", Role::Syntax(SyntaxRole::Function)),
        ("support.function", Role::Syntax(SyntaxRole::Function)),
        ("entity.name.type", Role::Syntax(SyntaxRole::Type)),
        ("entity.name.class", Role::Syntax(SyntaxRole::Type)),
        ("support.type", Role::Syntax(SyntaxRole::Type)),
        ("entity.name.tag", Role::Syntax(SyntaxRole::Tag)),
        (
            "entity.other.attribute-name",
            Role::Syntax(SyntaxRole::Attribute),
        ),
        (
            "variable.language",
            Role::Syntax(SyntaxRole::VariableBuiltin),
        ),
        ("variable.parameter", Role::Syntax(SyntaxRole::Parameter)),
        (
            "variable.other.property",
            Role::Syntax(SyntaxRole::Property),
        ),
        (
            "support.variable.property",
            Role::Syntax(SyntaxRole::Property),
        ),
        ("variable", Role::Syntax(SyntaxRole::Variable)),
        ("comment", Role::Syntax(SyntaxRole::Comment)),
        ("punctuation", Role::Syntax(SyntaxRole::Punctuation)),
    ];
    syntax
}

fn workbench_role(key: &str) -> Option<(Role, bool)> {
    use crate::ir::role::{DiagnosticKind, DiagnosticPart, DiffKind, DiffPart, InteractionRole};
    // (key, role, is_background_slot)
    let table: &[(&str, Role, bool)] = &[
        ("editor.background", Role::Surface(Surface::Base), true),
        ("editor.foreground", Role::Text(TextRole::Primary), false),
        ("sideBar.background", Role::Surface(Surface::Panel), true),
        (
            "editorWidget.background",
            Role::Surface(Surface::Float),
            true,
        ),
        ("editorWidget.border", Role::Surface(Surface::Border), false),
        (
            "editorLineNumber.foreground",
            Role::Surface(Surface::Gutter),
            false,
        ),
        (
            "editor.lineHighlightBackground",
            Role::Interaction(InteractionRole::CursorLine),
            true,
        ),
        (
            "editor.selectionBackground",
            Role::Interaction(InteractionRole::Selection),
            true,
        ),
        (
            "editorError.foreground",
            Role::Diagnostic {
                kind: DiagnosticKind::Error,
                part: DiagnosticPart::Text,
            },
            false,
        ),
        (
            "editorWarning.foreground",
            Role::Diagnostic {
                kind: DiagnosticKind::Warning,
                part: DiagnosticPart::Text,
            },
            false,
        ),
        (
            "editorInfo.foreground",
            Role::Diagnostic {
                kind: DiagnosticKind::Info,
                part: DiagnosticPart::Text,
            },
            false,
        ),
        (
            "editorGutter.addedBackground",
            Role::Diff {
                kind: DiffKind::Added,
                part: DiffPart::Gutter,
            },
            false,
        ),
        (
            "editorGutter.modifiedBackground",
            Role::Diff {
                kind: DiffKind::Changed,
                part: DiffPart::Gutter,
            },
            false,
        ),
    ];
    table
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, role, is_bg)| (role.clone(), *is_bg))
}

/// Load a theme from JSON, accepting either format:
/// * engine IR (has `meta` + `styles`)
/// * VS Code theme JSON (has `colors` and/or `tokenColors`)
///
/// This is what the CLI (and later the Neovim client) feed to analyze,
/// revise, and emit, so revisions round-trip through IR files.
pub fn load_theme(json: &str) -> Result<Theme, IngestError> {
    let v = crate::jsonc::parse(json).map_err(IngestError::InvalidSource)?;
    let is_ir = v.get("meta").is_some() && v.get("styles").is_some();
    if is_ir {
        Theme::from_json_str(&v.to_string()).map_err(IngestError::Json)
    } else {
        ingest_vscode_json(json)
    }
}

/// Ingest a VS Code theme JSON document into the IR.
pub fn ingest_vscode_json(json: &str) -> Result<Theme, IngestError> {
    let root = crate::jsonc::parse(json).map_err(IngestError::InvalidSource)?;
    if !root.is_object() {
        return Err(IngestError::InvalidSource(
            "theme root must be an object".into(),
        ));
    }
    if root.get("include").is_some() {
        return Err(IngestError::InvalidSource(
            "includes require load_theme_path for sandboxed resolution".into(),
        ));
    }
    if !["colors", "tokenColors", "semanticTokenColors"]
        .iter()
        .any(|key| root.get(key).is_some())
    {
        return Err(IngestError::InvalidSource(
            "expected colors, tokenColors or semanticTokenColors".into(),
        ));
    }
    if root.get("colors").is_some_and(|v| !v.is_object())
        || root.get("tokenColors").is_some_and(|v| !v.is_array())
        || root
            .get("semanticTokenColors")
            .is_some_and(|v| !v.is_object())
    {
        return Err(IngestError::InvalidSource(
            "invalid theme field type".into(),
        ));
    }
    let name = root
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Unnamed")
        .to_string();
    let appearance = match root.get("type").and_then(Value::as_str) {
        Some("light") => Appearance::Light,
        _ => Appearance::Dark,
    };

    let mut theme = Theme::new(name, appearance);
    theme.meta.source = Some("vscode json".to_string());
    theme.meta.ir_version = IR_VERSION;

    if let Some(colors) = root.get("colors").and_then(Value::as_object) {
        for (key, value) in colors {
            let hex = value.as_str().ok_or_else(|| {
                IngestError::InvalidSource(format!("workbench color {key} must be a string"))
            })?;
            let Some((role, is_bg)) = workbench_role(key) else {
                continue;
            };
            let color = parse_hex_maybe_alpha(hex).map_err(|_| IngestError::BadColor {
                key: key.clone(),
                value: hex.to_string(),
            })?;
            let style = theme.styles.entry(role).or_default();
            let opacity = hex
                .strip_prefix('#')
                .filter(|h| h.len() == 8 && h.is_ascii())
                .and_then(|h| u8::from_str_radix(&h[6..], 16).ok())
                .map(|a| a as f64 / 255.0);
            if is_bg {
                style.bg = Some(color);
                style.bg_opacity = opacity;
            } else {
                style.fg = Some(color);
                style.fg_opacity = opacity;
            }
            style.source = Some(crate::ir::SourceInfo {
                origin: crate::ir::Origin::Imported,
                reference: Some(key.clone()),
            });
        }
    }

    let token_colors: Vec<TokenColorEntry> = root
        .get("tokenColors")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|v| serde_json::from_value(v.clone()))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    ingest_token_colors(&mut theme, &token_colors)?;
    // Preserve the entire source, including unsupported rules; do not claim portability.
    let mut losses = vec![];
    if root.get("semanticTokenColors").is_some() {
        losses.push(serde_json::json!({"property":"semanticTokenColors","reason":"preserved but not yet mapped into portable modifiers"}));
    }
    if let Some(colors) = root.get("colors").and_then(Value::as_object) {
        for key in colors.keys().filter(|key| workbench_role(key).is_none()) {
            losses.push(serde_json::json!({"property":key,"reason":"unmapped workbench key; preserved in source extension"}));
        }
    }
    for entry in &token_colors {
        let scopes = match &entry.scope {
            ScopeSpec::None => vec![],
            ScopeSpec::One(s) => vec![s.clone()],
            ScopeSpec::Many(s) => s.clone(),
        };
        for scope in scopes {
            for selector in scope.split(',').map(str::trim) {
                let leaf = selector.split_whitespace().last().unwrap_or("");
                if !simple_selector(selector)
                    || !scope_rules()
                        .iter()
                        .any(|(sel, _)| selector_matches(sel, leaf))
                {
                    losses.push(serde_json::json!({"property":selector,"reason":"unsupported or unmapped selector; preserved in source extension"}));
                }
            }
        }
    }
    theme.extensions.insert(
        "hexwitch.import.losses.v1".into(),
        serde_json::json!(losses),
    );
    theme.extensions.insert("vscode.source".into(), root);
    theme.validate().map_err(IngestError::InvalidSource)?;
    Ok(theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::role::{SyntaxRole, TextRole};

    #[test]
    fn minimal_theme_maps_workbench_colors() {
        let json = r##"{
            "name": "Tiny", "type": "dark",
            "colors": {
                "editor.background": "#282828",
                "editor.foreground": "#ebdbb2",
                "editorError.foreground": "#fb4934"
            }
        }"##;
        let t = ingest_vscode_json(json).unwrap();
        assert_eq!(t.meta.name, "Tiny");
        assert_eq!(t.meta.appearance, Appearance::Dark);
        assert_eq!(
            t.bg(&Role::Surface(Surface::Base)),
            Some(Srgb::from_hex("#282828").unwrap())
        );
        assert_eq!(
            t.fg(&Role::Text(TextRole::Primary)),
            Some(Srgb::from_hex("#ebdbb2").unwrap())
        );
        assert_eq!(
            t.fg(&Role::Diagnostic {
                kind: crate::ir::role::DiagnosticKind::Error,
                part: crate::ir::role::DiagnosticPart::Text
            }),
            Some(Srgb::from_hex("#fb4934").unwrap())
        );
    }

    #[test]
    fn token_scopes_map_with_prefix_semantics() {
        let json = r##"{
            "name": "Scopes", "type": "dark",
            "colors": { "editor.background": "#000000" },
            "tokenColors": [
                { "scope": "comment.line.double-dash.lua", "settings": { "foreground": "#928374", "fontStyle": "italic" } },
                { "scope": ["keyword.control.import", "keyword.control.from"], "settings": { "foreground": "#fb4934" } },
                { "scope": "keyword.operator.logical", "settings": { "foreground": "#83a598" } },
                { "scope": "entity.name.function", "settings": { "foreground": "#b8bb26", "fontStyle": "bold" } }
            ]
        }"##;
        let t = ingest_vscode_json(json).unwrap();
        // "comment.line..." matched selector "comment" as dot-prefix.
        let comment = t.get(&Role::Syntax(SyntaxRole::Comment)).unwrap();
        assert_eq!(comment.fg, Some(Srgb::from_hex("#928374").unwrap()));
        assert_eq!(comment.italic, Some(true));
        // Keywords and the more specific operator rule both resolved.
        assert_eq!(
            t.fg(&Role::Syntax(SyntaxRole::Keyword)),
            Some(Srgb::from_hex("#fb4934").unwrap())
        );
        assert_eq!(
            t.fg(&Role::Syntax(SyntaxRole::Operator)),
            Some(Srgb::from_hex("#83a598").unwrap())
        );
        let func = t.get(&Role::Syntax(SyntaxRole::Function)).unwrap();
        assert_eq!(func.fg, Some(Srgb::from_hex("#b8bb26").unwrap()));
        assert_eq!(func.bold, Some(true));
    }

    #[test]
    fn specific_selectors_beat_general_ones() {
        // variable.parameter must win over variable for the parameter role,
        // and must not clobber the plain variable role.
        let json = r##"{
            "name": "Prec", "type": "dark",
            "tokenColors": [
                { "scope": "variable", "settings": { "foreground": "#111111" } },
                { "scope": "variable.parameter", "settings": { "foreground": "#222222" } }
            ]
        }"##;
        let t = ingest_vscode_json(json).unwrap();
        assert_eq!(
            t.fg(&Role::Syntax(SyntaxRole::Variable)),
            Some(Srgb::from_hex("#111111").unwrap())
        );
        assert_eq!(
            t.fg(&Role::Syntax(SyntaxRole::Parameter)),
            Some(Srgb::from_hex("#222222").unwrap())
        );
    }

    #[test]
    fn alpha_rgb_channel_and_appearance_are_preserved() {
        let json = r##"{
            "name": "A", "type": "light",
            "colors": { "editor.background": "#ffffff", "editor.selectionBackground": "#515c7e4d" }
        }"##;
        let t = ingest_vscode_json(json).unwrap();
        assert_eq!(t.meta.appearance, Appearance::Light);
        assert_eq!(
            t.bg(&Role::Interaction(
                crate::ir::role::InteractionRole::Selection
            )),
            Some(Srgb::from_hex("#515c7e").unwrap())
        );
    }

    #[test]
    fn unknown_keys_and_scopes_are_ignored() {
        let json = r##"{
            "name": "U", "type": "dark",
            "colors": { "editorCursor.foreground": "#abcdef", "editor.background": "#000000" },
            "tokenColors": [
                { "scope": "meta.weird.future-scope", "settings": { "foreground": "#123456" } }
            ]
        }"##;
        let t = ingest_vscode_json(json).unwrap();
        assert_eq!(t.styles.len(), 1); // only surface.base
    }

    #[test]
    fn bad_color_fails_loudly() {
        let json = r#"{ "name": "B", "colors": { "editor.background": "not-a-color" } }"#;
        let err = ingest_vscode_json(json).unwrap_err();
        assert!(err.to_string().contains("not-a-color"));
    }
}
