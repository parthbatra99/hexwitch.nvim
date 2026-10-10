//! End-to-end eval set: ingest + analyze the reference fixtures and check
//! them against independently derived goldens plus structural invariants.
//!
//! The contrast goldens were computed outside this codebase during the
//! research phase from the themes' published colors. If this test fails
//! after a color-math or ingest change, the engine moved — not the goal.

use hexwitch_core::eval::run_eval_dir;

fn eval_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval")
}

#[test]
fn eval_set_passes() {
    let outcomes = run_eval_dir(&eval_dir()).expect("eval set should load and run");
    assert!(
        !outcomes.is_empty(),
        "eval set must contain at least one case"
    );
    let report = hexwitch_core::eval::render_report(&outcomes);
    for outcome in &outcomes {
        assert!(
            outcome.passed,
            "case `{}` failed:\n{}",
            outcome.theme, report
        );
    }
}

#[test]
fn eval_goldens_are_actually_independent_numbers() {
    // Guard against goldens drifting into self-consistency: the six themes
    // span a wide, *distinct* range of body/comment contrast, which is the
    // signature of the research measurements (not of one formula's output).
    let outcomes = run_eval_dir(&eval_dir()).unwrap();
    let primaries: Vec<f64> = outcomes
        .iter()
        .map(|o| o.fingerprint.primary_contrast().unwrap())
        .collect();
    let comments: Vec<f64> = outcomes
        .iter()
        .map(|o| o.fingerprint.comment_contrast().unwrap())
        .collect();
    let spread = primaries.iter().cloned().fold(f64::MIN, f64::max)
        - primaries.iter().cloned().fold(f64::MAX, f64::min);
    assert!(spread > 8.0, "primary contrasts too similar: {primaries:?}");
    let c_spread = comments.iter().cloned().fold(f64::MIN, f64::max)
        - comments.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        c_spread > 3.0,
        "comment contrasts too similar: {comments:?}"
    );
}

#[test]
fn themes_ingest_with_distinct_identities() {
    // Same test as the product thesis, mechanically: six references must
    // produce six distinguishable fingerprints (accent hue sets), not six
    // variations of one engine aesthetic.
    let outcomes = run_eval_dir(&eval_dir()).unwrap();
    let hue_sets: Vec<Vec<f64>> = outcomes
        .iter()
        .map(|o| {
            let mut hues: Vec<f64> = o.fingerprint.accents.iter().map(|a| a.h).collect();
            hues.sort_by(|a, b| a.partial_cmp(b).unwrap());
            hues
        })
        .collect();
    for (i, a) in hue_sets.iter().enumerate() {
        for (j, b) in hue_sets.iter().enumerate().skip(i + 1) {
            assert_ne!(a, b, "themes {i} and {j} have identical accent hues");
        }
    }
}

#[test]
fn ir_json_roundtrip_survives_ingest() {
    let outcomes = run_eval_dir(&eval_dir()).unwrap();
    for o in &outcomes {
        let json = o.theme_ref.to_json_string().unwrap();
        let back = hexwitch_core::ir::theme::Theme::from_json_str(&json).unwrap();
        assert_eq!(back, o.theme_ref, "IR roundtrip failed for {}", o.theme);
    }
}
