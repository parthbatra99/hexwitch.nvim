//! Analyze: measure what a theme *is* numerically.
//!
//! This is the evaluation lens from the first-principles review: themes
//! are compared not by taste but by measurable structure — the contrast
//! ladder (how foreground hierarchy relates to the base surface), the
//! accent budget (how lightness/chroma/hue are spent on syntax), and
//! simple polarity invariants.
//!
//! What these numbers are NOT: proof of readability, comfort, or
//! superiority. They are diagnostics that make design decisions explicit
//! and revision effects visible.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::color::{contrast_ratio, Srgb};
use crate::ir::role::{Role, Surface, SyntaxRole, TextRole};
use crate::ir::theme::{Appearance, Theme};

#[derive(Debug, Error, PartialEq)]
pub enum AnalyzeError {
    #[error("theme `{0}` has no surface.base background; cannot measure against it")]
    MissingBase(String),
}

/// One rung of the contrast ladder: a foreground-bearing role and its
/// WCAG ratio against the theme's base background.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleContrast {
    pub role: String,
    pub ratio: f64,
}

/// OKLCH view of a syntax accent, for the accent-budget picture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentStats {
    pub role: String,
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

/// A theme's measurable fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    pub name: String,
    pub appearance: Appearance,
    pub base_bg: String,
    /// Foreground-bearing roles vs base background, highest ratio first.
    pub contrast_ladder: Vec<RoleContrast>,
    /// Syntax roles with a foreground color.
    pub syntax_roles: usize,
    /// OKLCH decomposition of syntax accents (identity lives here).
    pub accents: Vec<AccentStats>,
    /// Highest accent chroma — one number for "how vivid" a theme is.
    pub max_accent_chroma: f64,
    /// Spread of accent lightness — one number for "how much hierarchy is
    /// carried by color lightness vs hue".
    pub accent_lightness_spread: f64,
}

impl Fingerprint {
    pub fn primary_contrast(&self) -> Option<f64> {
        self.contrast_ladder
            .iter()
            .find(|rc| rc.role == "text.primary")
            .map(|rc| rc.ratio)
    }

    pub fn comment_contrast(&self) -> Option<f64> {
        self.contrast_ladder
            .iter()
            .find(|rc| rc.role == "syntax.comment")
            .map(|rc| rc.ratio)
    }
}

/// Compute a theme's fingerprint.
pub fn fingerprint(theme: &Theme) -> Result<Fingerprint, AnalyzeError> {
    let base = theme
        .base_bg()
        .ok_or_else(|| AnalyzeError::MissingBase(theme.meta.name.clone()))?;

    let mut ladder: Vec<RoleContrast> = theme
        .styles
        .iter()
        .filter_map(|(role, style)| {
            style.fg.map(|fg| RoleContrast {
                role: role.to_string(),
                ratio: contrast_ratio(fg, base),
            })
        })
        .collect();
    ladder.sort_by(|a, b| {
        b.ratio
            .partial_cmp(&a.ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut accents: Vec<AccentStats> = theme
        .styles
        .iter()
        .filter_map(|(role, style)| {
            let fg: Srgb = style.fg?;
            if matches!(role, Role::Syntax(_)) {
                let oklch = fg.to_oklch();
                Some(AccentStats {
                    role: role.to_string(),
                    l: oklch.l,
                    c: oklch.c,
                    h: oklch.h,
                })
            } else {
                None
            }
        })
        .collect();
    accents.sort_by(|a, b| a.role.cmp(&b.role));

    let syntax_roles = theme
        .styles
        .keys()
        .filter(|r| matches!(r, Role::Syntax(_)))
        .count();

    let max_accent_chroma = accents.iter().map(|a| a.c).fold(0.0, f64::max);
    let accent_lightness_spread = accents.iter().map(|a| a.l).fold(f64::MIN, f64::max)
        - accents.iter().map(|a| a.l).fold(f64::MAX, f64::min);

    Ok(Fingerprint {
        name: theme.meta.name.clone(),
        appearance: theme.meta.appearance,
        base_bg: base.to_hex(),
        contrast_ladder: ladder,
        syntax_roles,
        accents,
        max_accent_chroma,
        accent_lightness_spread,
    })
}

/// Polarity sanity: in a dark theme the base background should be darker
/// than the primary foreground (and vice versa). A cheap invariant that
/// catches ingest bugs and accidental inversions during revision.
pub fn polarity_is_consistent(theme: &Theme) -> bool {
    let (Some(base), Some(primary)) = (theme.base_bg(), theme.fg(&Role::Text(TextRole::Primary)))
    else {
        return false;
    };
    match theme.meta.appearance {
        Appearance::Dark => base.relative_luminance() < primary.relative_luminance(),
        Appearance::Light => base.relative_luminance() > primary.relative_luminance(),
    }
}

/// Secondary information (comments) should not outshine primary text.
/// Almost every serious theme obeys this; violations are worth seeing.
pub fn comment_not_brighter_than_primary(theme: &Theme) -> bool {
    let Some(base) = theme.base_bg() else {
        return false;
    };
    let primary = theme.fg(&Role::Text(TextRole::Primary));
    let comment = theme.fg(&Role::Syntax(SyntaxRole::Comment));
    match (primary, comment) {
        (Some(p), Some(c)) => contrast_ratio(c, base) <= contrast_ratio(p, base) + 1e-9,
        _ => false,
    }
}

/// Roles every dark-vs-light comparison and diff view leans on.
pub fn core_surfaces() -> Vec<Role> {
    vec![
        Role::Surface(Surface::Base),
        Role::Surface(Surface::Gutter),
        Role::Surface(Surface::Panel),
        Role::Surface(Surface::Float),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::style::Style;

    fn tiny() -> Theme {
        let mut t = Theme::new("tiny", Appearance::Dark);
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
    fn fingerprint_ladder_and_counts() {
        let f = fingerprint(&tiny()).unwrap();
        assert_eq!(f.syntax_roles, 2);
        assert_eq!(f.base_bg, "#282828");
        // Primary tops the ladder, comments below — gruvbox numbers again.
        assert!((f.primary_contrast().unwrap() - 10.75).abs() < 0.05);
        assert!((f.comment_contrast().unwrap() - 4.02).abs() < 0.05);
        assert_eq!(f.contrast_ladder[0].role, "text.primary");
        assert!(f
            .contrast_ladder
            .iter()
            .any(|rc| rc.role == "syntax.string"));
    }

    #[test]
    fn fingerprint_requires_base() {
        let mut t = Theme::new("empty", Appearance::Dark);
        t.set(Role::Text(TextRole::Primary), Style::fg(Srgb::WHITE));
        assert!(matches!(fingerprint(&t), Err(AnalyzeError::MissingBase(_))));
    }

    #[test]
    fn invariants_hold_for_well_formed_theme() {
        assert!(polarity_is_consistent(&tiny()));
        assert!(comment_not_brighter_than_primary(&tiny()));
    }

    #[test]
    fn invariants_catch_inversion() {
        let mut t = tiny();
        t.meta.appearance = Appearance::Light; // same colors, wrong claim
        assert!(!polarity_is_consistent(&t));
    }

    #[test]
    fn accent_budget_is_measured_in_oklch() {
        let f = fingerprint(&tiny()).unwrap();
        let string_accent = f
            .accents
            .iter()
            .find(|a| a.role == "syntax.string")
            .unwrap();
        // Gruvbox green (#b8bb26) is a *yellow*-green — OKLCH hue ~111°.
        assert!((0.75..0.85).contains(&string_accent.l));
        assert!(string_accent.c > 0.05 && string_accent.c < 0.2);
        assert!(
            (95.0..125.0).contains(&string_accent.h),
            "hue {}",
            string_accent.h
        );
        assert!(f.max_accent_chroma > 0.0);
        assert!(f.accent_lightness_spread >= 0.0);
    }

    #[test]
    fn fingerprint_serializes_for_goldens() {
        let f = fingerprint(&tiny()).unwrap();
        let json = serde_json::to_string(&f).unwrap();
        let back: Fingerprint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.syntax_roles, 2);
        assert_eq!(back.primary_contrast(), f.primary_contrast());
    }
}
