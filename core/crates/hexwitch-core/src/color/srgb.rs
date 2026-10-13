use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::color::ok::{Oklab, Oklch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Serialize for Srgb {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Srgb {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Srgb::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LinearRgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Error, PartialEq)]
#[error("invalid hex color `{input}`: expected `#rrggbb` or `rrggbb`")]
pub struct HexParseError {
    pub input: String,
}

impl Srgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Srgb = Srgb::new(0, 0, 0);
    pub const WHITE: Srgb = Srgb::new(255, 255, 255);

    pub fn from_hex(hex: &str) -> Result<Self, HexParseError> {
        let s = hex.strip_prefix('#').unwrap_or(hex);
        if s.len() != 6 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(HexParseError {
                input: hex.to_string(),
            });
        }
        let byte = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).expect("checked hex digits");
        Ok(Srgb::new(byte(0), byte(2), byte(4)))
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn encoded(self) -> LinearRgb {
        LinearRgb {
            r: self.r as f64 / 255.0,
            g: self.g as f64 / 255.0,
            b: self.b as f64 / 255.0,
        }
    }

    pub fn to_linear(self) -> LinearRgb {
        fn decode(c: f64) -> f64 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        let e = self.encoded();
        LinearRgb {
            r: decode(e.r),
            g: decode(e.g),
            b: decode(e.b),
        }
    }

    pub fn from_linear_clamped(lin: LinearRgb) -> Self {
        fn encode(c: f64) -> u8 {
            let c = c.clamp(0.0, 1.0);
            let e = if c <= 0.0031308 {
                12.92 * c
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            };
            (e * 255.0 + 0.5).floor().min(255.0) as u8
        }
        Srgb::new(encode(lin.r), encode(lin.g), encode(lin.b))
    }

    pub fn to_oklab(self) -> Oklab {
        Oklab::from_srgb(self)
    }

    pub fn to_oklch(self) -> Oklch {
        self.to_oklab().to_oklch()
    }

    pub fn relative_luminance(self) -> f64 {
        let l = self.to_linear();
        0.2126 * l.r + 0.7152 * l.g + 0.0722 * l.b
    }
}

impl fmt::Display for Srgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

pub fn contrast_ratio(a: Srgb, b: Srgb) -> f64 {
    let la = a.relative_luminance();
    let lb = b.relative_luminance();
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastVerdict {
    Aaa,
    Aa,
    LargeTextOnly,
    Fail,
}

impl ContrastVerdict {
    pub fn from_ratio(ratio: f64) -> Self {
        match ratio {
            r if r >= 7.0 => ContrastVerdict::Aaa,
            r if r >= 4.5 => ContrastVerdict::Aa,
            r if r >= 3.0 => ContrastVerdict::LargeTextOnly,
            _ => ContrastVerdict::Fail,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ContrastVerdict::Aaa => "AAA",
            ContrastVerdict::Aa => "AA",
            ContrastVerdict::LargeTextOnly => "large-text-only",
            ContrastVerdict::Fail => "fail",
        }
    }
}

pub fn contrast_verdict(a: Srgb, b: Srgb) -> (f64, ContrastVerdict) {
    let ratio = contrast_ratio(a, b);
    (ratio, ContrastVerdict::from_ratio(ratio))
}

impl LinearRgb {
    pub fn in_gamut(self, eps: f64) -> bool {
        self.r >= -eps
            && self.r <= 1.0 + eps
            && self.g >= -eps
            && self.g <= 1.0 + eps
            && self.b >= -eps
            && self.b <= 1.0 + eps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        for hex in [
            "#002b36", "#839496", "#ff79c6", "#000000", "#ffffff", "123abc",
        ] {
            let c = Srgb::from_hex(hex).unwrap();
            assert_eq!(c.to_hex(), format!("#{}", hex.trim_start_matches('#')));
        }
    }

    #[test]
    fn hex_rejects_garbage() {
        for bad in ["", "#", "#fff", "#gggggg", "12345", "#1234567", "zzzzzz"] {
            assert!(Srgb::from_hex(bad).is_err(), "{bad} should not parse");
        }
    }

    #[test]
    fn wcag_known_values() {
        assert!((contrast_ratio(Srgb::WHITE, Srgb::BLACK) - 21.0).abs() < 1e-9);
        assert!(
            (contrast_ratio(Srgb::new(0x28, 0x2a, 0x36), Srgb::new(0x28, 0x2a, 0x36)) - 1.0).abs()
                < 1e-9
        );
        let ratio = contrast_ratio(Srgb::from_hex("#767676").unwrap(), Srgb::WHITE);
        assert!((ratio - 4.54).abs() < 0.01, "got {ratio}");
        assert_eq!(ContrastVerdict::from_ratio(ratio), ContrastVerdict::Aa);
        let ratio = contrast_ratio(Srgb::from_hex("#777777").unwrap(), Srgb::WHITE);
        assert!(ratio < 4.5, "got {ratio}");
        assert_eq!(
            ContrastVerdict::from_ratio(ratio),
            ContrastVerdict::LargeTextOnly
        );
        let a = Srgb::from_hex("#839496").unwrap();
        let b = Srgb::from_hex("#002b36").unwrap();
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-12);
    }

    #[test]
    fn relative_luminance_extremes() {
        assert!(Srgb::WHITE.relative_luminance() > 0.999999);
        assert!(Srgb::BLACK.relative_luminance() < 1e-9);
    }

    #[test]
    fn verdict_boundaries() {
        assert_eq!(ContrastVerdict::from_ratio(7.0), ContrastVerdict::Aaa);
        assert_eq!(ContrastVerdict::from_ratio(6.99), ContrastVerdict::Aa);
        assert_eq!(ContrastVerdict::from_ratio(4.5), ContrastVerdict::Aa);
        assert_eq!(
            ContrastVerdict::from_ratio(4.499),
            ContrastVerdict::LargeTextOnly
        );
        assert_eq!(
            ContrastVerdict::from_ratio(3.0),
            ContrastVerdict::LargeTextOnly
        );
        assert_eq!(ContrastVerdict::from_ratio(2.99), ContrastVerdict::Fail);
    }
}
