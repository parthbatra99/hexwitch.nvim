use crate::color::ok::Oklch;
use crate::color::srgb::{LinearRgb, Srgb};

const BOUNDARY_EPS: f64 = 1e-4;

const ITERS: usize = 18;

pub fn gamut_map(color: Oklch) -> Srgb {
    let l = color.l.clamp(0.0, 1.0);

    let direct = Oklch::new(l, color.c, color.h).to_linear();
    if direct.in_gamut(BOUNDARY_EPS) {
        return Srgb::from_linear_clamped(direct);
    }

    let mut lo = 0.0f64;
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

    fn roundtrip_drift(color: Oklch) -> (f64, f64) {
        let mapped = gamut_map(color).to_oklch();
        (color.h - mapped.h, color.l - mapped.l)
    }

    #[test]
    fn in_gamut_colors_pass_through_untouched() {
        for hex in ["#8ec07c", "#d3869b", "#83a598", "#fabd2f", "#282828"] {
            let c = crate::color::srgb::Srgb::from_hex(hex).unwrap();
            let back = gamut_map(c.to_oklch());
            assert_eq!(back.to_hex(), c.to_hex(), "{hex} changed by gamut map");
        }
    }

    #[test]
    fn out_of_gamut_chroma_is_reduced_not_clipped() {
        let start = Oklch::new(0.7, 0.4, 29.23);
        let mapped = gamut_map(start);
        let mapped_lch = mapped.to_oklch();

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
