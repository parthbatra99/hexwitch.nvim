use crate::interpret_eval::{styles, Case, Metric, Pair, Predicate};
use crate::ir::{Property, Theme};
use std::collections::BTreeSet;

fn predicate(
    check: &Predicate,
    theme: &Theme,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), String> {
    *nodes += 1;
    if depth > 8 || *nodes > 1024 {
        return Err("evaluation predicate depth/count limit".into());
    }
    let target = match check {
        Predicate::AnyOf { alternatives } => {
            if alternatives.is_empty()
                || alternatives.len() > 8
                || alternatives.iter().any(Vec::is_empty)
            {
                return Err("invalid acceptable alternatives".into());
            }
            for alternative in alternatives {
                for check in alternative {
                    predicate(check, theme, depth + 1, nodes)?;
                }
            }
            return Ok(());
        }
        Predicate::Decision { allowed } => {
            if allowed.is_empty() {
                return Err("allowed decisions cannot be empty".into());
            }
            return Ok(());
        }
        Predicate::Outcome { allowed } => {
            if allowed.is_empty() {
                return Err("allowed outcomes cannot be empty".into());
            }
            return Ok(());
        }
        Predicate::OnlyChanged { allowed } | Predicate::OnlyLockChanges { allowed } => {
            if allowed.len() > 256 {
                return Err("allowed target count limit".into());
            }
            for target in allowed {
                styles(theme, target)?;
            }
            return Ok(());
        }
        Predicate::NoMutation => return Ok(()),
        Predicate::Preserved { target }
        | Predicate::State { target, .. }
        | Predicate::Bool { target, .. }
        | Predicate::Number { target, .. }
        | Predicate::Color { target, .. }
        | Predicate::Locked { target, .. }
        | Predicate::Lightness { target, .. }
        | Predicate::Chroma { target, .. }
        | Predicate::Contrast { target, .. }
        | Predicate::ContrastRequested { target, .. }
        | Predicate::HuePreserved { target, .. }
        | Predicate::Secondary { target, .. }
        | Predicate::ActionPresent { target, .. }
        | Predicate::ActionAbsent { target, .. } => target,
    };
    styles(theme, target)?;
    match check {
        Predicate::Bool { .. }
            if !matches!(
                target.property,
                Property::Bold | Property::Italic | Property::Strikethrough
            ) =>
        {
            return Err("bool expectation needs typography property".into())
        }
        Predicate::Number { value, .. } => {
            if !value.is_finite()
                || !matches!(
                    target.property,
                    Property::FgOpacity
                        | Property::BgOpacity
                        | Property::SpecialOpacity
                        | Property::Blend
                )
            {
                return Err("numeric expectation needs finite number and numeric property".into());
            }
        }
        Predicate::Lightness {
            min_delta,
            max_delta,
            ..
        }
        | Predicate::Chroma {
            min_delta,
            max_delta,
            ..
        } => {
            if !min_delta.is_finite() || !max_delta.is_finite() || min_delta > max_delta {
                return Err("invalid metric bounds".into());
            }
        }
        Predicate::Contrast {
            backgrounds, min, ..
        }
        | Predicate::ContrastRequested {
            backgrounds, min, ..
        } => {
            if !min.is_finite() || !(1.0..=21.0).contains(min) {
                return Err("invalid expected contrast".into());
            }
            if backgrounds.is_empty() || backgrounds.len() > 64 {
                return Err("expected background count limit".into());
            }
            for b in backgrounds {
                b.validate(theme)?;
            }
        }
        Predicate::HuePreserved {
            tolerance_degrees, ..
        } => {
            if !tolerance_degrees.is_finite() || !(0.0..=180.0).contains(tolerance_degrees) {
                return Err("invalid hue expectation".into());
            }
        }
        Predicate::Secondary {
            primary,
            background,
            min,
            ..
        } => {
            if !min.is_finite() || !(1.0..=21.0).contains(min) || theme.fg(primary).is_none() {
                return Err("invalid hierarchy primary or contrast".into());
            }
            background.validate(theme)?;
        }
        Predicate::ActionPresent { op, .. } | Predicate::ActionAbsent { op, .. }
            if !crate::interpret_v1::ACTIONS.contains(&op.as_str()) =>
        {
            return Err("unknown expected operation".into());
        }
        _ => {}
    }
    if matches!(
        check,
        Predicate::Color { .. }
            | Predicate::Lightness { .. }
            | Predicate::Chroma { .. }
            | Predicate::Contrast { .. }
            | Predicate::ContrastRequested { .. }
            | Predicate::HuePreserved { .. }
            | Predicate::Secondary { .. }
    ) && !matches!(
        target.property,
        Property::Fg | Property::Bg | Property::Special
    ) {
        return Err("color expectation needs color property".into());
    }
    Ok(())
}

pub(crate) fn validate_case(case: &Case, theme: &Theme) -> Result<(), String> {
    let mut nodes = 0;
    for check in &case.checks {
        predicate(check, theme, 0, &mut nodes)
            .map_err(|e| format!("invalid evaluation contract for {}: {e}", case.id))?;
    }
    Ok(())
}

pub(crate) fn validate_pairs(pairs: &[Pair], case_ids: &BTreeSet<String>) -> Result<(), String> {
    if pairs.len() > 256 {
        return Err("pair count limit".into());
    }
    let mut ids = BTreeSet::new();
    for pair in pairs {
        let (id, left, right) = match pair {
            Pair::Equivalent {
                id,
                left,
                right,
                targets,
            } => {
                if targets.is_empty()
                    || targets.len() > 256
                    || targets.iter().any(|t| {
                        t.roles.is_empty()
                            || t.roles.len() > 64
                            || t.roles.iter().any(|s| !crate::selector::valid_selector(s))
                    })
                {
                    return Err("pair targets cannot be empty".into());
                }
                (id, left, right)
            }
            Pair::StrengthOrdered {
                id,
                smaller,
                larger,
                min_gap,
                metric,
                target,
            } => {
                if !min_gap.is_finite()
                    || *min_gap < 0.0
                    || !matches!(metric, Metric::Lightness | Metric::Chroma)
                    || !matches!(
                        target.property,
                        Property::Fg | Property::Bg | Property::Special
                    )
                    || target.roles.is_empty()
                    || target.roles.len() > 64
                    || target
                        .roles
                        .iter()
                        .any(|s| !crate::selector::valid_selector(s))
                {
                    return Err("invalid pair metric".into());
                }
                (id, smaller, larger)
            }
        };
        if id.trim().is_empty()
            || !ids.insert(id.clone())
            || left == right
            || !case_ids.contains(left)
            || !case_ids.contains(right)
        {
            return Err("invalid pair case reference".into());
        }
    }
    Ok(())
}
