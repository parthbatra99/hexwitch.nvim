//! Visual treatment attached to a role, plus where it came from.

use serde::{Deserialize, Serialize};

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
/// `fg`/`bg`/`special` of `None` mean *unspecified* — the value may be
/// inherited or fall back per adapter rules. It does not mean "transparent"
/// and not "deliberately cleared"; explicit clearing (e.g. `ctermfg=NONE`
/// style semantics) is an adapter concern recorded at lowering time.
///
/// Unknown fields are rejected: the IR is a contract, not a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
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
    /// Floating-window blend (Neovim `winblend`-style, 0-100), where
    /// supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<u8>,
    #[serde(default, skip_serializing_if = "Provenance::is_ingested")]
    pub provenance: Provenance,
}

impl Provenance {
    pub fn is_ingested(&self) -> bool {
        matches!(self, Provenance::Ingested)
    }
}

impl Style {
    pub const fn new() -> Self {
        Style {
            fg: None,
            bg: None,
            special: None,
            bold: None,
            italic: None,
            underline: None,
            blend: None,
            provenance: Provenance::Ingested,
        }
    }

    /// A foreground-only style, the most common shape in a theme.
    pub const fn fg(fg: Srgb) -> Self {
        let mut s = Style::new();
        s.fg = Some(fg);
        s
    }

    /// A background-only style (surfaces, line tints).
    pub const fn bg(bg: Srgb) -> Self {
        let mut s = Style::new();
        s.bg = Some(bg);
        s
    }

    /// Foreground + background pair.
    pub const fn pair(fg: Srgb, bg: Srgb) -> Self {
        let mut s = Style::fg(fg);
        s.bg = Some(bg);
        s
    }

    pub const fn with_provenance(mut self, provenance: Provenance) -> Self {
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
