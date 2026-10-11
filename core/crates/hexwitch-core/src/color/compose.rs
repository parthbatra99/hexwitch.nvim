use crate::color::srgb::Srgb;

pub fn composite_over(src: Srgb, alpha: f64, backdrop: Srgb) -> Srgb {
    let alpha = alpha.clamp(0.0, 1.0);
    let mix = |s: u8, d: u8| -> u8 {
        let v = alpha * s as f64 + (1.0 - alpha) * d as f64;
        (v + 0.5).floor().clamp(0.0, 255.0) as u8
    };
    Srgb::new(
        mix(src.r, backdrop.r),
        mix(src.g, backdrop.g),
        mix(src.b, backdrop.b),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midpoints_round_as_expected() {
        let half = composite_over(Srgb::WHITE, 0.5, Srgb::BLACK);
        assert_eq!(half, Srgb::new(128, 128, 128));
    }

    #[test]
    fn alpha_extremes() {
        let src = Srgb::new(0x1e, 0x1e, 0x2e);
        let backdrop = Srgb::new(0xef, 0xf1, 0xf5);
        assert_eq!(composite_over(src, 0.0, backdrop), backdrop);
        assert_eq!(composite_over(src, 1.0, backdrop), src);
    }

    #[test]
    fn translucent_selection_can_break_contrast() {
        let text = Srgb::from_hex("#839496").unwrap();
        let sel = Srgb::from_hex("#073642").unwrap();
        let base = Srgb::from_hex("#002b36").unwrap();
        let composited = composite_over(sel, 0.85, base);
        assert!(composited != sel && composited != base);
        let with_sel = crate::color::contrast_ratio(text, composited);
        let without = crate::color::contrast_ratio(text, base);
        assert!(with_sel < without, "{with_sel} vs {without}");
    }
}
