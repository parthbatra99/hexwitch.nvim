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
}
#[derive(Debug, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case", deny_unknown_fields)]
pub enum Predicate {
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
    OnlyChanged {
        allowed: Vec<Target>,
    },
    Decision {
        allowed: Vec<Decision>,
    },
    NoMutation,
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
fn styles<'a>(theme: &'a Theme, target: &Target) -> Result<Vec<(String, &'a Style)>, String> {
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
fn property(style: &Style, p: Property) -> serde_json::Value {
    let mut isolated = Style::new();
    isolated.copy_property_from(style, p);
    serde_json::to_value(isolated).unwrap()
}
fn color(style: &Style, p: Property) -> Option<Srgb> {
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

fn evaluate_predicate(
    check: &Predicate,
    before: &Theme,
    after: &Theme,
    model: &ModelInterpretation,
) -> Result<Vec<String>, String> {
    let mut findings = vec![];
    match check {
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
                | Predicate::Contrast { target, .. } => target,
                _ => unreachable!(),
            };
            let old = styles(before, target)?;
            let new = styles(after, target)?;
            for ((id, a), (_, b)) in old.iter().zip(&new) {
                let ok = match check {
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
    for case in &suite.cases {
        if case.id.trim().is_empty()
            || !ids.insert(case.id.clone())
            || case.checks.is_empty()
            || case.checks.len() > 256
            || !matches!(case.split.as_str(), "development" | "held_out")
            || case.category.trim().is_empty()
        {
            return Err("invalid or duplicate evaluation case".into());
        }
    }
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
        let theme = crate::import_path::load_theme_path(&reference).map_err(|e| e.to_string())?;
        let prompt = crate::interpret_v1::user_prompt(&theme, &case.request, &case.context)?;
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
        };
        let checked = (|| -> Result<(), (&str, String)> {
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
            let after = match &interpreted.spec {
                Some(spec) => {
                    crate::intent::revise_intent(&theme, spec)
                        .map_err(|e| ("revision", e))?
                        .0
                }
                None => theme.clone(),
            };
            for check in &case.checks {
                result.findings.extend(
                    evaluate_predicate(check, &theme, &after, &model)
                        .map_err(|e| ("evaluation_contract", e))?,
                );
            }
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
        results.push(result);
    }
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
        limitations,
    })
}
