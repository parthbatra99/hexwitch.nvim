//! Gamut mapping by OKLCH chroma reduction.
//!
//! Strategy (after CSS Color 4's approach): when a color falls outside
//! sRGB, reduce chroma while holding lightness and hue fixed, via binary
//! search on the largest in-gamut chroma. This keeps a theme's hue identity
//! ("this is still our orange") at the cost of saturation — never a silent
//! per-channel clip that would skew hue.
//!
//! Determinism: fixed iteration count, no dependence on ambient state.

use crate::color::ok::Oklch;
use crate::color::srgb::{LinearRgb, Srgb};

/// Tolerance for treating floating-point noise at the gamut boundary as
/// in-gamut during the search.
const BOUNDARY_EPS: f64 = 1e-4;

/// Iterations of the binary search. 18 halvings from any starting chroma
/// over-shoot the 8-bit output resolution by a wide margin.
const ITERS: usize = 18;

/// Map an arbitrary OKLCH color into the sRGB gamut.
///
/// - If the color is already in gamut, it converts exactly (no chroma lost).
/// - Otherwise chroma is reduced with lightness and hue preserved.
/// - Lightness outside `[0, 1]` (representable only by whites/blacks beyond
///   sRGB) is clamped onto the neutral axis first.
pub fn gamut_map(color: Oklch) -> Srgb {
    let l = color.l.clamp(0.0, 1.0);

    let direct = Oklch::new(l, color.c, color.h).to_linear();
    if direct.in_gamut(BOUNDARY_EPS) {
        return Srgb::from_linear_clamped(direct);
    }

    let mut lo = 0.0f64; // neutral axis: always in gamut after L clamp
    let mut hi = color.c;
    for _ in 0..ITERS {
        let mid = 0.5 * (lo + hi);
        let candidate: LinearRgb = Oklch::new(l, mid, color.h).to_linear();
        if candidate.in_gamut(BOUNDARY_EPS) {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    Srgb::from_linear_clamped(Oklch::new(l, lo, color.h).to_linear())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ok::Oklab;

    /// Hue/lightness drift introduced by the full map + 8-bit encode path.
    fn roundtrip_drift(color: Oklch) -> (f64, f64) {
        let mapped = gamut_map(color).to_oklch();
        (color.h - mapped.h, color.l - mapped.l)
    }

    #[test]
    fn in_gamut_colors_pass_through_untouched() {
        // Colors already representable in sRGB must not lose any chroma.
        for hex in ["#8ec07c", "#d3869b", "#83a598", "#fabd2f", "#282828"] {
            let c = crate::color::srgb::Srgb::from_hex(hex).unwrap();
            let back = gamut_map(c.to_oklch());
            assert_eq!(back.to_hex(), c.to_hex(), "{hex} changed by gamut map");
        }
    }

    #[test]
    fn out_of_gamut_chroma_is_reduced_not_clipped() {
        // Aggressive chroma at a red hue: far outside sRGB.
        let start = Oklch::new(0.7, 0.4, 29.23);
        let mapped = gamut_map(start);
        let mapped_lch = mapped.to_oklch();

        // Lightness and hue survive. (Up to ~0.5° hue drift is expected:
        // the found chroma sits at the gamut boundary, and the final 8-bit
        // quantization nudges the round-trip.)
        assert!(
            (start.l - mapped_lch.l).abs() < 2e-3,
            "lightness drifted: {}",
            mapped_lch.l
        );
        assert!(
            (start.h - mapped_lch.h).abs() < 0.5,
            "hue drifted: {}",
            mapped_lch.h
        );
        // Chroma landed at the real sRGB boundary for this lightness/hue:
        // measured max is ~0.19 at l=0.7 in the red direction — a genuine
        // gamut fact, kept as a golden so mapping regressions surface.
        assert!(
            (0.15..0.25).contains(&mapped_lch.c) && mapped_lch.c < start.c,
            "chroma: {}",
            mapped_lch.c
        );
    }

    #[test]
    fn extreme_lightness_clamps_to_neutral_axis() {
        let white_side = gamut_map(Oklch::new(1.05, 0.1, 90.0));
        assert_eq!(white_side, crate::color::srgb::Srgb::WHITE);
        let black_side = gamut_map(Oklch::new(-0.05, 0.1, 90.0));
        assert_eq!(black_side, crate::color::srgb::Srgb::BLACK);
    }

    #[test]
    fn mapping_is_deterministic() {
        let c = Oklch::new(0.55, 0.28, 264.05);
        assert_eq!(gamut_map(c), gamut_map(c));
    }

    #[test]
    fn hue_identity_survives_across_lightness_range() {
        // A theme "orange" asked for at several lightness levels keeps the
        // same hue band after mapping — this is what preserves identity.
        // Tolerance ~1°: boundary chroma + 8-bit quantization cost.
        for l in [0.3, 0.5, 0.7, 0.85] {
            let (h_drift, _) = roundtrip_drift(Oklch::new(l, 0.25, 55.0));
            assert!(h_drift.abs() < 1.0, "hue drift {h_drift} at l={l}");
        }
    }

    #[test]
    fn oklab_distance_zero_for_identical_colors() {
        let c = Oklab::from_srgb(crate::color::srgb::Srgb::from_hex("#fb4934").unwrap());
        assert!(c.distance(c) < 1e-12);
    }
}
