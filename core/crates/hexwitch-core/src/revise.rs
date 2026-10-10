//! Revise: apply an edit spec to a theme with minimal diff and maximal
//! identity preservation.
//!
//! An [`EditSpec`] is the structured, reviewable output the interpretation
//! layer (LLM bridge, M3) must produce — the engine never guesses intent.
//! The revision contract:
//!
//! * **Minimal diff.** Roles not named by the spec stay bit-identical.
//! * **Identity preservation.** Color work happens in OKLCH: lightness
//!   edits keep hue and chroma, contrast solves keep hue. Gamut mapping is
//!   explicit, never per-channel clipping.
//! * **Protection wins.** Roles in `protect` (and any role with `Locked`
//!   provenance) refuse changes; refusals are reported, never silently
//!   applied.
//! * **Order matters.** `changes` apply sequentially; a contrast solve
//!   against `surface.base` sees the *revised* base if it was changed
//!   earlier in the list.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use thiserror::Error;

use crate::color::{contrast_ratio, gamut_map, Oklch, Srgb};
use crate::ir::role::Role;
use crate::ir::style::Provenance;
use crate::ir::theme::Theme;

#[derive(Debug, Error, PartialEq)]
pub enum ReviseError {
    #[error("unknown role in `protect`: {0}")]
    UnknownProtectRole(String),
    #[error("unknown role in change target `{target}`: {role}")]
    UnknownTargetRole { target: String, role: String },
    #[error("invalid color in `against`: {0}")]
    BadAgainstColor(String),
}

/// What a contrast solve is measured against: another role's background,
/// or a literal hex color.
#[derive(Debug, Clone, PartialEq)]
pub enum Against {
    Role(String),
    Hex(Srgb),
}

impl<'de> Deserialize<'de> for Against {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match Against::parse(&s) {
            Some(a) => Ok(a),
            None => Err(serde::de::Error::custom(format!(
                "neither a role nor a #rrggbb color: {s}"
            ))),
        }
    }
}

impl Serialize for Against {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Against::Role(r) => serializer.serialize_str(r),
            Against::Hex(c) => serializer.serialize_str(&c.to_hex()),
        }
    }
}

impl Against {
    pub fn parse(s: &str) -> Option<Self> {
        if s.starts_with('#') {
            Srgb::from_hex(s).ok().map(Against::Hex)
        } else {
            s.parse::<Role>().ok().map(|r| Against::Role(r.to_string()))
        }
    }
}

/// A single operation, applied in OKLCH unless noted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Op {
    /// Raise OKLCH lightness by `amount` (0-1).
    Lighter { amount: f64 },
    /// Lower OKLCH lightness by `amount` (0-1).
    Darker { amount: f64 },
    /// Raise OKLCH chroma by `amount` (capped by gamut mapping).
    Saturate { amount: f64 },
    /// Lower OKLCH chroma by `amount`.
    Desaturate { amount: f64 },
    /// Rotate hue by `shift` degrees.
    Hue { shift: f64 },
    /// Solve for the smallest lightness change (hue/chroma preserved)
    /// that reaches at least `min` WCAG contrast against `against`.
    MinContrast { against: Against, min: f64 },
    /// Mark as protected; no color change. (Equivalent to listing the role
    /// in `protect`.)
    Protect,
}

/// One change: an operation applied to every role matching `target`.
/// Targets are exact role strings (`syntax.comment`) or a trailing
/// wildcard (`surface.*`, `*`). Serializes flat:
/// `{"target": "syntax.comment", "op": "min_contrast", ...}`.
///
/// Note: `Change` itself cannot `deny_unknown_fields` (incompatible with
/// `flatten`), but unknown keys still fail inside `Op`'s own strict
/// deserialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub target: String,
    #[serde(flatten)]
    pub op: Op,
}

/// The edit spec: the only way colors change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditSpec {
    /// Roles that must not change. Combined with `Locked` provenance.
    #[serde(default)]
    pub protect: Vec<String>,
    #[serde(default)]
    pub changes: Vec<Change>,
}

impl EditSpec {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// One applied color move.
#[derive(Debug, Clone, Serialize)]
pub struct Applied {
    pub target: String,
    pub role: String,
    /// Which slot moved: `fg` or `bg`.
    pub slot: &'static str,
    pub op: String,
    pub before: String,
    pub after: String,
    pub delta_e_ok: f64,
    /// Extra context (e.g. achieved contrast).
    pub note: Option<String>,
}

/// A change that was refused — protection always wins.
#[derive(Debug, Clone, Serialize)]
pub struct Refused {
    pub target: String,
    pub role: String,
    pub reason: String,
}

/// A change that could not be satisfied; the original color stands.
#[derive(Debug, Clone, Serialize)]
pub struct Failed {
    pub target: String,
    pub role: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RevisionReport {
    pub applied: Vec<Applied>,
    pub refused: Vec<Refused>,
    pub failed: Vec<Failed>,
    /// Roles checked and left bit-identical (explicitly or by omission).
    pub untouched: usize,
}

impl RevisionReport {
    pub fn changed_roles(&self) -> usize {
        self.applied.len()
    }

    pub fn summary(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "revision: {} moved, {} refused, {} failed, {} untouched",
            self.applied.len(),
            self.refused.len(),
            self.failed.len(),
            self.untouched
        );
        for a in &self.applied {
            let _ = writeln!(
                s,
                "  moved   {:<28} {} {} -> {} (dE={:.3})",
                a.role, a.slot, a.before, a.after, a.delta_e_ok
            );
        }
        for r in &self.refused {
            let _ = writeln!(s, "  refused {:<28} {}", r.role, r.reason);
        }
        for f in &self.failed {
            let _ = writeln!(s, "  failed  {:<28} {}", f.role, f.reason);
        }
        s
    }
}

fn matches(target: &str, role: &Role) -> bool {
    if target == "*" {
        return true;
    }
    if let Some(prefix) = target.strip_suffix(".*") {
        role.to_string().starts_with(&format!("{prefix}."))
    } else {
        target == role.to_string()
    }
}

/// True for dotted names in a known role namespace (surface. syntax. ...),
/// used to separate "typo inside a namespace" from free-form no-ops.
fn has_known_namespace(target: &str) -> bool {
    [
        "surface.",
        "text.",
        "syntax.",
        "interaction.",
        "diagnostic.",
        "diff.",
        "terminal.",
    ]
    .iter()
    .any(|ns| target.starts_with(ns))
}

fn op_name(op: &Op) -> String {
    serde_json::to_value(op)
        .ok()
        .and_then(|v| v.get("op").cloned())
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "op".into())
}

/// Move a color's lightness by `delta`, preserving hue and chroma, through
/// explicit gamut mapping.
fn shift_lightness(color: Srgb, delta: f64) -> Srgb {
    let ok = color.to_oklch();
    gamut_map(Oklch::new((ok.l + delta).clamp(0.0, 1.0), ok.c, ok.h))
}

fn shift_chroma(color: Srgb, delta: f64) -> Srgb {
    let ok = color.to_oklch();
    gamut_map(Oklch::new(ok.l, (ok.c + delta).max(0.0), ok.h))
}

fn shift_hue(color: Srgb, degrees: f64) -> Srgb {
    let ok = color.to_oklch();
    gamut_map(Oklch::new(ok.l, ok.c, ok.h + degrees))
}

/// Solve for minimal lightness movement reaching `min` contrast.
/// Hue and chroma are preserved; quantization is accounted for by
/// verifying the *rounded* color and nudging if the search lands on a
/// rounding edge.
fn solve_min_contrast(fg: Srgb, backdrop: Srgb, min: f64) -> Result<Srgb, String> {
    if contrast_ratio(fg, backdrop) >= min {
        return Ok(fg);
    }
    let start = fg.to_oklch();
    let brighten = fg.relative_luminance() >= backdrop.relative_luminance();

    let try_l = |l: f64| -> Option<Srgb> {
        let candidate = gamut_map(Oklch::new(l.clamp(0.0, 1.0), start.c, start.h));
        (contrast_ratio(candidate, backdrop) >= min).then_some(candidate)
    };

    let limit = if brighten { 1.0 } else { 0.0 };
    let bound = if brighten { 1.0 - start.l } else { start.l };
    if bound <= 0.0 {
        return Err("already at the lightness limit".into());
    }

    // Coarse scan (0.01) to find the crossing region, then refine. A scan
    // avoids assuming perfect monotonicity under 8-bit quantization.
    let steps = (bound / 0.01).ceil() as usize;
    let mut found: Option<(f64, Srgb)> = None;
    for step in 1..=steps {
        let delta = bound * (step as f64) / (steps.max(1) as f64);
        let l = if brighten {
            start.l + delta
        } else {
            start.l - delta
        };
        if let Some(c) = try_l(l) {
            found = Some((l, c));
            break;
        }
    }
    match found {
        Some((_, c)) => Ok(c),
        None => {
            let extreme = try_l(limit)
                .map(|c| contrast_ratio(c, backdrop))
                .unwrap_or(0.0);
            Err(format!(
                "unreachable: best achievable contrast at this hue is about {extreme:.2}:1 (< {min})"
            ))
        }
    }
}

fn describe_op(op: &Op) -> String {
    match op {
        Op::Lighter { amount } | Op::Darker { amount } => format!("L by {amount}"),
        Op::Saturate { amount } | Op::Desaturate { amount } => format!("C by {amount}"),
        Op::Hue { shift } => format!("H by {shift}"),
        Op::MinContrast { .. } => "contrast solve".into(),
        Op::Protect => "protect".into(),
    }
}

/// Apply an edit spec, returning the revised theme and an honest report.
/// The input theme is never mutated.
pub fn revise(theme: &Theme, spec: &EditSpec) -> Result<(Theme, RevisionReport), ReviseError> {
    let mut revised = theme.clone();
    let mut report = RevisionReport::default();

    // ---- protection ----
    let mut protected: Vec<Role> = Vec::new();
    // Locked provenance from the IR is always protection.
    for (role, style) in &revised.styles {
        if style.provenance == Provenance::Locked {
            protected.push(role.clone());
        }
    }
    for target in &spec.protect {
        let mut any = false;
        for role in revised.styles.keys() {
            if matches(target, role) {
                any = true;
                if !protected.contains(role) {
                    protected.push(role.clone());
                }
            }
        }
        // Wildcards matching nothing are fine; an exact, well-formed role
        // name matching nothing is almost certainly a typo (likely from an
        // LLM-written spec) and deserves a hard error.
        if !any && !target.ends_with(".*") && target != "*" && target.parse::<Role>().is_ok() {
            return Err(ReviseError::UnknownProtectRole(target.clone()));
        }
    }
    for role in &protected {
        if let Some(style) = revised.styles.get_mut(role) {
            style.provenance = Provenance::Locked;
        }
    }

    // ---- changes ----
    for change in &spec.changes {
        let mut targets: Vec<Role> = revised
            .styles
            .keys()
            .filter(|r| matches(&change.target, r))
            .cloned()
            .collect();
        if targets.is_empty() {
            // Distinguish failure modes: a namespaced but unparseable
            // target ("syntax.nonexistent") is a typo and a hard error; a
            // bare word or wildcard matching nothing is a recorded no-op.
            if has_known_namespace(&change.target) {
                return Err(ReviseError::UnknownTargetRole {
                    target: change.target.clone(),
                    role: change.target.clone(),
                });
            }
            report.failed.push(Failed {
                target: change.target.clone(),
                role: "(no matching roles)".into(),
                reason: "target matched nothing".into(),
            });
            continue;
        }
        targets.sort();

        for role in targets {
            if let Some(style) = revised.styles.get(&role) {
                if protected.contains(&role) && !matches!(change.op, Op::Protect) {
                    report.refused.push(Refused {
                        target: change.target.clone(),
                        role: role.to_string(),
                        reason: "role is protected".into(),
                    });
                    continue;
                }
                if style.provenance == Provenance::Locked && !matches!(change.op, Op::Protect) {
                    report.refused.push(Refused {
                        target: change.target.clone(),
                        role: role.to_string(),
                        reason: "role has locked provenance".into(),
                    });
                    continue;
                }
            }

            if matches!(change.op, Op::Protect) {
                if let Some(style) = revised.styles.get_mut(&role) {
                    style.provenance = Provenance::Locked;
                }
                continue;
            }

            // Resolve the backdrop for contrast solves before borrowing.
            let backdrop = match &change.op {
                Op::MinContrast { against, .. } => match against {
                    Against::Role(r) => {
                        let role_key: Role = r
                            .parse()
                            .map_err(|_| ReviseError::BadAgainstColor(r.clone()))?;
                        match revised.bg(&role_key).or_else(|| revised.fg(&role_key)) {
                            Some(c) => Some(c),
                            None => {
                                report.failed.push(Failed {
                                    target: change.target.clone(),
                                    role: role.to_string(),
                                    reason: format!("`against` role {r} has no color"),
                                });
                                continue;
                            }
                        }
                    }
                    Against::Hex(c) => Some(*c),
                },
                _ => None,
            };

            let slots: [(&'static str, Option<Srgb>); 2] = {
                let style = revised.styles.get(&role).unwrap();
                [("fg", style.fg), ("bg", style.bg)]
            };
            let mut changed = false;

            for (slot, current) in slots {
                let Some(color) = current else { continue };
                let new_color = match (&change.op, backdrop) {
                    (Op::Lighter { amount }, _) => Some(shift_lightness(color, *amount)),
                    (Op::Darker { amount }, _) => Some(shift_lightness(color, -*amount)),
                    (Op::Saturate { amount }, _) => Some(shift_chroma(color, *amount)),
                    (Op::Desaturate { amount }, _) => Some(shift_chroma(color, -*amount)),
                    (Op::Hue { shift }, _) => Some(shift_hue(color, *shift)),
                    (Op::MinContrast { min, .. }, Some(backdrop)) => {
                        match solve_min_contrast(color, backdrop, *min) {
                            Ok(c) => Some(c),
                            Err(reason) => {
                                report.failed.push(Failed {
                                    target: change.target.clone(),
                                    role: role.to_string(),
                                    reason,
                                });
                                continue;
                            }
                        }
                    }
                    (Op::MinContrast { .. }, None) => unreachable!("backdrop resolved above"),
                    (Op::Protect, _) => None,
                };
                if let Some(new_color) = new_color {
                    if new_color != color {
                        changed = true;
                        let note = match &change.op {
                            Op::MinContrast { min, .. } => Some(format!(
                                "contrast now {:.2}:1 (target {min})",
                                contrast_ratio(new_color, backdrop.unwrap())
                            )),
                            _ => Some(describe_op(&change.op)),
                        };
                        let before_lab = color.to_oklab();
                        let after_lab = new_color.to_oklab();
                        report.applied.push(Applied {
                            target: change.target.clone(),
                            role: role.to_string(),
                            slot,
                            op: op_name(&change.op),
                            before: color.to_hex(),
                            after: new_color.to_hex(),
                            delta_e_ok: before_lab.distance(after_lab),
                            note,
                        });
                    }
                    let style = revised.styles.get_mut(&role).unwrap();
                    if slot == "fg" {
                        style.fg = Some(new_color);
                    } else {
                        style.bg = Some(new_color);
                    }
                }
            }
            if changed {
                if let Some(style) = revised.styles.get_mut(&role) {
                    style.provenance = Provenance::Generated;
                }
            }
        }
    }

    // ---- bookkeeping ----
    let changed: std::collections::HashSet<String> = report
        .applied
        .iter()
        .map(|a| a.role.clone())
        .chain(report.failed.iter().map(|f| f.role.clone()))
        .collect();
    report.untouched = revised
        .styles
        .keys()
        .filter(|r| !changed.contains(&r.to_string()))
        .count();

    revised.meta.source = Some(format!(
        "revised from {}",
        theme.meta.source.as_deref().unwrap_or(&theme.meta.name)
    ));

    Ok((revised, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::role::{Surface, SyntaxRole, TextRole};
    use crate::ir::style::Style;

    fn nordish() -> Theme {
        // Nord numbers: base/text/comment contrast 9.25 / 2.44.
        let mut t = Theme::new("nordish", crate::ir::theme::Appearance::Dark);
        t.set(
            Role::Surface(Surface::Base),
            Style::bg(Srgb::from_hex("#2e3440").unwrap()),
        );
        t.set(
            Role::Surface(Surface::Panel),
            Style::bg(Srgb::from_hex("#3b4252").unwrap()),
        );
        t.set(
            Role::Text(TextRole::Primary),
            Style::fg(Srgb::from_hex("#d8dee9").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Comment),
            Style::fg(Srgb::from_hex("#616e88").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Keyword),
            Style::fg(Srgb::from_hex("#81a1c1").unwrap()),
        );
        t
    }

    fn spec(json: &str) -> EditSpec {
        EditSpec::from_json(json).unwrap()
    }

    fn apply(t: &Theme, s: &EditSpec) -> (Theme, RevisionReport) {
        revise(t, s).unwrap()
    }

    #[test]
    fn unmentioned_roles_stay_bit_identical() {
        let (out, report) = apply(
            &nordish(),
            &spec(
                r##"{ "changes": [ { "target": "surface.base", "op": "darker", "amount": 0.02 } ] }"##,
            ),
        );
        assert_eq!(
            out.fg(&Role::Syntax(SyntaxRole::Keyword)),
            nordish().fg(&Role::Syntax(SyntaxRole::Keyword))
        );
        assert_eq!(report.changed_roles(), 1);
        assert_eq!(report.untouched, 4);
        assert!(out.base_bg().unwrap() != nordish().base_bg().unwrap());
    }

    #[test]
    fn protect_blocks_changes() {
        let t = nordish();
        let s = spec(
            r##"{ "protect": ["syntax.comment"],
                  "changes": [ { "target": "syntax.comment", "op": "lighter", "amount": 0.1 } ] }"##,
        );
        let (out, report) = apply(&t, &s);
        assert_eq!(
            out.fg(&Role::Syntax(SyntaxRole::Comment)),
            t.fg(&Role::Syntax(SyntaxRole::Comment))
        );
        assert_eq!(report.applied.len(), 0);
        assert_eq!(report.refused.len(), 1);
        assert!(report.refused[0].reason.contains("protected"));
        // Protection is recorded as Locked provenance for future rounds.
        assert_eq!(
            out.get(&Role::Syntax(SyntaxRole::Comment))
                .unwrap()
                .provenance,
            Provenance::Locked
        );
    }

    #[test]
    fn protection_is_idempotent_across_rounds() {
        let t = nordish();
        let spec = spec(r##"{"protect":["syntax.keyword","syntax.keyword"]}"##);
        let (first, _) = revise(&t, &spec).unwrap();
        let (second, _) = revise(&first, &spec).unwrap();
        assert_eq!(first.styles, second.styles);
        assert_eq!(
            second
                .get(&Role::Syntax(SyntaxRole::Keyword))
                .unwrap()
                .provenance,
            Provenance::Locked
        );
    }

    #[test]
    fn locked_provenance_refuses_without_being_listed() {
        let mut t = nordish();
        t.set(
            Role::Syntax(SyntaxRole::Keyword),
            Style::fg(Srgb::from_hex("#81a1c1").unwrap()).with_provenance(Provenance::Locked),
        );
        let (out, report) = apply(
            &t,
            &spec(r##"{ "changes": [ { "target": "*", "op": "saturate", "amount": 0.02 } ] }"##),
        );
        assert_eq!(
            out.fg(&Role::Syntax(SyntaxRole::Keyword)),
            t.fg(&Role::Syntax(SyntaxRole::Keyword))
        );
        assert!(report.refused.iter().any(|r| r.role == "syntax.keyword"));
    }

    #[test]
    fn min_contrast_solves_with_minimal_lightness_and_kept_hue() {
        let t = nordish();
        let base = t.base_bg().unwrap();
        let comment = t.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap();
        let start_ratio = contrast_ratio(comment, base);
        assert!(start_ratio < 4.5);

        let (out, report) = apply(
            &t,
            &spec(
                r##"{ "changes": [ { "target": "syntax.comment",
                     "op": "min_contrast", "against": "surface.base", "min": 4.5 } ] }"##,
            ),
        );
        let solved = out.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap();
        let ratio = contrast_ratio(solved, base);
        assert!(ratio >= 4.5, "got {ratio}");
        // Minimal movement: one scan step below the solution must fail —
        // that's what "smallest lightness change" means.
        let (l0, l1) = (comment.to_oklch().l, solved.to_oklch().l);
        assert!(l1 - l0 < 0.2, "moved L {l0} -> {l1}");
        let c0 = comment.to_oklch();
        let below = gamut_map(Oklch::new(l1 - 0.015, c0.c, c0.h));
        assert!(
            contrast_ratio(below, base) < 4.5,
            "solution was not minimal"
        );
        // Identity: hue preserved, chroma nearly so.
        let (c0, c1) = (comment.to_oklch(), solved.to_oklch());
        assert!((c0.h - c1.h).abs() < 0.5, "hue {} -> {}", c0.h, c1.h);
        assert!((c0.c - c1.c).abs() < 0.02);
        assert!(report.applied[0]
            .note
            .as_deref()
            .unwrap()
            .contains("contrast"));
    }

    #[test]
    fn min_contrast_failure_is_honest() {
        let t = nordish();
        let (out, report) = apply(
            &t,
            &spec(
                r##"{ "changes": [ { "target": "text.primary",
                     "op": "min_contrast", "against": "surface.base", "min": 25.0 } ] }"##,
            ),
        );
        assert_eq!(
            out.fg(&Role::Text(TextRole::Primary)),
            t.fg(&Role::Text(TextRole::Primary))
        );
        assert_eq!(report.failed.len(), 1);
        assert!(report.failed[0].reason.contains("unreachable"));
    }

    #[test]
    fn wildcard_targets_surfaces() {
        let (out, report) = apply(
            &nordish(),
            &spec(
                r##"{ "changes": [ { "target": "surface.*", "op": "darker", "amount": 0.02 } ] }"##,
            ),
        );
        assert_ne!(out.base_bg(), nordish().base_bg());
        assert_ne!(
            out.bg(&Role::Surface(Surface::Panel)),
            nordish().bg(&Role::Surface(Surface::Panel))
        );
        // Syntax untouched.
        assert_eq!(
            out.fg(&Role::Syntax(SyntaxRole::Keyword)),
            nordish().fg(&Role::Syntax(SyntaxRole::Keyword))
        );
        assert_eq!(report.changed_roles(), 2);
    }

    #[test]
    fn revised_base_is_used_by_later_contrast_solves() {
        let (out, _) = apply(
            &nordish(),
            &spec(
                r##"{ "changes": [
                    { "target": "surface.base", "op": "darker", "amount": 0.05 },
                    { "target": "syntax.comment", "op": "min_contrast", "against": "surface.base", "min": 4.5 }
                ] }"##,
            ),
        );
        let ratio = contrast_ratio(
            out.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap(),
            out.base_bg().unwrap(),
        );
        assert!(ratio >= 4.5, "got {ratio}");
    }

    #[test]
    fn ops_preserve_identity_by_design() {
        let t = nordish();
        let keyword = t.fg(&Role::Syntax(SyntaxRole::Keyword)).unwrap();
        let (out, _) = apply(
            &t,
            &spec(
                r##"{ "changes": [ { "target": "syntax.keyword", "op": "hue", "shift": 15.0 } ] }"##,
            ),
        );
        let moved = out.fg(&Role::Syntax(SyntaxRole::Keyword)).unwrap();
        let (a, b) = (keyword.to_oklch(), moved.to_oklch());
        assert!((b.h - a.h - 15.0).abs() < 1.5, "hue {} -> {}", a.h, b.h);
        assert!((a.l - b.l).abs() < 0.01);
    }

    #[test]
    fn changed_roles_become_generated() {
        let (out, _) = apply(
            &nordish(),
            &spec(
                r##"{ "changes": [ { "target": "surface.base", "op": "darker", "amount": 0.01 } ] }"##,
            ),
        );
        assert_eq!(
            out.get(&Role::Surface(Surface::Base)).unwrap().provenance,
            Provenance::Generated
        );
        assert_eq!(
            out.get(&Role::Syntax(SyntaxRole::Keyword))
                .unwrap()
                .provenance,
            Provenance::Ingested
        );
    }

    #[test]
    fn unknown_target_roles_are_errors_not_noops() {
        let err = revise(
            &nordish(),
            &spec(r##"{ "changes": [ { "target": "syntax.nonexistent", "op": "darker", "amount": 0.1 } ] }"##),
        )
        .unwrap_err();
        assert!(matches!(err, ReviseError::UnknownTargetRole { .. }));
        // Typos outside the vocabulary are reported as failures instead.
        let (_, report) = apply(
            &nordish(),
            &spec(
                r##"{ "changes": [ { "target": "syntax-keyword-typo", "op": "darker", "amount": 0.1 } ] }"##,
            ),
        );
        assert_eq!(report.failed.len(), 1);
    }

    #[test]
    fn spec_rejects_unknown_fields() {
        assert!(EditSpec::from_json(r##"{ "vibe": "warmer" }"##).is_err());
        assert!(EditSpec::from_json(r##"{ "changes": [ { "target": "x", "op": "lighter", "amount": 0.1, "force": true } ] }"##).is_err());
    }

    #[test]
    fn report_serializes_for_cli() {
        let (_, report) = apply(
            &nordish(),
            &spec(
                r##"{ "changes": [ { "target": "surface.base", "op": "darker", "amount": 0.01 } ] }"##,
            ),
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"applied\""));
        assert!(report.summary().contains("moved"));
    }
}
