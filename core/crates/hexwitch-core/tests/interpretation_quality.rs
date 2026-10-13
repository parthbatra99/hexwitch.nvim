use std::path::Path;

fn suite() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval/interpretation-suite-v1.json")
}
fn replay() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval/interpretation-contract-replay.json")
}
#[test]
fn canned_replay_tests_semantic_contract_not_model_accuracy() {
    let report = hexwitch_core::interpret_eval::run_suite(&suite(), &replay()).unwrap();
    assert_eq!(
        report.failed,
        0,
        "{:?}",
        serde_json::to_value(&report).unwrap()
    );
    assert_eq!(report.passed, 12);
    assert!(report.cases.iter().all(|c| c.input_sha256.len() == 64
        && c.prompt_sha256.len() == 64
        && c.response_sha256.as_ref().is_some_and(|h| h.len() == 64)));
    assert_eq!(
        report
            .cases
            .iter()
            .filter(|c| c.split == "held_out")
            .count(),
        4
    );
}
#[test]
fn absent_response_is_a_categorized_failure_not_a_skipped_case() {
    let mut run: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(replay()).unwrap()).unwrap();
    run["responses"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["case_id"] != "i37");
    let path =
        std::env::temp_dir().join(format!("hexwitch-eval-missing-{}.json", std::process::id()));
    std::fs::write(&path, run.to_string()).unwrap();
    let report = hexwitch_core::interpret_eval::run_suite(&suite(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(report.failed, 1);
    assert_eq!(report.failures_by_class["missing_response"], 1);
    assert_eq!(report.results_by_split["held_out"].failed, 1);
}
#[test]
fn schema_valid_unrelated_changes_fail_semantic_outcomes() {
    let mut run: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(replay()).unwrap()).unwrap();
    let record = run["responses"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["case_id"] == "i16")
        .unwrap();
    record["response"]["actions"][0]["roles"] = serde_json::json!(["syntax.comment"]);
    record["response"]["actions"][0]["property"] = serde_json::json!("fg");
    let path = std::env::temp_dir().join(format!(
        "hexwitch-eval-regression-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, run.to_string()).unwrap();
    let report = hexwitch_core::interpret_eval::run_suite(&suite(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(report.failed, 1);
    let failed = report.cases.iter().find(|c| c.id == "i16").unwrap();
    assert_eq!(failed.failure_class.as_deref(), Some("semantic_outcome"));
    assert!(failed.findings.iter().any(|f| f.contains("unrelated")));
}
