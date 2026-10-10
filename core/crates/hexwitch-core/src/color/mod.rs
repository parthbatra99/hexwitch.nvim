//! Deterministic color science for the theme engine.
//!
//! Design rules, established during the first-principles review:
//!
//! * **Authoring space is not validation space.** [`Oklch`] is used to
//!   *manipulate* colors (lightness/chroma/hue edits, identity preservation).
//!   WCAG [`contrast_ratio`](crate::color::contrast_ratio) is computed
//!   independently from linear-light relative luminance. Equal OKLab
//!   lightness does not imply equal WCAG contrast, and neither is a comfort
//!   metric. Both facts are respected by keeping the computations separate.
//! * **Everything is `f64`.** Themes are compiled a handful of times and
//!   inspected by humans; determinism and clarity beat speed here.
//! * **Conversions follow published references**: sRGB transfer functions
//!   per IEC 61966-2-1 / CSS Color 4, OKLab per Björn Ottosson's reference
//!   implementation, WCAG relative luminance per WCAG 2.2.
//! * **Gamut mapping is explicit** ([`gamut::gamut_map`]): chroma reduction
//!   in OKLCH, never silent per-channel clipping. Any color that leaves
//!   this module as [`Srgb`] is in gamut by construction (or deliberately
//!   clamped at a documented boundary).

pub mod compose;
pub mod gamut;
pub mod ok;
pub mod srgb;

pub use compose::composite_over;
pub use gamut::gamut_map;
pub use ok::{Oklab, Oklch};
pub use srgb::{contrast_ratio, contrast_verdict, ContrastVerdict, LinearRgb, Srgb};
