//! Ingest: turn reference themes into IR.
//!
//! First (and most useful) source format: VS Code theme JSON — it is plain
//! data, widely available for nearly every famous theme, and exercises both
//! halves of what a theme *is*: workbench `colors` (UI surfaces/states) and
//! `tokenColors` (TextMate scopes → syntax styles).
//!
//! Deliberate v0 limits, documented rather than hidden:
//! * Only a curated, unambiguous subset of workbench keys and TextMate
//!   scopes is mapped. Unmapped entries are ignored, and adapters/reporting
//!   will surface coverage later (M1 continues this work).
//! * Colors with alpha (`#rrggbbaa`) drop the alpha channel here; compositing
//!   semantics belong to scene validation, which models them explicitly.
//! * `semanticTokenColors` is not mapped yet; it arrives with the VS Code
//!   adapter's lowering tables.

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
        // Keep the RGB part; alpha handling is scene validation's job.
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

fn depth(s: &str) -> usize {
    s.split('.').count()
}

fn ingest_token_colors(
    theme: &mut Theme,
    token_colors: &[TokenColorEntry],
) -> Result<(), IngestError> {
    // scope -> (settings, order of appearance)
    let mut rules: BTreeMap<String, (TokenSettings, usize)> = BTreeMap::new();
    for (order, entry) in token_colors.iter().enumerate() {
        let scopes = match &entry.scope {
            ScopeSpec::None => continue,
            ScopeSpec::One(s) => vec![s.clone()],
            ScopeSpec::Many(v) => v.clone(),
        };
        for scope in scopes {
            rules
                .entry(scope)
                .and_modify(|(s, _)| *s = entry.settings.clone())
                .or_insert_with(|| (entry.settings.clone(), order));
        }
    }

    // TextMate-style resolution: for each scope in the theme, the *deepest*
    // selector from our table that matches it claims the role. This keeps
    // `variable.other.property` styling the property role (not being
    // stolen by the general `variable` selector) while still letting
    // `comment.line.double-dash.lua` claim the comment role. A role is
    // claimed once; iteration is over a BTreeMap, so duplicate sources
    // resolve deterministically.
    for (scope, (settings, _order)) in &rules {
        let best = scope_rules()
            .into_iter()
            .filter(|(sel, _)| selector_matches(sel, scope))
            .max_by_key(|(sel, _)| depth(sel));
        let Some((_, role)) = best else { continue };
        if theme.styles.contains_key(&role) {
            continue;
        }

        let mut style = Style::new();
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
        apply_font_style(&mut style, settings.font_style.as_deref());
        if style.fg.is_some() || style.bg.is_some() || style.underline.is_some() {
            theme.set(role, style);
        }
    }
    Ok(())
}

fn apply_font_style(style: &mut Style, font_style: Option<&str>) {
    let Some(fs) = font_style else { return };
    for token in fs.split_whitespace() {
        match token {
            "bold" => style.bold = Some(true),
            "italic" => style.italic = Some(true),
            "underline" => style.underline = Some(UnderlineStyle::Single),
            "strikethrough" => style.underline = Some(UnderlineStyle::Strikethrough),
            _ => {}
        }
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
        ("editorWidget.border", Role::Surface(Surface::Border), true),
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
            true,
        ),
        (
            "editorGutter.modifiedBackground",
            Role::Diff {
                kind: DiffKind::Changed,
                part: DiffPart::Gutter,
            },
            true,
        ),
    ];
    table
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, role, is_bg)| (role.clone(), *is_bg))
}

/// Ingest a VS Code theme JSON document into the IR.
pub fn ingest_vscode_json(json: &str) -> Result<Theme, IngestError> {
    let root: Value = serde_json::from_str(json)?;
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
            let Some(hex) = value.as_str() else { continue };
            let Some((role, is_bg)) = workbench_role(key) else {
                continue;
            };
            let color = parse_hex_maybe_alpha(hex).map_err(|_| IngestError::BadColor {
                key: key.clone(),
                value: hex.to_string(),
            })?;
            let style = theme.styles.entry(role).or_default();
            if is_bg {
                style.bg = Some(color);
            } else {
                style.fg = Some(color);
            }
        }
    }

    let token_colors: Vec<TokenColorEntry> = root
        .get("tokenColors")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    ingest_token_colors(&mut theme, &token_colors)?;
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
    fn alpha_colors_drop_to_rgb() {
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
