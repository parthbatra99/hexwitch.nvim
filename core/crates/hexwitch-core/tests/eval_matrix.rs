use hexwitch_core::interpret_eval::{run_suite, Evaluation};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval")
}
fn suite() -> PathBuf {
    root().join("interpretation-matrix-v1.json")
}
fn replay() -> PathBuf {
    root().join("interpretation-matrix-replay.json")
}
fn result(id: &str, report: &Evaluation) -> Value {
    serde_json::to_value(report.cases.iter().find(|c| c.id == id).unwrap()).unwrap()
}

#[test]
fn full_matrix_covers_original_requests_states_and_sequences() {
    let report = run_suite(&suite(), &replay()).unwrap();
    assert_eq!(
        report.failed,
        0,
        "{}",
        serde_json::to_string_pretty(&report).unwrap()
    );
    assert_eq!(report.passed, 74);
    assert_eq!(report.pair_failures, 0);
    assert_eq!(report.pairs.len(), 3);
    let ids: std::collections::BTreeSet<_> = report
        .cases
        .iter()
        .filter_map(|c| c.source_id.as_deref())
        .collect();
    assert_eq!(ids.len(), 50);
    assert_eq!(
        report.cases.iter().filter(|c| c.previous.is_some()).count(),
        6
    );
    assert!(report.results_by_expectation["edit"].passed > 30);
    assert!(report.results_by_expectation["clarification"].passed > 10);
    assert_ne!(
        result("seq-lock", &report)["resolved_theme_sha256"],
        result("seq-opacity", &report)["resolved_theme_sha256"]
    );
    assert_ne!(
        result("i45", &report)["prompt_sha256"],
        result("i45-with-context", &report)["prompt_sha256"]
    );
    assert_eq!(result("seq-refused", &report)["outcome"], "refused");
}
#[test]
fn known_bad_outputs_trigger_expected_detectors() {
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("mutations/manifest.json")).unwrap(),
    )
    .unwrap();
    for mutation in manifest["mutations"].as_array().unwrap() {
        let report = run_suite(
            &suite(),
            &root()
                .join("mutations")
                .join(mutation["file"].as_str().unwrap()),
        )
        .unwrap();
        let case = result(mutation["case_id"].as_str().unwrap(), &report);
        let kind = mutation["expected_check"].as_str().unwrap();
        if kind == "pair_only" {
            assert_eq!(
                report.failed, 0,
                "individual checks should still pass: {}",
                mutation["file"]
            );
            assert!(report.pair_failures > 0);
        } else if kind == "interpretation_validation" {
            assert_eq!(case["failure_class"], "interpretation_validation");
        } else {
            assert_eq!(
                case["passed"], false,
                "mutation escaped detection: {}",
                mutation["file"]
            );
            assert!(
                case["checks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["kind"] == kind && c["passed"] == false),
                "expected {kind} detector for {}: {case}",
                mutation["file"]
            );
        }
        if let Some(pair) = mutation.get("expected_pair") {
            assert!(report
                .pairs
                .iter()
                .any(|p| p.id == pair.as_str().unwrap() && !p.passed));
        }
    }
}
#[test]
fn malformed_annotations_are_authoring_errors_not_model_failures() {
    let mut suite_value: Value =
        serde_json::from_str(&std::fs::read_to_string(suite()).unwrap()).unwrap();
    suite_value["cases"][0]["checks"] = serde_json::json!([{"check":"bool","target":{"roles":["surface.base"],"property":"bg"},"value":false}]);
    let path = root().join(format!(
        ".eval-contract-failure-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, suite_value.to_string()).unwrap();
    let error = run_suite(&path, &replay()).unwrap_err();
    std::fs::remove_file(path).unwrap();
    assert!(error.contains("invalid evaluation contract"));
}
#[test]
fn malformed_previous_turn_blocks_dependents_not_fresh_state_fallback() {
    let mut run: Value = serde_json::from_str(&std::fs::read_to_string(replay()).unwrap()).unwrap();
    run["responses"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["case_id"] == "seq-lock")
        .unwrap()["response"] = serde_json::json!({"not":"a response"});
    let path = std::env::temp_dir().join(format!(
        "hexwitch-sequence-failure-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, run.to_string()).unwrap();
    let report = run_suite(&suite(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(result("seq-lock", &report)["failure_class"], "schema");
    assert_eq!(
        result("seq-opacity", &report)["failure_class"],
        "dependency_failure"
    );
    assert_eq!(
        result("seq-readable", &report)["failure_class"],
        "dependency_failure"
    );
    assert!(report.scorecard["readability"].not_evaluated > 0);
    assert_eq!(
        result("seq-opacity", &report)["checks"][0]["evaluated"],
        false
    );
}
