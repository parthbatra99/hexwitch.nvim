use serde_json::Value;
use std::process::Command;
fn eval(path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../eval")
        .join(path)
}
fn call(args: &[&str]) -> (bool, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_hexwitch"))
        .args(["--machine", "--request-id", "v1-test"])
        .args(args)
        .env_remove("OPENAI_API_KEY")
        .env_remove("OPENROUTER_API_KEY")
        .output()
        .unwrap();
    (
        output.status.success(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
#[test]
fn property_interpretation_writes_reviewable_intent_and_revises_offline() {
    let root = std::env::temp_dir().join(format!("hexwitch-v1-cli-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let spec = root.join("spec.json");
    let theme = eval("themes/nord.json");
    let response = eval("llm/property-opaque-background.json");
    let (ok, result) = call(&[
        "interpret",
        theme.to_str().unwrap(),
        "Keep the background color, but remove its transparency.",
        "--v1",
        "--from-llm",
        response.to_str().unwrap(),
        "--out",
        spec.to_str().unwrap(),
    ]);
    assert!(ok, "{result}");
    assert_eq!(result["result"]["spec"]["version"], 1);
    assert!(spec.is_file());
    let (ok, revised) = call(&[
        "revise",
        theme.to_str().unwrap(),
        "--spec",
        spec.to_str().unwrap(),
    ]);
    assert!(ok, "{revised}");
    assert_eq!(
        revised["result"]["theme"]["styles"]["surface.base"]["bg"],
        "#2e3440"
    );
    assert_eq!(
        revised["result"]["theme"]["styles"]["surface.base"]["bg_opacity"],
        1.0
    );
    std::fs::remove_file(spec).unwrap();
    std::fs::remove_dir(root).unwrap();
}
#[test]
fn clarification_does_not_write_or_overwrite_action_artifact() {
    let output =
        std::env::temp_dir().join(format!("hexwitch-v1-clarify-{}.json", std::process::id()));
    std::fs::write(&output, "reviewed original").unwrap();
    let (ok, result) = call(&[
        "interpret",
        eval("themes/nord.json").to_str().unwrap(),
        "Make that blue quieter.",
        "--v1",
        "--from-llm",
        eval("llm/clarify-blue.json").to_str().unwrap(),
        "--out",
        output.to_str().unwrap(),
    ]);
    assert!(ok);
    assert_eq!(result["outcome"], "needs_clarification");
    assert!(result["result"]["spec"].is_null());
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "reviewed original"
    );
    std::fs::remove_file(output).unwrap();
}
#[test]
fn v1_rejects_untrusted_anchor_and_requires_original_request() {
    let (ok, result) = call(&[
        "interpret",
        eval("themes/nord.json").to_str().unwrap(),
        "Warmer keywords.",
        "--v1",
        "--from-llm",
        eval("llm/invalid-invented-anchor.json").to_str().unwrap(),
    ]);
    assert!(!ok);
    assert_eq!(result["errors"][0]["code"], "invalid_input");
    let (ok, result) = call(&[
        "interpret",
        eval("themes/nord.json").to_str().unwrap(),
        "--v1",
        "--from-llm",
        eval("llm/property-opaque-background.json")
            .to_str()
            .unwrap(),
    ]);
    assert!(!ok);
    assert_eq!(result["errors"][0]["code"], "invalid_input");
}
#[test]
fn executable_semantic_eval_reports_reproducibility_and_honest_replay_kind() {
    let (ok, result) = call(&[
        "eval-interpretation",
        eval("interpretation-suite-v1.json").to_str().unwrap(),
        "--run",
        eval("interpretation-contract-replay.json")
            .to_str()
            .unwrap(),
    ]);
    assert!(ok, "{result}");
    assert_eq!(result["result"]["passed"], 12);
    assert_eq!(result["result"]["kind"], "canned_replay");
    assert_eq!(result["result"]["suite_sha256"].as_str().unwrap().len(), 64);
}
