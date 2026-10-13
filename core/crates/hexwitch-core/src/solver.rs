use crate::color::{composite_over, contrast_ratio, gamut_map, Oklch, Srgb};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Both,
    Lighter,
    Darker,
}
#[derive(Debug, Clone, Serialize)]
pub struct SolveResult {
    pub color: Srgb,
    pub achieved: f64,
    pub delta_l: f64,
    pub tolerance: f64,
}
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}; best achievable contrast {achieved:.4}:1")]
pub struct SolveError {
    pub message: String,
    pub best: Srgb,
    pub achieved: f64,
    pub policy_restricted: bool,
}

pub fn solve_contrast(
    fg: Srgb,
    backgrounds: &[Srgb],
    min: f64,
    direction: Direction,
    alpha: f64,
) -> Result<SolveResult, SolveError> {
    let score = |c: Srgb| {
        backgrounds
            .iter()
            .map(|bg| contrast_ratio(composite_over(c, alpha, *bg), *bg))
            .fold(21.0, f64::min)
    };
    if backgrounds.is_empty()
        || backgrounds.len() > 64
        || !min.is_finite()
        || !(1.0..=21.0).contains(&min)
        || !alpha.is_finite()
        || !(0.0..=1.0).contains(&alpha)
    {
        return Err(SolveError {
            message: "invalid solver parameters".into(),
            best: fg,
            achieved: 0.0,
            policy_restricted: false,
        });
    }
    let start = fg.to_oklch();
    let result = |c: Srgb| SolveResult {
        color: c,
        achieved: score(c),
        delta_l: c.to_oklch().l - start.l,
        tolerance: 0.000001,
    };
    if score(fg) >= min {
        return Ok(result(fg));
    }
    let at = |l: f64| gamut_map(Oklch::new(l, start.c, start.h));
    let mut best = fg;
    let mut best_score = score(fg);
    let mut solutions = vec![];
    for sign in [-1.0, 1.0] {
        if (sign < 0.0 && direction == Direction::Lighter)
            || (sign > 0.0 && direction == Direction::Darker)
        {
            continue;
        }
        let bound = if sign < 0.0 { start.l } else { 1.0 - start.l };
        let mut previous = 0.0;
        for step in 1..=4096 {
            let delta = bound * step as f64 / 4096.0;
            let candidate = at(start.l + sign * delta);
            let achieved = score(candidate);
            if achieved > best_score {
                best = candidate;
                best_score = achieved;
            }
            if achieved >= min {
                let (mut low, mut high) = (previous, delta);
                let mut passing = candidate;
                for _ in 0..24 {
                    let mid = (low + high) / 2.0;
                    let c = at(start.l + sign * mid);
                    if score(c) >= min {
                        high = mid;
                        passing = c;
                    } else {
                        low = mid;
                    }
                }
                solutions.push(result(passing));
                break;
            }
            previous = delta;
        }
    }
    solutions.sort_by(|a, b| {
        a.delta_l
            .abs()
            .total_cmp(&b.delta_l.abs())
            .then(a.color.to_hex().cmp(&b.color.to_hex()))
    });
    if let Some(solution) = solutions.into_iter().next() {
        return Ok(solution);
    }
    let policy_restricted = direction != Direction::Both
        && solve_contrast(fg, backgrounds, min, Direction::Both, alpha).is_ok();
    Err(SolveError {
        message: if policy_restricted {
            "contrast reachable only outside permitted direction"
        } else {
            "contrast infeasible within fixed-hue lightness search"
        }
        .into(),
        best,
        achieved: best_score,
        policy_restricted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn searches_opposite_direction_when_needed() {
        let fg = Srgb::new(130, 130, 130);
        let bg = Srgb::new(128, 128, 128);
        let solved = solve_contrast(fg, &[bg], 4.5, Direction::Both, 1.0).unwrap();
        assert!(solved.delta_l < 0.0);
        assert!(contrast_ratio(solved.color, bg) >= 4.5);
        let error = solve_contrast(fg, &[bg], 4.5, Direction::Lighter, 1.0).unwrap_err();
        assert!(error.policy_restricted && error.achieved > 1.0);
    }
    #[test]
    fn reports_nonzero_best_and_alpha_infeasibility() {
        let error = solve_contrast(
            Srgb::new(128, 128, 128),
            &[Srgb::BLACK, Srgb::WHITE],
            7.0,
            Direction::Both,
            1.0,
        )
        .unwrap_err();
        assert!(error.achieved > 4.0 && !error.policy_restricted);
        let error =
            solve_contrast(Srgb::WHITE, &[Srgb::BLACK], 4.5, Direction::Both, 0.0).unwrap_err();
        assert_eq!(error.achieved, 1.0);
    }
}
