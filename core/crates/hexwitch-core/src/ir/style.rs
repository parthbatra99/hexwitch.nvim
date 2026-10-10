//! Visual treatment attached to a role, plus where it came from.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::color::Srgb;

/// Underline family. Adapters lower this to what the editor supports
/// (VS Code has a single underline; Neovim has the full set) and *report*
/// the degradation — curl vs line can matter for diagnostics that must not
/// rely on color alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnderlineStyle {
    Single,
    Double,
    Curl,
    Dot,
    Dash,
    Strikethrough,
}

/// Why a style has the value it has. The revise engine consults this:
/// `Ingested` colors are the reference identity, `Locked` ones are
/// user-protected and must survive revision untouched, `Derived` styles are
/// recomputed when their inputs change, `Generated` styles were
/// synthesized. Changing provenance silently is forbidden.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Carried over unchanged from the ingested reference.
    #[default]
    Ingested,
    /// Explicitly protected by the user ("don't touch the background").
    Locked,
    /// Computed from other roles (e.g. a selection bg derived from an accent).
    Derived,
    /// Synthesized during revision or generation.
    Generated,
}

/// The full visual treatment for one role.
///
/// Omitted concrete properties are unspecified unless `states` explicitly
/// selects inherit or clear. Opacity is independent from color; omitted
/// opacity is opaque. `locked` is independent from source origin.
///
/// Unknown fields are rejected: the IR is a contract, not a suggestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub states: BTreeMap<Property, PropertyState>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub locked: BTreeSet<Property>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<Srgb>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Srgb>,
    /// Decoration color (underline/undercurl/sign bars), where the editor
    /// separates it from the foreground.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special: Option<Srgb>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underline: Option<UnderlineStyle>,
    /// Independent from underline so both decorations can coexist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    /// Floating-window blend (Neovim `winblend`-style, 0-100), where
    /// supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<u8>,
    #[serde(default, skip_serializing_if = "Provenance::is_ingested")]
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Property {
    Fg,
    Bg,
    Special,
    FgOpacity,
    BgOpacity,
    SpecialOpacity,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Blend,
}

impl Property {
    pub const ALL: [Self; 11] = [
        Self::Fg,
        Self::Bg,
        Self::Special,
        Self::FgOpacity,
        Self::BgOpacity,
        Self::SpecialOpacity,
        Self::Bold,
        Self::Italic,
        Self::Underline,
        Self::Strikethrough,
        Self::Blend,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyState {
    Unspecified,
    Inherit,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInfo {
    pub origin: Origin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Imported,
    Generated,
    Derived,
    User,
}

impl Style {
    pub fn has_value(&self, property: Property) -> bool {
        match property {
            Property::Fg => self.fg.is_some(),
            Property::Bg => self.bg.is_some(),
            Property::Special => self.special.is_some(),
            Property::FgOpacity => self.fg_opacity.is_some(),
            Property::BgOpacity => self.bg_opacity.is_some(),
            Property::SpecialOpacity => self.special_opacity.is_some(),
            Property::Bold => self.bold.is_some(),
            Property::Italic => self.italic.is_some(),
            Property::Underline => self.underline.is_some(),
            Property::Strikethrough => self.strikethrough.is_some(),
            Property::Blend => self.blend.is_some(),
        }
    }
    pub fn clear_value(&mut self, property: Property) {
        match property {
            Property::Fg => self.fg = None,
            Property::Bg => self.bg = None,
            Property::Special => self.special = None,
            Property::FgOpacity => self.fg_opacity = None,
            Property::BgOpacity => self.bg_opacity = None,
            Property::SpecialOpacity => self.special_opacity = None,
            Property::Bold => self.bold = None,
            Property::Italic => self.italic = None,
            Property::Underline => self.underline = None,
            Property::Strikethrough => self.strikethrough = None,
            Property::Blend => self.blend = None,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        for opacity in [self.fg_opacity, self.bg_opacity, self.special_opacity]
            .into_iter()
            .flatten()
        {
            if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
                return Err("opacity must be finite and in 0..1".into());
            }
        }
        if self.blend.is_some_and(|b| b > 100) {
            return Err("blend must be in 0..100".into());
        }
        for property in self.states.keys() {
            if self.has_value(*property) {
                return Err(format!(
                    "{property:?} cannot have both a state and a concrete value"
                ));
            }
        }
        Ok(())
    }
    /// Copy exactly one visual property, including explicit state semantics.
    /// Locks and source metadata are not inherited through this operation.
    pub fn copy_property_from(&mut self, other: &Style, property: Property) {
        self.clear_value(property);
        self.states.remove(&property);
        if let Some(state) = other.states.get(&property) {
            self.states.insert(property, *state);
        }
        match property {
            Property::Fg => self.fg = other.fg,
            Property::Bg => self.bg = other.bg,
            Property::Special => self.special = other.special,
            Property::FgOpacity => self.fg_opacity = other.fg_opacity,
            Property::BgOpacity => self.bg_opacity = other.bg_opacity,
            Property::SpecialOpacity => self.special_opacity = other.special_opacity,
            Property::Bold => self.bold = other.bold,
            Property::Italic => self.italic = other.italic,
            Property::Underline => self.underline = other.underline,
            Property::Strikethrough => self.strikethrough = other.strikethrough,
            Property::Blend => self.blend = other.blend,
        }
    }

    pub fn is_locked(&self, property: Property) -> bool {
        self.provenance == Provenance::Locked || self.locked.contains(&property)
    }
}

impl Provenance {
    pub fn is_ingested(&self) -> bool {
        matches!(self, Provenance::Ingested)
    }
}

impl Style {
    pub fn new() -> Self {
        Style {
            fg_opacity: None,
            bg_opacity: None,
            special_opacity: None,
            states: BTreeMap::new(),
            locked: BTreeSet::new(),
            source: None,
            fg: None,
            bg: None,
            special: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
            blend: None,
            provenance: Provenance::Ingested,
        }
    }

    /// A foreground-only style, the most common shape in a theme.
    pub fn fg(fg: Srgb) -> Self {
        let mut s = Style::new();
        s.fg = Some(fg);
        s
    }

    /// A background-only style (surfaces, line tints).
    pub fn bg(bg: Srgb) -> Self {
        let mut s = Style::new();
        s.bg = Some(bg);
        s
    }

    /// Foreground + background pair.
    pub fn pair(fg: Srgb, bg: Srgb) -> Self {
        let mut s = Style::fg(fg);
        s.bg = Some(bg);
        s
    }

    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = provenance;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_styles_serialize_compactly() {
        let red = Srgb::new(0xfb, 0x49, 0x34);
        let s = Style::fg(red);
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r##"{"fg":"#fb4934"}"##);

        let back: Style = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
        assert_eq!(back.provenance, Provenance::Ingested);
    }

    #[test]
    fn full_style_roundtrips() {
        let s = Style {
            fg: Some(Srgb::new(0xeb, 0xdb, 0xb2)),
            bg: Some(Srgb::new(0x28, 0x28, 0x28)),
            special: Some(Srgb::new(0xcc, 0x24, 0x1d)),
            bold: Some(false),
            italic: Some(true),
            underline: Some(UnderlineStyle::Curl),
            blend: Some(10),
            provenance: Provenance::Locked,
            ..Style::new()
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Style = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn unspecified_fields_stay_absent() {
        let json = serde_json::to_string(&Style::default()).unwrap();
        assert_eq!(json, "{}");
        // Unknown-but-tolerated extras are rejected loudly rather than
        // silently dropped.
        assert!(serde_json::from_str::<Style>(r##"{"fg":"#123456","bold":true}"##).is_ok());
        assert!(serde_json::from_str::<Style>(r#"{"nonsense":1}"#).is_err());
    }
}
