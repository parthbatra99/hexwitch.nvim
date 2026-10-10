//! The theme: metadata plus a role -> style graph.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use crate::color::{contrast_ratio, Srgb};
use crate::ir::role::{Role, Surface, SyntaxRole, TextRole};
use crate::ir::style::Style;
use crate::IR_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeMeta {
    pub name: String,
    pub appearance: Appearance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Where this theme came from, e.g. `gruvbox (vscode json) @ abc1234`
    /// or `synthesized from brief v3`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// IR version this theme was built against. Older IR themes load into
    /// newer engines via explicit migration steps only.
    pub ir_version: u32,
}

/// A theme: one appearance mode. Light + dark variants sharing an identity
/// are two `Theme`s linked by a family concept (added with revise).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub meta: ThemeMeta,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<SemanticStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<Relationship>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
    /// Role -> treatment. `BTreeMap` for deterministic, diff-friendly JSON.
    pub styles: BTreeMap<Role, Style>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticStyle {
    pub role: Role,
    pub modifiers: std::collections::BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub style: Style,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub from: Role,
    pub to: Role,
    pub kind: RelationshipKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    Inherits,
    Surface,
    Association,
    LowerProminence,
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            meta: ThemeMeta,
            styles: BTreeMap<Role, Style>,
            #[serde(default)]
            modifiers: Vec<SemanticStyle>,
            #[serde(default)]
            relationships: Vec<Relationship>,
            #[serde(default)]
            extensions: BTreeMap<String, serde_json::Value>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let theme = Theme {
            meta: wire.meta,
            styles: wire.styles,
            modifiers: wire.modifiers,
            relationships: wire.relationships,
            extensions: wire.extensions,
        };
        theme.validate().map_err(serde::de::Error::custom)?;
        Ok(theme)
    }
}

/// Problems a theme can have before any scene validation runs.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ThemeError {
    #[error("theme `{0}` is missing required role `{1}`")]
    MissingRole(String, String),
    #[error("role `{0}` has neither a foreground nor a background")]
    EmptyStyle(String),
}

impl Theme {
    pub fn new(name: impl Into<String>, appearance: Appearance) -> Self {
        Theme {
            meta: ThemeMeta {
                name: name.into(),
                appearance,
                description: None,
                source: None,
                ir_version: IR_VERSION,
            },
            styles: BTreeMap::new(),
            modifiers: vec![],
            relationships: vec![],
            extensions: BTreeMap::new(),
        }
    }

    pub fn get(&self, role: &Role) -> Option<&Style> {
        self.styles.get(role)
    }

    pub fn set(&mut self, role: Role, style: Style) {
        self.styles.insert(role, style);
    }

    /// Foreground color for a role, if specified.
    pub fn fg(&self, role: &Role) -> Option<Srgb> {
        self.styles.get(role).and_then(|s| s.fg)
    }

    /// Background color for a role, if specified.
    pub fn bg(&self, role: &Role) -> Option<Srgb> {
        self.styles.get(role).and_then(|s| s.bg)
    }

    /// The editor base background (what most tokens sit on).
    pub fn base_bg(&self) -> Option<Srgb> {
        self.bg(&Role::Surface(Surface::Base))
    }

    /// Contrast of a role's foreground against a given background color —
    /// the primitive scene checks are built from.
    pub fn fg_contrast_against(&self, role: &Role, backdrop: Srgb) -> Option<f64> {
        self.fg(role).map(|fg| {
            contrast_ratio(
                crate::color::composite_over(
                    fg,
                    self.get(role).and_then(|s| s.fg_opacity).unwrap_or(1.0),
                    backdrop,
                ),
                backdrop,
            )
        })
    }

    /// Quick structural sanity: the roles every emitted theme must define
    /// for an adapter to have something to lower. Not a readability audit —
    /// that is scene validation's job.
    pub fn check_core_roles(&self) -> Result<(), Vec<ThemeError>> {
        let required: Vec<Role> = vec![
            Role::Surface(Surface::Base),
            Role::Surface(Surface::Panel),
            Role::Surface(Surface::Float),
            Role::Text(TextRole::Primary),
            Role::Text(TextRole::Secondary),
            Role::Syntax(SyntaxRole::Comment),
            Role::Syntax(SyntaxRole::Keyword),
            Role::Syntax(SyntaxRole::String),
            Role::Syntax(SyntaxRole::Function),
            Role::Syntax(SyntaxRole::Variable),
            Role::Syntax(SyntaxRole::Type),
        ];
        let mut errs = Vec::new();
        for role in required {
            match self.styles.get(&role) {
                None => errs.push(ThemeError::MissingRole(
                    self.meta.name.clone(),
                    role.to_string(),
                )),
                Some(style) if style.fg.is_none() && style.bg.is_none() => {
                    errs.push(ThemeError::EmptyStyle(role.to_string()))
                }
                _ => {}
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    /// Serialize to pretty, deterministic JSON (map order is fixed by
    /// `BTreeMap`; `to_string` on keys is stable).
    pub fn to_json_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.styles.len() > 4096
            || self.modifiers.len() > 4096
            || self.relationships.len() > 4096
        {
            return Err("IR collection size limit exceeded".into());
        }
        if self.meta.ir_version != IR_VERSION {
            return Err(format!(
                "unsupported IR version {}; explicit migration required (engine {})",
                self.meta.ir_version, IR_VERSION
            ));
        }
        if self.meta.name.trim().is_empty() {
            return Err("theme name must not be empty".into());
        }
        for (role, style) in &self.styles {
            if role.to_string().parse::<Role>().is_err() {
                return Err(format!("invalid role {role}"));
            }
            style.validate().map_err(|e| format!("{role}: {e}"))?;
        }
        for entry in &self.modifiers {
            if !self.styles.contains_key(&entry.role) {
                return Err(format!("modifier references missing role {}", entry.role));
            }
            if entry.modifiers.is_empty()
                || entry.modifiers.iter().any(|m| {
                    m.is_empty() || !m.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                })
            {
                return Err("invalid semantic modifiers".into());
            }
            entry.style.validate()?;
        }
        let mut parents = std::collections::BTreeSet::new();
        for edge in &self.relationships {
            if edge.kind == RelationshipKind::Inherits && !parents.insert(edge.from.clone()) {
                return Err("multiple inheritance parents are ambiguous".into());
            }
            if !self.styles.contains_key(&edge.from) || !self.styles.contains_key(&edge.to) {
                return Err("relationship references missing role".into());
            }
        }
        for role in self.styles.keys() {
            let mut seen = std::collections::BTreeSet::new();
            let mut current = role;
            while let Some(edge) = self
                .relationships
                .iter()
                .find(|e| e.from == *current && e.kind == RelationshipKind::Inherits)
            {
                if !seen.insert(current.clone()) {
                    return Err("inheritance cycle".into());
                }
                current = &edge.to;
            }
        }
        Ok(())
    }

    pub fn from_json_str(json: &str) -> serde_json::Result<Self> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        if value
            .pointer("/meta/ir_version")
            .and_then(serde_json::Value::as_u64)
            != Some(IR_VERSION as u64)
        {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "explicit ir_version required; use migration for v0",
            ));
        }
        let theme: Self = serde_json::from_value(value)?;
        theme
            .validate()
            .map_err(<serde_json::Error as serde::de::Error>::custom)?;
        Ok(theme)
    }

    /// Explicit compatibility conversion; never guesses at newer schemas.
    pub fn migrate_v0(json: &str) -> serde_json::Result<Self> {
        let mut value: serde_json::Value = serde_json::from_str(json)?;
        if !value.get("meta").is_some_and(serde_json::Value::is_object)
            || !value
                .get("styles")
                .is_some_and(serde_json::Value::is_object)
        {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "migration requires an IR object with meta and styles",
            ));
        }
        let version = match value.pointer("/meta/ir_version") {
            None => 0,
            Some(v) => v.as_u64().ok_or_else(|| {
                <serde_json::Error as serde::de::Error>::custom("invalid IR version")
            })?,
        };
        if version != 0 {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "migration accepts IR v0 only",
            ));
        }
        value["meta"]["ir_version"] = serde_json::json!(IR_VERSION);
        let mut theme: Self = serde_json::from_value(value)?;
        for style in theme.styles.values_mut() {
            if style.provenance == crate::ir::style::Provenance::Locked {
                style.locked.extend(crate::ir::style::Property::ALL);
                style.provenance = crate::ir::style::Provenance::Ingested;
            }
        }
        theme
            .validate()
            .map_err(<serde_json::Error as serde::de::Error>::custom)?;
        Ok(theme)
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {} roles)",
            self.meta.name,
            match self.meta.appearance {
                Appearance::Dark => "dark",
                Appearance::Light => "light",
            },
            self.styles.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::style::Provenance;

    fn gruvbox_ish() -> Theme {
        let mut t = Theme::new("gruvbox-ish", Appearance::Dark);
        t.set(
            Role::Surface(Surface::Base),
            Style::bg(Srgb::from_hex("#282828").unwrap()),
        );
        t.set(
            Role::Text(TextRole::Primary),
            Style::fg(Srgb::from_hex("#ebdbb2").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Comment),
            Style::fg(Srgb::from_hex("#928374").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::String),
            Style::fg(Srgb::from_hex("#b8bb26").unwrap()),
        );
        t
    }

    #[test]
    fn json_roundtrip_preserves_everything() {
        let t = gruvbox_ish();
        let json = t.to_json_string().unwrap();
        let back = Theme::from_json_str(&json).unwrap();
        assert_eq!(t, back);
        assert_eq!(back.meta.ir_version, IR_VERSION);
    }

    #[test]
    fn serialization_is_deterministic_and_diffable() {
        let t = gruvbox_ish();
        let a = t.to_json_string().unwrap();
        let b = t.to_json_string().unwrap();
        assert_eq!(a, b);
        // Role keys appear as human-readable dotted strings.
        assert!(a.contains("\"syntax.comment\""));
        assert!(a.contains("\"surface.base\""));
        // And hex colors stay lowercase hex strings.
        assert!(a.contains("\"#282828\""));
    }

    #[test]
    fn core_role_check_reports_missing_pieces() {
        let mut t = gruvbox_ish();
        // Missing plenty of required roles right now.
        let errs = t.check_core_roles().unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.to_string().contains("syntax.keyword")));

        // Complete the required set -> passes.
        for role in [
            Role::Surface(Surface::Panel),
            Role::Surface(Surface::Float),
            Role::Text(TextRole::Secondary),
            Role::Syntax(SyntaxRole::Keyword),
            Role::Syntax(SyntaxRole::Function),
            Role::Syntax(SyntaxRole::Variable),
            Role::Syntax(SyntaxRole::Type),
        ] {
            t.set(role, Style::fg(Srgb::new(0xde, 0xde, 0xde)));
        }
        assert!(t.check_core_roles().is_ok());
    }

    #[test]
    fn fg_contrast_against_base() {
        let t = gruvbox_ish();
        let base = t.base_bg().unwrap();
        let text = t
            .fg_contrast_against(&Role::Text(TextRole::Primary), base)
            .unwrap();
        let comment = t
            .fg_contrast_against(&Role::Syntax(SyntaxRole::Comment), base)
            .unwrap();
        // Matches the numbers from the theme study: ~10.7 vs ~4.0.
        assert!((text - 10.75).abs() < 0.05, "text {text}");
        assert!((comment - 4.02).abs() < 0.05, "comment {comment}");
    }

    #[test]
    fn locked_provenance_survives_serialization() {
        let mut t = gruvbox_ish();
        t.set(
            Role::Surface(Surface::Base),
            Style::bg(Srgb::from_hex("#1d2021").unwrap()).with_provenance(Provenance::Locked),
        );
        let json = t.to_json_string().unwrap();
        assert!(json.contains("\"locked\""));
        let back = Theme::from_json_str(&json).unwrap();
        assert_eq!(
            back.get(&Role::Surface(Surface::Base)).unwrap().provenance,
            Provenance::Locked
        );
    }
}
