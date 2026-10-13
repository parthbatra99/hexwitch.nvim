use super::interpret_eval::{color, property, styles, Metric, Pair, PairResult, Turn};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn evaluate_pairs(
    pairs: &[Pair],
    turns: &BTreeMap<String, Turn>,
    case_ids: &BTreeSet<String>,
) -> Result<Vec<PairResult>, String> {
    if pairs.len() > 256 {
        return Err("pair count limit".into());
    }
    let mut ids = BTreeSet::new();
    let mut results = vec![];
    for pair in pairs {
        let (id, left, right) = match pair {
            Pair::Equivalent {
                id, left, right, ..
            } => (id, left, right),
            Pair::StrengthOrdered {
                id,
                smaller,
                larger,
                ..
            } => (id, smaller, larger),
        };
        if id.trim().is_empty()
            || !ids.insert(id.clone())
            || !case_ids.contains(left)
            || !case_ids.contains(right)
            || left == right
        {
            return Err("invalid pair identifier or case reference".into());
        }
        let mut findings = vec![];
        if let (Some(a), Some(b)) = (turns.get(left), turns.get(right)) {
            if !a.passed || !b.passed {
                findings.push("pair depends on a failed semantic case".into());
            }
            match pair {
                Pair::Equivalent { targets, .. } => {
                    if targets.is_empty() {
                        return Err("equivalence targets required".into());
                    }
                    if a.model.decision != b.model.decision {
                        findings.push("paraphrase decisions differ".into());
                    }
                    for target in targets {
                        let old_a = styles(&a.before, target)?;
                        let old_b = styles(&b.before, target)?;
                        if old_a.len() != old_b.len()
                            || old_a.iter().zip(&old_b).any(|((id, a), (other, b))| {
                                id != other
                                    || property(a, target.property) != property(b, target.property)
                            })
                        {
                            findings.push(
                                "equivalence pair did not start from equivalent target state"
                                    .into(),
                            );
                        }
                        let aa = styles(&a.after, target)?;
                        let bb = styles(&b.after, target)?;
                        if aa.len() != bb.len() {
                            findings.push("paired treatment inventories differ".into());
                            continue;
                        }
                        for ((id, a), (other, b)) in aa.iter().zip(&bb) {
                            if id != other {
                                findings.push("paired treatment identities differ".into());
                                continue;
                            }
                            if property(a, target.property) != property(b, target.property)
                                || a.is_locked(target.property) != b.is_locked(target.property)
                            {
                                findings.push(format!(
                                    "paired property differs {id} {:?}",
                                    target.property
                                ));
                            }
                        }
                    }
                }
                Pair::StrengthOrdered {
                    target,
                    metric,
                    min_gap,
                    ..
                } => {
                    if !min_gap.is_finite() || *min_gap < 0.0 {
                        return Err("invalid pair gap".into());
                    }
                    let ab = styles(&a.before, target)?;
                    let aa = styles(&a.after, target)?;
                    let bb = styles(&b.before, target)?;
                    let ba = styles(&b.after, target)?;
                    if ab.len() != bb.len() {
                        return Err("strength pair inventory mismatch".into());
                    }
                    for (((id, old_a), (_, new_a)), ((_, old_b), (_, new_b))) in
                        ab.iter().zip(&aa).zip(bb.iter().zip(&ba))
                    {
                        match (
                            color(old_a, target.property),
                            color(new_a, target.property),
                            color(old_b, target.property),
                            color(new_b, target.property),
                        ) {
                            (Some(oa), Some(na), Some(ob), Some(nb)) => {
                                if oa != ob {
                                    findings.push(format!(
                                        "strength pair starts from different color {id}"
                                    ));
                                    continue;
                                }
                                let delta =
                                    |x: crate::color::Srgb, y: crate::color::Srgb| match metric {
                                        Metric::Lightness => y.to_oklch().l - x.to_oklch().l,
                                        Metric::Chroma => y.to_oklch().c - x.to_oklch().c,
                                    };
                                let small = delta(oa, na);
                                let large = delta(ob, nb);
                                if small * large <= 0.0 || large.abs() < small.abs() + min_gap {
                                    findings.push(format!("strength ordering failed {id}: small {small:.4}, large {large:.4}"));
                                }
                            }
                            _ => findings.push(format!("strength pair needs concrete colors {id}")),
                        }
                    }
                }
            }
        } else {
            findings.push("pair dependency missing after parse/validation failure".into());
        }
        results.push(PairResult {
            id: id.clone(),
            passed: findings.is_empty(),
            findings,
        });
    }
    Ok(results)
}
