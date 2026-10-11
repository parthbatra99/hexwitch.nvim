use serde::{Deserialize, Serialize};

use crate::color::srgb::{LinearRgb, Srgb};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Oklab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

impl Oklab {
    pub fn from_linear(lin: LinearRgb) -> Self {
        let l_ = (0.4122214708 * lin.r + 0.5363325363 * lin.g + 0.0514459929 * lin.b).cbrt();
        let m_ = (0.2119034982 * lin.r + 0.6806995451 * lin.g + 0.1073969566 * lin.b).cbrt();
        let s_ = (0.0883024619 * lin.r + 0.2817188376 * lin.g + 0.6299787005 * lin.b).cbrt();

        Oklab {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
        }
    }

    pub fn from_srgb(c: Srgb) -> Self {
        Self::from_linear(c.to_linear())
    }

    pub fn to_linear(self) -> LinearRgb {
        let l = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s = self.l - 0.0894841775 * self.a - 1.2914855480 * self.b;

        let l_ = l * l * l;
        let m_ = m * m * m;
        let s_ = s * s * s;

        LinearRgb {
            r: 4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
            g: -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
            b: -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_,
        }
    }

    pub fn to_srgb_clamped(self) -> Srgb {
        Srgb::from_linear_clamped(self.to_linear())
    }

    pub fn to_oklch(self) -> Oklch {
        let c = (self.a * self.a + self.b * self.b).sqrt();
        let mut h = self.b.atan2(self.a).to_degrees();
        if h < 0.0 {
            h += 360.0;
        }
        Oklch { l: self.l, c, h }
    }

    pub fn distance(self, other: Oklab) -> f64 {
        ((self.l - other.l).powi(2) + (self.a - other.a).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }
}

impl Oklch {
    pub fn new(l: f64, c: f64, h: f64) -> Self {
        let mut h = h % 360.0;
        if h < 0.0 {
            h += 360.0;
        }
        Oklch { l, c, h }
    }

    pub fn to_oklab(self) -> Oklab {
        let h = self.h.to_radians();
        Oklab {
            l: self.l,
            a: self.c * h.cos(),
            b: self.c * h.sin(),
        }
    }

    pub fn to_linear(self) -> LinearRgb {
        self.to_oklab().to_linear()
    }

    pub fn to_srgb_clamped(self) -> Srgb {
        Srgb::from_linear_clamped(self.to_linear())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, eps: f64, what: &str) {
        assert!(
            (actual - expected).abs() < eps,
            "{what}: expected {expected}, got {actual}"
        );
    }

    #[test]
    fn white_and_black_anchors() {
        let white = Oklab::from_srgb(Srgb::WHITE);
        assert_close(white.l, 1.0, 1e-7, "white L");
        assert_close(white.a, 0.0, 1e-7, "white a");
        assert_close(white.b, 0.0, 1e-7, "white b");

        let black = Oklab::from_srgb(Srgb::BLACK);
        assert_close(black.l, 0.0, 1e-9, "black L");
        assert_close(black.a, 0.0, 1e-9, "black a");
        assert_close(black.b, 0.0, 1e-9, "black b");
    }

    #[test]
    fn pure_red_anchor() {
        let red = Oklab::from_srgb(Srgb::new(255, 0, 0));
        assert_close(red.l, 0.6279554, 1e-5, "red L");
        assert_close(red.a, 0.2248630, 1e-5, "red a");
        assert_close(red.b, 0.1258423, 1e-5, "red b");

        let lch = red.to_oklch();
        assert_close(lch.h, 29.23, 0.02, "red hue");
        assert_close(lch.c, 0.25768, 1e-3, "red chroma");
    }

    #[test]
    fn midgray_is_neutral_with_known_lightness() {
        let gray = Oklab::from_srgb(Srgb::from_hex("#808080").unwrap());
        assert_close(gray.l, 0.6, 1e-3, "gray L");
        assert_close(gray.a, 0.0, 1e-6, "gray a");
        assert_close(gray.b, 0.0, 1e-6, "gray b");
    }

    #[test]
    fn srgb_oklab_roundtrip_across_cube() {
        let mut checked = 0;
        let mut max_err: f64 = 0.0;
        for r in (0..=255).step_by(5) {
            for g in (0..=255).step_by(5) {
                for b in (0..=255).step_by(5) {
                    let c = Srgb::new(r as u8, g as u8, b as u8);
                    let back = c.to_oklab().to_srgb_clamped();
                    let err = (back.r as i32 - c.r as i32)
                        .abs()
                        .max((back.g as i32 - c.g as i32).abs())
                        .max((back.b as i32 - c.b as i32).abs());
                    max_err = max_err.max(err as f64);
                    assert!(err <= 1, "roundtrip moved {c} to {back}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 100_000);
        assert!(max_err <= 1.0);
    }

    #[test]
    fn oklch_oklab_roundtrip() {
        for l in [0.0, 0.1, 0.5, 0.9, 1.0] {
            for c in [0.0, 0.05, 0.15, 0.3] {
                for h in [0.0, 29.23, 90.0, 142.5, 264.05, 350.0] {
                    let start = Oklch::new(l, c, h);
                    let back = start.to_oklab().to_oklch();
                    assert!((start.l - back.l).abs() < 1e-12, "l drift at {start:?}");
                    assert!((start.c - back.c).abs() < 1e-12, "c drift at {start:?}");
                    if c > 1e-9 && h != 0.0 {
                        assert!((start.h - back.h).abs() < 1e-9, "h drift at {start:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn hue_wraps_into_0_360() {
        assert_eq!(Oklch::new(0.5, 0.1, -30.0).h, 330.0);
        assert_eq!(Oklch::new(0.5, 0.1, 390.0).h, 30.0);
        assert!(Oklab::from_srgb(Srgb::BLACK).l.is_finite());
    }
}
