//! Neovim lowering for explicit semantic rules. Conjunctions of modifiers
//! cannot be represented by independent @lsp.typemod groups and are reported.
use crate::ir::{Property, PropertyState, Style, Theme, UnderlineStyle};
use crate::semantic::{portable_role, resolve_semantic, SemanticContext, STANDARD_TYPES};
use std::collections::BTreeSet;
use std::fmt::Write;

#[derive(Default)]
pub(super) struct SemanticEmit {
    pub lua: String,
    pub defined: Vec<String>,
    pub losses: Vec<(String, String, String)>,
}

fn attrs(
    style: &Style,
    base: crate::color::Srgb,
    group: &str,
    losses: &mut Vec<(String, String, String)>,
) -> Vec<String> {
    let mut attrs = vec![];
    for (name, color, alpha) in [
        ("fg", style.fg, style.fg_opacity),
        ("bg", style.bg, style.bg_opacity),
        ("sp", style.special, style.special_opacity),
    ] {
        if let Some(color) = color {
            let color = if let Some(alpha) = alpha.filter(|a| *a < 1.0) {
                losses.push((
                    group.into(),
                    name.into(),
                    "semantic alpha flattened against base; real syntax background may differ"
                        .into(),
                ));
                crate::color::composite_over(color, alpha, base)
            } else {
                color
            };
            attrs.push(format!("{name} = \"{}\"", color.to_hex()));
        }
    }
    for (name, value) in [
        ("bold", style.bold),
        ("italic", style.italic),
        ("strikethrough", style.strikethrough),
    ] {
        if let Some(value) = value {
            attrs.push(format!("{name} = {value}"));
        }
    }
    if let Some(underline) = style.underline {
        let name = match underline {
            UnderlineStyle::Single => "underline",
            UnderlineStyle::Double => "underdouble",
            UnderlineStyle::Curl => "undercurl",
            UnderlineStyle::Dot => "underdotted",
            UnderlineStyle::Dash => "underdashed",
            UnderlineStyle::Strikethrough => "strikethrough",
        };
        attrs.push(format!("{name} = true"));
    }
    for (property, state) in &style.states {
        if *state == PropertyState::Clear {
            match property {
                Property::Bold => attrs.push("bold = false".into()),
                Property::Italic => attrs.push("italic = false".into()),
                Property::Strikethrough => attrs.push("strikethrough = false".into()),
                Property::Underline => attrs.extend(
                    [
                        "underline",
                        "undercurl",
                        "underdouble",
                        "underdotted",
                        "underdashed",
                    ]
                    .into_iter()
                    .map(|name| format!("{name} = false")),
                ),
                _ => losses.push((
                    group.into(),
                    format!("{property:?}"),
                    "semantic clear needs editor defaults; omitted".into(),
                )),
            }
        } else {
            losses.push((
                group.into(),
                format!("{property:?}"),
                "semantic state cannot be resolved without editor stack; omitted".into(),
            ));
        }
    }
    attrs
}

pub(super) fn emit(theme: &Theme) -> Result<SemanticEmit, String> {
    let mut emitted = SemanticEmit::default();
    if theme.modifiers.is_empty()
        || theme
            .extensions
            .get("vscode.semanticHighlighting")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    {
        return Ok(emitted);
    }
    let mut contexts = BTreeSet::new();
    for rule in &theme.modifiers {
        if rule.modifiers.len() > 1 {
            emitted.losses.push((
                rule.role.to_string(),
                "modifiers".into(),
                "modifier conjunction is not representable by Neovim typemod groups; rule skipped"
                    .into(),
            ));
            continue;
        }
        if rule
            .token_type
            .as_ref()
            .is_some_and(|t| !STANDARD_TYPES.contains(&t.as_str()))
        {
            emitted.losses.push((
                rule.role.to_string(),
                "token_type".into(),
                "custom semantic type not supported by Neovim mapping; rule skipped".into(),
            ));
            continue;
        }
        for token_type in STANDARD_TYPES {
            if portable_role(token_type).as_ref() != Some(&rule.role)
                || rule.token_type.as_ref().is_some_and(|t| t != token_type)
            {
                continue;
            }
            contexts.insert((token_type.to_string(), None, rule.language.clone()));
            if let Some(modifier) = rule.modifiers.iter().next() {
                contexts.insert((
                    token_type.to_string(),
                    Some(modifier.clone()),
                    rule.language.clone(),
                ));
            }
        }
    }
    // Multi-modifier rules are deliberately excluded from target lowering.
    let mut supported = theme.clone();
    supported.modifiers.retain(|r| r.modifiers.len() <= 1);
    let base = theme.base_bg().unwrap_or(crate::color::Srgb::BLACK);
    for (token_type, modifier, language) in contexts {
        let role = portable_role(&token_type).unwrap();
        let context = SemanticContext {
            token_type: token_type.clone(),
            modifiers: modifier.iter().cloned().collect(),
            language: language.clone(),
        };
        let resolution = resolve_semantic(&supported, &role, &context, theme.get(&role).unwrap())?;
        let mut patch = Style::new();
        for property in resolution.property_rules.keys() {
            patch.copy_property_from(&resolution.style, *property);
        }
        let mut group = match modifier {
            Some(modifier) => format!("@lsp.typemod.{token_type}.{modifier}"),
            None => format!("@lsp.type.{token_type}"),
        };
        if let Some(language) = language {
            group.push('.');
            group.push_str(&language);
        }
        let opts = attrs(&patch, base, &group, &mut emitted.losses);
        if opts.is_empty() {
            continue;
        }
        writeln!(
            emitted.lua,
            "vim.api.nvim_set_hl(0, \"{group}\", {{ {} }})",
            opts.join(", ")
        )
        .unwrap();
        emitted.defined.push(group.clone());
        for limitation in resolution.limitations {
            emitted
                .losses
                .push((group.clone(), "enablement".into(), limitation));
        }
    }
    if !emitted.defined.is_empty() {
        emitted.losses.push(("semantic".into(),"priority".into(),"emitted native LSP groups; simultaneous modifier and Tree-sitter/plugin priority interactions remain unverified".into()));
    }
    Ok(emitted)
}
