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
    #[serde(default = "default_ir_version")]
    pub ir_version: u32,
}

fn default_ir_version() -> u32 {
    IR_VERSION
}

/// A theme: one appearance mode. Light + dark variants sharing an identity
/// are two `Theme`s linked by a family concept (added with revise).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub meta: ThemeMeta,
    /// Role -> treatment. `BTreeMap` for deterministic, diff-friendly JSON.
    pub styles: BTreeMap<Role, Style>,
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
        self.fg(role).map(|fg| contrast_ratio(fg, backdrop))
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

    pub fn from_json_str(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
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
