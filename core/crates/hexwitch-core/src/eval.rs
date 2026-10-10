//! Eval set: ingest + analyze real reference themes and compare against
//! independently derived golden numbers.
//!
//! The goldens for contrast ratios come from measurements made *outside
//! this codebase* during the research phase (hand-computed WCAG values for
//! the published colors of Gruvbox, Tokyo Night, Catppuccin, Dracula,
//! Solarized, Nord). That makes this a real cross-validation of the color
//! math and ingest mapping, not a self-consistency check.
//!
//! Layout of an eval directory:
//!
//! ```text
//! eval/
//!   manifest.json          { "cases": [ { "file": "...", "expect": {...} } ] }
//!   themes/*.json          VS Code-format reference fixtures
//! ```
//!
//! Run via the CLI (`hexwitch eval <dir>`) or the integration test.

use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::analyze::{fingerprint, Fingerprint};
use crate::ingest::ingest_vscode_json;
use crate::ir::theme::Theme;

/// Contrast goldens are compared with this tolerance.
const RATIO_TOLERANCE: f64 = 0.02;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ingest: {0}")]
    Ingest(#[from] crate::ingest::IngestError),
    #[error("analyze: {0}")]
    Analyze(#[from] crate::analyze::AnalyzeError),
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default)]
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    file: String,
    #[serde(default)]
    expect: Expect,
}

#[derive(Debug, Deserialize, Default)]
struct Expect {
    appearance: Option<String>,
    primary_contrast: Option<f64>,
    comment_contrast: Option<f64>,
    syntax_roles: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub what: &'static str,
    pub expected: String,
    pub actual: String,
    pub ok: bool,
}

#[derive(Debug, Clone)]
pub struct EvalOutcome {
    pub theme: String,
    pub fingerprint: Fingerprint,
    pub theme_ref: Theme,
    pub checks: Vec<Check>,
    pub passed: bool,
}

impl EvalOutcome {
    /// Structural invariants every theme in the set must satisfy,
    /// independent of per-theme goldens.
    fn invariant_checks(theme: &Theme, fp: &Fingerprint) -> Vec<Check> {
        let mut checks = Vec::new();
        checks.push(Check {
            what: "polarity",
            expected: format!(
                "{} bg/fg ordering",
                match fp.appearance {
                    crate::ir::theme::Appearance::Dark => "dark",
                    crate::ir::theme::Appearance::Light => "light",
                }
            ),
            actual: if crate::analyze::polarity_is_consistent(theme) {
                "consistent".into()
            } else {
                "INVERTED".into()
            },
            ok: crate::analyze::polarity_is_consistent(theme),
        });
        checks.push(Check {
            what: "comment<=primary",
            expected: "comments not brighter than primary".into(),
            actual: if crate::analyze::comment_not_brighter_than_primary(theme) {
                "ok".into()
            } else {
                "comment outshines text".into()
            },
            ok: crate::analyze::comment_not_brighter_than_primary(theme),
        });
        checks
    }
}

/// Run the eval set in `dir`, returning one outcome per case.
pub fn run_eval_dir(dir: &Path) -> Result<Vec<EvalOutcome>, EvalError> {
    let manifest: Manifest = serde_json::from_str(&fs::read_to_string(dir.join("manifest.json"))?)?;
    let mut outcomes = Vec::with_capacity(manifest.cases.len());

    for case in &manifest.cases {
        let path = dir.join(&case.file);
        let json = fs::read_to_string(&path)?;
        let theme = ingest_vscode_json(&json)?;
        let fp = fingerprint(&theme)?;

        let mut checks = EvalOutcome::invariant_checks(&theme, &fp);

        if let Some(want) = &case.expect.appearance {
            let got = match fp.appearance {
                crate::ir::theme::Appearance::Dark => "dark",
                crate::ir::theme::Appearance::Light => "light",
            };
            checks.push(Check {
                what: "appearance",
                expected: want.clone(),
                actual: got.to_string(),
                ok: want == got,
            });
        }
        if let Some(want) = case.expect.primary_contrast {
            let got = fp.primary_contrast().unwrap_or(f64::NAN);
            checks.push(Check {
                what: "primary_contrast",
                expected: format!("{want:.2}"),
                actual: format!("{got:.2}"),
                ok: (got - want).abs() <= RATIO_TOLERANCE,
            });
        }
        if let Some(want) = case.expect.comment_contrast {
            let got = fp.comment_contrast().unwrap_or(f64::NAN);
            checks.push(Check {
                what: "comment_contrast",
                expected: format!("{want:.2}"),
                actual: format!("{got:.2}"),
                ok: (got - want).abs() <= RATIO_TOLERANCE,
            });
        }
        if let Some(want) = case.expect.syntax_roles {
            checks.push(Check {
                what: "syntax_roles",
                expected: want.to_string(),
                actual: fp.syntax_roles.to_string(),
                ok: fp.syntax_roles == want,
            });
        }

        let passed = checks.iter().all(|c| c.ok);
        outcomes.push(EvalOutcome {
            theme: theme.meta.name.clone(),
            fingerprint: fp,
            theme_ref: theme,
            checks,
            passed,
        });
    }
    Ok(outcomes)
}

/// Render a human-readable report (used by CLI and tests on failure).
pub fn render_report(outcomes: &[EvalOutcome]) -> String {
    let mut out = String::new();
    for o in outcomes {
        let status = if o.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "[{status}] {} ({} syntax roles)\n",
            o.theme, o.fingerprint.syntax_roles
        ));
        for c in &o.checks {
            if !c.ok || c.what.starts_with("primary") || c.what.starts_with("comment") {
                out.push_str(&format!(
                    "       {:<18} expected {:<10} got {}\n",
                    c.what, c.expected, c.actual
                ));
            }
        }
    }
    let failed = outcomes.iter().filter(|o| !o.passed).count();
    out.push_str(&format!(
        "\n{} case(s), {} failed\n",
        outcomes.len(),
        failed
    ));
    out
}
