use crate::color::{composite_over, Srgb};
use crate::ir::{Property, PropertyState, Role, Theme};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Backdrop {
    Color(Srgb),
    Scene(BackgroundScene),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackgroundScene {
    pub surface: Role,
    #[serde(default)]
    pub overlays: Vec<Role>,
}

impl Backdrop {
    pub fn validate(&self, theme: &Theme) -> Result<(), String> {
        if let Self::Scene(scene) = self {
            if scene.overlays.len() > 32 {
                return Err("background overlay limit exceeded".into());
            }
            for role in std::iter::once(&scene.surface).chain(&scene.overlays) {
                if !theme.styles.contains_key(role) {
                    return Err(format!("missing backdrop role {role}"));
                }
            }
        }
        Ok(())
    }
    pub fn resolve(&self, theme: &Theme) -> Result<Srgb, String> {
        self.validate(theme)?;
        match self {
            Self::Color(color) => Ok(*color),
            Self::Scene(scene) => {
                let surface = crate::validate::resolved_style(theme, &scene.surface)
                    .ok_or("missing resolved surface")?;
                let mut background = surface.bg.ok_or_else(|| {
                    format!("background role {} has no background", scene.surface)
                })?;
                if surface.bg_opacity.unwrap_or(1.0) < 1.0 {
                    return Err("translucent surface needs an external backdrop".into());
                }
                for role in &scene.overlays {
                    let overlay = crate::validate::resolved_style(theme, role)
                        .ok_or("missing resolved overlay")?;
                    if overlay.states.get(&Property::Bg) == Some(&PropertyState::Clear)
                        || overlay.blend.is_some_and(|b| b > 0)
                    {
                        return Err(format!("unverified background semantics for {role}"));
                    }
                    if overlay.fg.is_some() {
                        return Err(format!("overlay {role} replaces foreground; contrast requires scene validation"));
                    }
                    if let Some(color) = overlay.bg {
                        background =
                            composite_over(color, overlay.bg_opacity.unwrap_or(1.0), background);
                    } else {
                        return Err(format!("overlay {role} has no background treatment"));
                    }
                }
                Ok(background)
            }
        }
    }
}

pub fn resolve_all(theme: &Theme, backdrops: &[Backdrop]) -> Result<Vec<Srgb>, String> {
    if backdrops.is_empty() || backdrops.len() > 64 {
        return Err("background count must be 1..64".into());
    }
    backdrops.iter().map(|b| b.resolve(theme)).collect()
}
