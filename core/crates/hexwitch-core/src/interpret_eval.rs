//! Offline semantic-outcome evaluation. Recorded model outputs and annotations
//! are separate inputs; canned replay is contract evidence, not model quality.
use crate::backdrop::Backdrop;
use crate::color::{composite_over, contrast_ratio, Srgb};
use crate::intent::Target;
use crate::interpret_v1::{Decision, ModelInterpretation};
use crate::ir::{Property, Style, Theme};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub version: u32,
    pub origin: String,
    pub license: String,
    pub revision: String,
    pub purpose: String,
    pub limitations: Vec<String>,
    pub cases: Vec<Case>,
    #[serde(default)]
    pub pairs: Vec<Pair>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub split: String,
    pub category: String,
    pub request: String,
    pub theme: String,
    pub checks: Vec<Predicate>,
    #[serde(default)]
    pub context: crate::interpret_v1::IterationContext,
    /// Previous turn's actual result, not an independent fresh reference.
    #[serde(default)]
    pub previous: Option<String>,
    /// Explicit authored initial-state transformation, never model output.
    #[serde(default)]
    pub setup: Option<crate::intent::IntentSpec>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub expectation: Expectation,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expectation {
    #[default]
    Edit,
    Clarification,
    Unsupported,
    Refusal,
    Mixed,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case", deny_unknown_fields)]
pub enum Predicate {
    /// At least one complete alternative must pass; do not mix requirements
    /// from incompatible interpretations to manufacture a passing answer.
    AnyOf {
        alternatives: Vec<Vec<Predicate>>,
    },
    Outcome {
        allowed: Vec<crate::protocol::Outcome>,
    },
    OnlyLockChanges {
        allowed: Vec<Target>,
    },
    ActionAbsent {
        op: String,
        target: Target,
    },
    ActionPresent {
        op: String,
        target: Target,
        scope: Option<crate::intent::Scope>,
    },
    HuePreserved {
        target: Target,
        tolerance_degrees: f64,
    },
    Secondary {
        target: Target,
        primary: crate::ir::Role,
        background: Backdrop,
        min: f64,
    },
    State {
        target: Target,
        value: crate::ir::PropertyState,
    },
    Preserved {
        target: Target,
    },
    Bool {
        target: Target,
        value: bool,
    },
    Number {
        target: Target,
        value: f64,
    },
    Locked {
        target: Target,
        value: bool,
    },
    Color {
        target: Target,
        value: Srgb,
    },
    Lightness {
        target: Target,
        min_delta: f64,
        max_delta: f64,
    },
    Chroma {
        target: Target,
        min_delta: f64,
        max_delta: f64,
    },
    Contrast {
        target: Target,
        backgrounds: Vec<Backdrop>,
        min: f64,
    },
    /// A floor must be requested, not merely happen to hold in the starting
    /// theme. Checks all expected background contexts and lifetime.
    ContrastRequested {
        target: Target,
        backgrounds: Vec<Backdrop>,
        min: f64,
        persistent: bool,
    },
    OnlyChanged {
        allowed: Vec<Target>,
    },
    Decision {
        allowed: Vec<Decision>,
    },
    NoMutation,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case", deny_unknown_fields)]
pub enum Pair {
    Equivalent {
        id: String,
        left: String,
        right: String,
        targets: Vec<Target>,
    },
    StrengthOrdered {
        id: String,
        smaller: String,
        larger: String,
        target: Target,
        metric: Metric,
        min_gap: f64,
    },
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Lightness,
    Chroma,
}
#[derive(Debug, Serialize)]
pub struct PairResult {
    pub id: String,
    pub passed: bool,
    pub findings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub version: u32,
    pub label: String,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub kind: RunKind,
    pub responses: Vec<RecordedResponse>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    CannedReplay,
    RecordedProvider,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedResponse {
    pub case_id: String,
    pub response: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub split: String,
    pub category: String,
    pub passed: bool,
    pub failure_class: Option<String>,
    pub findings: Vec<String>,
    pub input_sha256: String,
    /// Includes resolved includes, mappings and protections, not just root bytes.
    pub resolved_theme_sha256: String,
    pub response_sha256: Option<String>,
    pub prompt_sha256: String,
    pub outcome: Option<crate::protocol::Outcome>,
    pub decision: Option<Decision>,
    pub expectation: Expectation,
    pub source_id: Option<String>,
    pub previous: Option<String>,
    pub context_sha256: String,
    pub checks: Vec<CheckResult>,
    /// A real prior result was evaluated but failed its semantic assertions.
    pub upstream_failed: bool,
}
#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub kind: &'static str,
    pub passed: bool,
    pub evaluated: bool,
    pub findings: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct CheckSummary {
    pub passed: usize,
    pub failed: usize,
    pub not_evaluated: usize,
}
#[derive(Debug, Serialize)]
pub struct SplitSummary {
    pub passed: usize,
    pub failed: usize,
}
#[derive(Debug, Serialize)]
pub struct Evaluation {
    pub version: u32,
    pub label: String,
    pub provider: String,
    pub model: String,
    pub kind: RunKind,
    pub prompt_version: String,
    pub engine_version: String,
    pub suite_sha256: String,
    pub run_sha256: String,
    pub passed: usize,
    pub failed: usize,
    pub cases: Vec<CaseResult>,
    pub failures_by_category: BTreeMap<String, usize>,
    pub failures_by_class: BTreeMap<String, usize>,
    pub results_by_split: BTreeMap<String, SplitSummary>,
    pub limitations: Vec<String>,
    pub pairs: Vec<PairResult>,
    pub pair_failures: usize,
    pub scorecard: BTreeMap<String, CheckSummary>,
    pub results_by_expectation: BTreeMap<String, SplitSummary>,
}

pub(crate) struct Turn {
    pub(crate) before: Theme,
    pub(crate) after: Theme,
    context: crate::interpret_v1::IterationContext,
    pub(crate) model: ModelInterpretation,
    pub(crate) passed: bool,
}

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut text = String::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() > 8 * 1024 * 1024 {
        return Err("evaluation input exceeds 8 MiB".into());
    }
    Ok(text)
}
pub(crate) fn styles<'a>(
    theme: &'a Theme,
    target: &Target,
) -> Result<Vec<(String, &'a Style)>, String> {
    if target.roles.is_empty() {
        return Err("empty evaluation target".into());
    }
    let mut keys = BTreeSet::new();
    for selector in &target.roles {
        if !crate::selector::valid_selector(selector) {
            return Err(format!("invalid evaluation selector {selector}"));
        }
        let matched: Vec<_> = theme
            .styles
            .keys()
            .filter(|r| crate::selector::matches(selector, r))
            .cloned()
            .collect();
        if matched.is_empty() {
            return Err(format!("evaluation target absent {selector}"));
        }
        keys.extend(matched);
    }
    let mut result = vec![];
    for role in keys {
        result.push((role.to_string(), theme.get(&role).unwrap()));
        if target.include_semantic {
            result.extend(
                theme
                    .modifiers
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.role == role)
                    .map(|(i, r)| (format!("{role}/semantic/{i}"), &r.style)),
            );
        }
    }
    Ok(result)
}
pub(crate) fn property(style: &Style, p: Property) -> serde_json::Value {
    let mut isolated = Style::new();
    isolated.copy_property_from(style, p);
    serde_json::to_value(isolated).unwrap()
}
pub(crate) fn color(style: &Style, p: Property) -> Option<Srgb> {
    match p {
        Property::Fg => style.fg,
        Property::Bg => style.bg,
        Property::Special => style.special,
        _ => None,
    }
}
fn bool_value(style: &Style, p: Property) -> Option<bool> {
    match p {
        Property::Bold => style.bold,
        Property::Italic => style.italic,
        Property::Strikethrough => style.strikethrough,
        _ => None,
    }
}
fn number(style: &Style, p: Property) -> Option<f64> {
    match p {
        Property::FgOpacity => Some(style.fg_opacity.unwrap_or(1.0)),
        Property::BgOpacity => Some(style.bg_opacity.unwrap_or(1.0)),
        Property::SpecialOpacity => Some(style.special_opacity.unwrap_or(1.0)),
        Property::Blend => style.blend.map(f64::from),
        _ => None,
    }
}

impl Predicate {
    fn kind(&self) -> &'static str {
        match self {
            Self::AnyOf { .. } => "acceptable_alternative",
            Self::Outcome { .. } => "reported_outcome",
            Self::OnlyLockChanges { .. }
            | Self::Locked { .. }
            | Self::ActionPresent { .. }
            | Self::ActionAbsent { .. } => "protection",
            Self::HuePreserved { .. } => "identity",
            Self::Secondary { .. } => "hierarchy",
            Self::Preserved { .. } => "preservation",
            Self::OnlyChanged { .. } => "unrelated_changes",
            Self::Lightness { .. } | Self::Chroma { .. } => "strength",
            Self::Contrast { .. } | Self::ContrastRequested { .. } => "readability",
            Self::Decision { .. } => "decision",
            Self::NoMutation => "no_mutation",
            _ => "property_correctness",
        }
    }
}

fn evaluate_predicate(
    check: &Predicate,
    before: &Theme,
    after: &Theme,
    model: &ModelInterpretation,
    outcome: crate::protocol::Outcome,
) -> Result<Vec<String>, String> {
    let mut findings = vec![];
    match check {
        Predicate::AnyOf { alternatives } => {
            if alternatives.is_empty()
                || alternatives.len() > 8
                || alternatives.iter().any(Vec::is_empty)
            {
                return Err("invalid acceptable alternatives".into());
            }
            let mut failures = vec![];
            for alternative in alternatives {
                let mut failed = vec![];
                for check in alternative {
                    failed.extend(evaluate_predicate(check, before, after, model, outcome)?);
                }
                if failed.is_empty() {
                    return Ok(vec![]);
                }
                failures.push(failed.join("; "));
            }
            findings.push(format!(
                "no acceptable alternative satisfied: {}",
                failures.join(" | ")
            ));
        }
        Predicate::Outcome { allowed } => {
            if allowed.is_empty() {
                return Err("empty allowed outcome list".into());
            }
            if !allowed.contains(&outcome) {
                findings.push(format!("reported outcome {outcome:?} not accepted"));
            }
        }
        Predicate::ContrastRequested {
            target,
            backgrounds,
            min,
            persistent,
        } => {
            let selected: BTreeSet<_> = styles(before, target)?
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            let mut covered = BTreeSet::new();
            for action in &model.actions {
                if action.op == "contrast"
                    && action.property == target.property
                    && action.min.is_some_and(|n| n >= *min)
                    && action.persistent == Some(*persistent)
                    && action.backgrounds.as_ref().is_some_and(|actual| {
                        backgrounds.iter().all(|expected| actual.contains(expected))
                    })
                {
                    let actual = Target {
                        roles: action.roles.clone(),
                        property: action.property,
                        include_semantic: action.include_semantic,
                    };
                    covered.extend(styles(before, &actual)?.into_iter().map(|(id, _)| id));
                }
            }
            if !selected.is_subset(&covered) {
                findings
                    .push("requested readability contexts or constraint lifetime omitted".into());
            }
        }
        Predicate::ActionAbsent { op, target } => {
            let forbidden: BTreeSet<_> = styles(before, target)?
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            for action in &model.actions {
                if action.op == *op && action.property == target.property {
                    let actual = Target {
                        roles: action.roles.clone(),
                        property: action.property,
                        include_semantic: action.include_semantic,
                    };
                    if styles(before, &actual)?
                        .iter()
                        .any(|(id, _)| forbidden.contains(id))
                    {
                        findings.push(format!(
                            "forbidden {op} operation proposed for requested property"
                        ));
                    }
                }
            }
        }
        Predicate::ActionPresent { op, target, scope } => {
            let selected = styles(before, target)?;
            let selected_ids: BTreeSet<_> = selected.iter().map(|(id, _)| id.clone()).collect();
            let mut covered = BTreeSet::new();
            for action in &model.actions {
                if action.op == *op
                    && action.property == target.property
                    && (scope.is_none() || action.scope == *scope)
                {
                    let action_target = Target {
                        roles: action.roles.clone(),
                        property: action.property,
                        include_semantic: action.include_semantic,
                    };
                    covered.extend(
                        styles(before, &action_target)?
                            .into_iter()
                            .map(|(id, _)| id),
                    );
                }
            }
            if !selected_ids.is_subset(&covered) {
                findings.push(format!("missing {op} action for requested property scope"));
            }
        }
        Predicate::OnlyLockChanges { allowed } => {
            let mut permitted = BTreeSet::new();
            for target in allowed {
                for (id, _) in styles(before, target)? {
                    permitted.insert((id, target.property));
                }
            }
            let target = Target {
                roles: vec!["*".into()],
                property: Property::Fg,
                include_semantic: true,
            };
            for ((id, a), (_, b)) in styles(before, &target)?.iter().zip(styles(after, &target)?) {
                for p in Property::ALL {
                    if a.is_locked(p) != b.is_locked(p) && !permitted.contains(&(id.clone(), p)) {
                        findings.push(format!("unrelated lock change {id} {p:?}"));
                    }
                }
            }
        }
        Predicate::Decision { allowed } => {
            if allowed.is_empty() {
                return Err("empty allowed decision list".into());
            }
            if !allowed.contains(&model.decision) {
                findings.push(format!("decision {:?} not accepted", model.decision));
            }
        }
        Predicate::NoMutation => {
            let target = Target {
                roles: vec!["*".into()],
                property: Property::Fg,
                include_semantic: true,
            };
            let a = styles(before, &target)?;
            let b = styles(after, &target)?;
            for ((id, a), (_, b)) in a.iter().zip(&b) {
                for p in Property::ALL {
                    if property(a, p) != property(b, p) || a.is_locked(p) != b.is_locked(p) {
                        findings.push(format!("unexpected mutation {id} {p:?}"));
                    }
                }
            }
        }
        Predicate::OnlyChanged { allowed } => {
            let mut permitted = BTreeSet::new();
            for target in allowed {
                for (id, _) in styles(before, target)? {
                    permitted.insert((id, target.property));
                }
            }
            let target = Target {
                roles: vec!["*".into()],
                property: Property::Fg,
                include_semantic: true,
            };
            for ((id, a), (_, b)) in styles(before, &target)?.iter().zip(styles(after, &target)?) {
                for p in Property::ALL {
                    if property(a, p) != property(b, p) && !permitted.contains(&(id.clone(), p)) {
                        findings.push(format!("unrelated visual change {id} {p:?}"));
                    }
                }
            }
        }
        _ => {
            let target = match check {
                Predicate::Preserved { target }
                | Predicate::State { target, .. }
                | Predicate::Bool { target, .. }
                | Predicate::Number { target, .. }
                | Predicate::Locked { target, .. }
                | Predicate::Color { target, .. }
                | Predicate::Lightness { target, .. }
                | Predicate::Chroma { target, .. }
                | Predicate::Contrast { target, .. }
                | Predicate::HuePreserved { target, .. }
                | Predicate::Secondary { target, .. } => target,
                _ => unreachable!(),
            };
            let old = styles(before, target)?;
            let new = styles(after, target)?;
            for ((id, a), (_, b)) in old.iter().zip(&new) {
                let ok = match check {
                    Predicate::HuePreserved {
                        tolerance_degrees, ..
                    } => {
                        if !tolerance_degrees.is_finite()
                            || !(0.0..=180.0).contains(tolerance_degrees)
                        {
                            return Err("invalid hue tolerance".into());
                        }
                        color(a, target.property)
                            .zip(color(b, target.property))
                            .is_some_and(|(a, b)| {
                                let distance = (a.to_oklch().h - b.to_oklch().h).abs();
                                distance.min(360.0 - distance) <= *tolerance_degrees
                            })
                    }
                    Predicate::Secondary {
                        primary,
                        background,
                        min,
                        ..
                    } => {
                        if !min.is_finite() || !(1.0..=21.0).contains(min) {
                            return Err("invalid secondary readability floor".into());
                        }
                        let bg = background.resolve(after)?;
                        let primary_style = after
                            .get(primary)
                            .ok_or("missing primary role for hierarchy")?;
                        let primary = color(primary_style, Property::Fg)
                            .ok_or("missing primary foreground for hierarchy")?;
                        let primary =
                            composite_over(primary, primary_style.fg_opacity.unwrap_or(1.0), bg);
                        color(b, target.property).is_some_and(|fg| {
                            let ratio = contrast_ratio(
                                composite_over(fg, b.fg_opacity.unwrap_or(1.0), bg),
                                bg,
                            );
                            ratio >= *min && ratio < contrast_ratio(primary, bg)
                        })
                    }
                    Predicate::State { value, .. } => b.states.get(&target.property) == Some(value),
                    Predicate::Preserved { .. } => {
                        property(a, target.property) == property(b, target.property)
                    }
                    Predicate::Bool { value, .. } => {
                        if !matches!(
                            target.property,
                            Property::Bold | Property::Italic | Property::Strikethrough
                        ) {
                            return Err("bool check needs typography property".into());
                        }
                        bool_value(b, target.property) == Some(*value)
                    }
                    Predicate::Number { value, .. } => {
                        if !value.is_finite() {
                            return Err("nonfinite expectation".into());
                        }
                        number(b, target.property).is_some_and(|n| (n - value).abs() <= 1e-9)
                    }
                    Predicate::Locked { value, .. } => b.is_locked(target.property) == *value,
                    Predicate::Color { value, .. } => color(b, target.property) == Some(*value),
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
                        if !min_delta.is_finite() || !max_delta.is_finite() || min_delta > max_delta
                        {
                            return Err("invalid metric bounds".into());
                        }
                        color(a, target.property)
                            .zip(color(b, target.property))
                            .is_some_and(|(a, b)| {
                                let delta = if matches!(check, Predicate::Lightness { .. }) {
                                    b.to_oklch().l - a.to_oklch().l
                                } else {
                                    b.to_oklch().c - a.to_oklch().c
                                };
                                delta >= *min_delta && delta <= *max_delta
                            })
                    }
                    Predicate::Contrast {
                        backgrounds, min, ..
                    } => {
                        if !min.is_finite() || !(1.0..=21.0).contains(min) {
                            return Err("invalid contrast expectation".into());
                        }
                        let backgrounds = crate::backdrop::resolve_all(after, backgrounds)?;
                        let alpha = match target.property {
                            Property::Fg => b.fg_opacity,
                            Property::Bg => b.bg_opacity,
                            Property::Special => b.special_opacity,
                            _ => None,
                        }
                        .unwrap_or(1.0);
                        color(b, target.property).is_some_and(|c| {
                            backgrounds.iter().all(|background| {
                                contrast_ratio(composite_over(c, alpha, *background), *background)
                                    >= *min
                            })
                        })
                    }
                    _ => unreachable!(),
                };
                if !ok {
                    findings.push(format!(
                        "semantic expectation failed for {id} {:?}",
                        target.property
                    ));
                }
            }
        }
    }
    Ok(findings)
}

pub fn run_suite(suite_path: &Path, run_path: &Path) -> Result<Evaluation, String> {
    let suite_raw = read(suite_path)?;
    let run_raw = read(run_path)?;
    let suite: Suite =
        serde_json::from_value(crate::jsonc::parse(&suite_raw)?).map_err(|e| e.to_string())?;
    let run: Run =
        serde_json::from_value(crate::jsonc::parse(&run_raw)?).map_err(|e| e.to_string())?;
    if suite.version != 1
        || run.version != 1
        || suite.cases.is_empty()
        || suite.cases.len() > 512
        || run.responses.len() > 512
    {
        return Err("invalid evaluation versions or count".into());
    }
    for text in [
        &suite.origin,
        &suite.license,
        &suite.revision,
        &suite.purpose,
        &run.label,
        &run.provider,
        &run.model,
        &run.prompt_version,
    ] {
        if text.trim().is_empty() {
            return Err("missing evaluation metadata".into());
        }
    }
    if suite.limitations.is_empty() || suite.limitations.iter().any(|l| l.trim().is_empty()) {
        return Err("evaluation limitations required".into());
    }
    if run.prompt_version != crate::interpret_v1::PROMPT_VERSION {
        return Err(
            "recorded prompt version does not match evaluator; replay with matching engine".into(),
        );
    }
    let mut ids = BTreeSet::new();
    let mut splits_by_id = BTreeMap::new();
    for case in &suite.cases {
        if let Some(previous) = &case.previous {
            if !ids.contains(previous) {
                return Err("previous turn must exist earlier in suite order".into());
            }
            if splits_by_id.get(previous) != Some(&case.split) {
                return Err("sequence cannot cross development/held-out partitions".into());
            }
        }
        if case.previous.is_some() && case.setup.is_some() {
            return Err("setup is only allowed at sequence root".into());
        }
        if case.id.trim().is_empty()
            || !ids.insert(case.id.clone())
            || case.checks.is_empty()
            || case.checks.len() > 256
            || !matches!(case.split.as_str(), "development" | "held_out")
            || case.category.trim().is_empty()
        {
            return Err("invalid or duplicate evaluation case".into());
        }
        splits_by_id.insert(case.id.clone(), case.split.clone());
    }
    crate::eval_contract::validate_pairs(&suite.pairs, &ids)?;
    let mut responses = BTreeMap::new();
    for response in &run.responses {
        if !ids.contains(&response.case_id)
            || responses
                .insert(response.case_id.clone(), &response.response)
                .is_some()
        {
            return Err("unknown or duplicate recorded response".into());
        }
    }
    let root = suite_path
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let mut results = vec![];
    let mut turns: BTreeMap<String, Turn> = BTreeMap::new();
    for case in &suite.cases {
        case.context.validate()?;
        let reference = root
            .join(&case.theme)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !reference.starts_with(&root) {
            return Err("evaluation reference escapes suite directory".into());
        }
        let input = read(&reference)?;
        let mut theme =
            crate::import_path::load_theme_path(&reference).map_err(|e| e.to_string())?;
        let state_shape = |t: &Theme| {
            (
                t.styles.keys().cloned().collect::<Vec<_>>(),
                t.modifiers
                    .iter()
                    .map(|r| {
                        (
                            r.role.clone(),
                            r.modifiers.clone(),
                            r.language.clone(),
                            r.token_type.clone(),
                            r.priority,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        };
        let reference_shape = state_shape(&theme);
        let mut context = case.context.clone();
        if let Some(previous) = case.previous.as_ref().and_then(|id| turns.get(id)) {
            if state_shape(&previous.after) != reference_shape {
                return Err(
                    "sequence reference inventory changed; use the same role/semantic contract"
                        .into(),
                );
            }
            theme = previous.after.clone();
            let mut inherited = previous.context.clone();
            inherited
                .previous_requests
                .extend(context.previous_requests);
            inherited.previous_actions.extend(context.previous_actions);
            context = inherited;
        } else if let Some(setup) = &case.setup {
            let (configured, report) = crate::intent::revise_intent(&theme, setup)?;
            if report.rolled_back
                || matches!(
                    report.outcome,
                    crate::protocol::Outcome::Infeasible
                        | crate::protocol::Outcome::Refused
                        | crate::protocol::Outcome::Partial
                )
            {
                return Err(format!("invalid authored setup for {}", case.id));
            }
            theme = configured;
        }
        crate::eval_contract::validate_case(case, &theme)?;
        let prompt = crate::interpret_v1::user_prompt(&theme, &case.request, &context)?;
        let mut result = CaseResult {
            id: case.id.clone(),
            split: case.split.clone(),
            category: case.category.clone(),
            passed: false,
            failure_class: None,
            findings: vec![],
            input_sha256: digest(input.as_bytes()),
            resolved_theme_sha256: digest(
                theme
                    .to_json_string()
                    .map_err(|e| e.to_string())?
                    .as_bytes(),
            ),
            response_sha256: None,
            prompt_sha256: digest(
                format!("{}\n{prompt}", crate::interpret_v1::system_prompt()).as_bytes(),
            ),
            outcome: None,
            decision: None,
            expectation: case.expectation,
            source_id: case.source_id.clone(),
            previous: case.previous.clone(),
            context_sha256: digest(
                serde_json::to_string(&context)
                    .map_err(|e| e.to_string())?
                    .as_bytes(),
            ),
            checks: case
                .checks
                .iter()
                .map(|check| CheckResult {
                    kind: check.kind(),
                    passed: false,
                    evaluated: false,
                    findings: vec!["not evaluated because response or dependency failed".into()],
                })
                .collect(),
            upstream_failed: case
                .previous
                .as_ref()
                .and_then(|id| turns.get(id))
                .is_some_and(|t| !t.passed),
        };
        let mut completed_turn = None;
        let checked = (|| -> Result<(), (&str, String)> {
            if case
                .previous
                .as_ref()
                .is_some_and(|id| !turns.contains_key(id))
            {
                return Err((
                    "dependency_failure",
                    "previous response could not be evaluated; no fresh-state fallback".into(),
                ));
            }
            let response = responses
                .get(&case.id)
                .ok_or(("missing_response", "no recorded response".into()))?;
            let raw = response
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| response.to_string());
            result.response_sha256 = Some(digest(raw.as_bytes()));
            let model = crate::interpret_v1::parse_model_output(&raw).map_err(|e| ("schema", e))?;
            let interpreted = crate::interpret_v1::validate(&model, &theme, &case.request)
                .map_err(|e| ("interpretation_validation", e))?;
            result.outcome = Some(interpreted.outcome);
            result.decision = Some(model.decision);
            let after = match &interpreted.spec {
                Some(spec) => {
                    crate::intent::revise_intent(&theme, spec)
                        .map_err(|e| ("revision", e))?
                        .0
                }
                None => theme.clone(),
            };
            for (index, check) in case.checks.iter().enumerate() {
                let findings =
                    evaluate_predicate(check, &theme, &after, &model, interpreted.outcome)
                        .map_err(|e| ("evaluation_contract", e))?;
                result.checks[index] = CheckResult {
                    kind: check.kind(),
                    passed: findings.is_empty(),
                    evaluated: true,
                    findings: findings.clone(),
                };
                result.findings.extend(findings);
            }
            let mut next_context = context.clone();
            next_context.previous_requests.push(case.request.clone());
            // The result theme carries retained locks/constraints. Do not tell
            // the next turn that refused or rolled-back actions were applied.
            if let Some(spec) = &interpreted.spec {
                if matches!(
                    interpreted.outcome,
                    crate::protocol::Outcome::Satisfied | crate::protocol::Outcome::Unchanged
                ) {
                    // Request-local locks and no-op moves are not retained edits.
                    for action in &spec.actions {
                        if matches!(
                            action,
                            crate::intent::Action::Lock {
                                scope: crate::intent::Scope::Request,
                                ..
                            }
                        ) {
                            continue;
                        }
                        let target = action.target();
                        let actual_change = styles(&theme, target)
                            .map_err(|e| ("evaluation_contract", e))?
                            .iter()
                            .zip(styles(&after, target).map_err(|e| ("evaluation_contract", e))?)
                            .any(|((_, a), (_, b))| {
                                property(a, target.property) != property(b, target.property)
                                    || a.is_locked(target.property) != b.is_locked(target.property)
                            });
                        if actual_change {
                            next_context.previous_actions.push(action.clone());
                        }
                    }
                }
            }
            completed_turn = Some(Turn {
                before: theme.clone(),
                after,
                context: next_context,
                model,
                passed: result.findings.is_empty(),
            });
            if !result.findings.is_empty() {
                return Err((
                    "semantic_outcome",
                    "required or forbidden semantic outcomes failed".into(),
                ));
            }
            Ok(())
        })();
        match checked {
            Ok(()) => result.passed = true,
            Err((class, reason)) => {
                result.failure_class = Some(class.into());
                result.findings.push(reason);
            }
        }
        if let Some(turn) = completed_turn {
            turns.insert(case.id.clone(), turn);
        }
        results.push(result);
    }
    let pair_results = crate::eval_pairs::evaluate_pairs(&suite.pairs, &turns, &ids)?;
    let pair_failures = pair_results.iter().filter(|p| !p.passed).count();
    let mut categories = BTreeMap::new();
    let mut classes = BTreeMap::new();
    for r in results.iter().filter(|r| !r.passed) {
        *categories.entry(r.category.clone()).or_insert(0) += 1;
        *classes.entry(r.failure_class.clone().unwrap()).or_insert(0) += 1;
    }
    let mut splits = BTreeMap::new();
    for r in &results {
        let summary = splits.entry(r.split.clone()).or_insert(SplitSummary {
            passed: 0,
            failed: 0,
        });
        if r.passed {
            summary.passed += 1;
        } else {
            summary.failed += 1;
        }
    }
    let mut scorecard = BTreeMap::new();
    let mut expectations = BTreeMap::new();
    for r in &results {
        let kind = serde_json::to_value(r.expectation)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let summary = expectations.entry(kind).or_insert(SplitSummary {
            passed: 0,
            failed: 0,
        });
        if r.passed {
            summary.passed += 1;
        } else {
            summary.failed += 1;
        }
        for check in &r.checks {
            let summary = scorecard
                .entry(check.kind.to_string())
                .or_insert(CheckSummary {
                    passed: 0,
                    failed: 0,
                    not_evaluated: 0,
                });
            if !check.evaluated {
                summary.not_evaluated += 1;
            } else if check.passed {
                summary.passed += 1;
            } else {
                summary.failed += 1;
            }
        }
    }
    let passed = results.iter().filter(|r| r.passed).count();
    let mut limitations = suite.limitations;
    limitations.push("Recorded provenance is caller-supplied; hashes establish reproducibility, not genuine provider authorship".into());
    limitations.push(
        "Canned replay is evaluator/contract evidence, not model accuracy or human preference"
            .into(),
    );
    Ok(Evaluation {
        version: 1,
        label: run.label,
        provider: run.provider,
        model: run.model,
        kind: run.kind,
        prompt_version: run.prompt_version,
        engine_version: env!("CARGO_PKG_VERSION").into(),
        suite_sha256: digest(suite_raw.as_bytes()),
        run_sha256: digest(run_raw.as_bytes()),
        passed,
        failed: results.len() - passed,
        cases: results,
        failures_by_category: categories,
        failures_by_class: classes,
        results_by_split: splits,
        pairs: pair_results,
        pair_failures,
        scorecard,
        results_by_expectation: expectations,
        limitations,
    })
}
