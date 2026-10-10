//! Resolved color scenes. These are calculated profiles, not editor screenshots.
use crate::color::{composite_over, contrast_ratio, Srgb};
use crate::ir::{Property, PropertyState, RelationshipKind, Role, Style, Theme};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub name: String,
    pub foreground: Role,
    pub surface: Role,
    #[serde(default)]
    pub overlays: Vec<Role>,
    pub min_contrast: f64,
    #[serde(default)]
    pub ordering_verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic: Option<crate::semantic::SemanticContext>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Unverified,
}
#[derive(Debug, Clone, Serialize)]
pub struct SceneResult {
    pub name: String,
    pub status: Status,
    pub foreground: Option<Srgb>,
    pub background: Option<Srgb>,
    pub contrast: Option<f64>,
    pub reasons: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub status: Status,
    pub calculated_profile: String,
    pub scenes: Vec<SceneResult>,
}

pub fn resolved_style(theme: &Theme, role: &Role) -> Option<Style> {
    fn resolve(
        theme: &Theme,
        role: &Role,
        seen: &mut std::collections::BTreeSet<Role>,
    ) -> Option<Style> {
        if !seen.insert(role.clone()) {
            return None;
        }
        let own = theme.get(role)?.clone();
        let inherited = theme
            .relationships
            .iter()
            .find(|e| e.from == *role && e.kind == RelationshipKind::Inherits)
            .and_then(|e| resolve(theme, &e.to, seen));
        let Some(parent) = inherited else {
            return Some(own);
        };
        let mut style = own.clone();
        for property in Property::ALL {
            if own.has_value(property) || own.states.get(&property) == Some(&PropertyState::Clear) {
                continue;
            }
            match property {
                Property::Fg => style.fg = parent.fg,
                Property::Bg => style.bg = parent.bg,
                Property::Special => style.special = parent.special,
                Property::FgOpacity => style.fg_opacity = parent.fg_opacity,
                Property::BgOpacity => style.bg_opacity = parent.bg_opacity,
                Property::SpecialOpacity => style.special_opacity = parent.special_opacity,
                Property::Bold => style.bold = parent.bold,
                Property::Italic => style.italic = parent.italic,
                Property::Underline => style.underline = parent.underline,
                Property::Strikethrough => style.strikethrough = parent.strikethrough,
                Property::Blend => style.blend = parent.blend,
            }
            if style.has_value(property) {
                style.states.remove(&property);
            }
        }
        Some(style)
    }
    resolve(theme, role, &mut std::collections::BTreeSet::new())
}

pub fn validate_scenes(theme: &Theme, scenes: &[Scene]) -> Result<ValidationReport, String> {
    theme.validate()?;
    if scenes.is_empty() || scenes.len() > 4096 {
        return Err("scene count must be 1..4096".into());
    }
    let mut results = vec![];
    for scene in scenes {
        if !scene.min_contrast.is_finite()
            || !(1.0..=21.0).contains(&scene.min_contrast)
            || scene.overlays.len() > 32
        {
            return Err("invalid scene constraints".into());
        }
        let mut reasons = vec![];
        let surface = resolved_style(theme, &scene.surface);
        let mut bg = surface.as_ref().and_then(|s| s.bg);
        if surface
            .as_ref()
            .is_some_and(|s| s.bg_opacity.unwrap_or(1.0) < 1.0)
        {
            bg = None;
            reasons.push("surface needs an unknown external backdrop".into());
        }
        let mut text = resolved_style(theme, &scene.foreground);
        if let Some(context) = &scene.semantic {
            context.validate()?;
            if let Some(base) = &text {
                let resolved =
                    crate::semantic::resolve_semantic(theme, &scene.foreground, context, base)?;
                reasons.extend(resolved.limitations);
                text = Some(resolved.style);
            }
        }
        let mut fg = text.as_ref().and_then(|s| s.fg);
        let mut fg_alpha = text.as_ref().and_then(|s| s.fg_opacity).unwrap_or(1.0);
        if fg.is_none() {
            reasons.push(format!("missing foreground {}", scene.foreground));
        }
        if bg.is_none() {
            reasons.push(format!("missing opaque surface {}", scene.surface));
        }
        for overlay in &scene.overlays {
            match resolved_style(theme, overlay) {
                None => reasons.push(format!("missing overlay {overlay}")),
                Some(style) => {
                    if style.states.get(&Property::Bg) == Some(&PropertyState::Clear)
                        || style.states.get(&Property::Fg) == Some(&PropertyState::Clear)
                    {
                        reasons.push(format!("editor clear semantics unverified for {overlay}"));
                    }
                    if let Some(layer) = style.bg {
                        bg = bg.map(|backdrop| {
                            composite_over(layer, style.bg_opacity.unwrap_or(1.0), backdrop)
                        });
                    }
                    if let Some(replacement) = style.fg {
                        fg = Some(replacement);
                        fg_alpha = style.fg_opacity.unwrap_or(1.0);
                    }
                    if style.blend.is_some_and(|b| b > 0) {
                        reasons.push(format!("window blend ordering unverified for {overlay}"));
                    }
                    // Decoration-only diagnostics do not replace syntax foreground.
                    if style.fg.is_none() && style.bg.is_none() && style.special.is_none() {
                        reasons.push(format!("overlay has no color treatment {overlay}"));
                    }
                }
            }
        }
        let final_fg = fg.zip(bg).map(|(c, b)| composite_over(c, fg_alpha, b));
        let contrast = final_fg.zip(bg).map(|(f, b)| contrast_ratio(f, b));
        if !scene.overlays.is_empty() && !scene.ordering_verified {
            reasons.push("target overlay priority not verified".into());
        }
        let status = match contrast {
            Some(ratio) if reasons.is_empty() => {
                if ratio >= scene.min_contrast {
                    Status::Pass
                } else {
                    Status::Fail
                }
            }
            _ => Status::Unverified,
        };
        results.push(SceneResult {
            name: scene.name.clone(),
            status,
            foreground: final_fg,
            background: bg,
            contrast,
            reasons,
        });
    }
    let status = if results.iter().any(|s| s.status == Status::Fail) {
        Status::Fail
    } else if results.iter().any(|s| s.status == Status::Unverified) {
        Status::Unverified
    } else {
        Status::Pass
    };
    Ok(ValidationReport {
        status,
        calculated_profile:
            "gamma-encoded sRGB source-over; explicit overlay sequence; not live editor resolution"
                .into(),
        scenes: results,
    })
}

pub fn initial_scenes() -> Vec<Scene> {
    let role = |s: &str| s.parse::<Role>().unwrap();
    let mut scenes = vec![];
    for foreground in [
        "text.primary",
        "syntax.comment",
        "syntax.keyword",
        "syntax.string",
    ] {
        for (name, overlays) in [
            ("normal", vec![]),
            ("selection", vec!["interaction.selection"]),
            ("inactive-selection", vec!["interaction.selection-inactive"]),
            ("current-line", vec!["interaction.cursor-line"]),
            ("search", vec!["interaction.search"]),
            ("current-search", vec!["interaction.search-current"]),
            ("added", vec!["diff.added.line-bg"]),
            ("removed", vec!["diff.removed.line-bg"]),
            ("modified", vec!["diff.changed.line-bg"]),
            (
                "selected-diff",
                vec!["diff.added.line-bg", "interaction.selection"],
            ),
            (
                "selected-search",
                vec!["interaction.search", "interaction.selection"],
            ),
            ("diagnostic-decoration", vec!["diagnostic.error.text"]),
        ] {
            scenes.push(Scene {
                name: format!("{name}/{foreground}"),
                foreground: role(foreground),
                surface: role("surface.base"),
                overlays: overlays.into_iter().map(role).collect(),
                min_contrast: 4.5,
                ordering_verified: false,
                semantic: None,
            });
        }
    }
    scenes.push(Scene {
        name: "terminal/default".into(),
        foreground: role("other.terminal-default"),
        surface: role("other.terminal-default"),
        overlays: vec![],
        min_contrast: 4.5,
        ordering_verified: false,
        semantic: None,
    });
    scenes.push(Scene {
        name: "completion-selected".into(),
        foreground: role("text.primary"),
        surface: role("surface.float"),
        overlays: vec![role("other.completion-selection")],
        min_contrast: 4.5,
        ordering_verified: false,
        semantic: None,
    });
    for kind in ["error", "warning", "info", "hint"] {
        scenes.push(Scene {
            name: format!("floating-diagnostic/{kind}"),
            foreground: role(&format!("diagnostic.{kind}.text")),
            surface: role("surface.float"),
            overlays: vec![],
            min_contrast: 4.5,
            ordering_verified: false,
            semantic: None,
        });
    }
    for i in 0..16 {
        scenes.push(Scene {
            name: format!("terminal/{i}"),
            foreground: Role::Terminal(i),
            surface: role("other.terminal-default"),
            overlays: vec![],
            min_contrast: 4.5,
            ordering_verified: false,
            semantic: None,
        });
    }
    scenes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Appearance;
    #[test]
    fn unknown_is_not_pass_and_alpha_replacement_is_composed() {
        let mut t = Theme::new("scene", Appearance::Dark);
        t.set("surface.base".parse().unwrap(), Style::bg(Srgb::BLACK));
        t.set("text.primary".parse().unwrap(), Style::fg(Srgb::WHITE));
        let mut s = Style::bg(Srgb::WHITE);
        s.bg_opacity = Some(0.5);
        t.set("interaction.selection".parse().unwrap(), s);
        let mut scene = Scene {
            name: "selected".into(),
            foreground: "text.primary".parse().unwrap(),
            surface: "surface.base".parse().unwrap(),
            overlays: vec!["interaction.selection".parse().unwrap()],
            min_contrast: 4.5,
            ordering_verified: false,
            semantic: None,
        };
        assert_eq!(
            validate_scenes(&t, &[scene.clone()]).unwrap().status,
            Status::Unverified
        );
        scene.ordering_verified = true;
        let report = validate_scenes(&t, &[scene]).unwrap();
        assert_eq!(report.status, Status::Fail);
        assert_eq!(report.scenes[0].background, Some(Srgb::new(128, 128, 128)));
        assert_eq!(
            validate_scenes(&t, &initial_scenes())
                .unwrap()
                .scenes
                .iter()
                .find(|s| s.name == "completion-selected")
                .unwrap()
                .status,
            Status::Unverified
        );
    }
}
