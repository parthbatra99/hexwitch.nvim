use serde_json::Value;
use std::process::Command;
fn call(args: &[&str]) -> (bool, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_hexwitch"))
        .args(["--machine", "--request-id", "test-1"])
        .args(args)
        .output()
        .unwrap();
    (
        output.status.success(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
#[test]
fn discovery_and_input_errors_have_stable_envelopes() {
    let (ok, value) = call(&["capabilities"]);
    assert!(ok);
    assert_eq!(value["protocol_version"], 1);
    assert_eq!(value["request_id"], "test-1");
    let (ok, value) = call(&["import", "/nonexistent/theme.json"]);
    assert!(!ok);
    assert_eq!(value["errors"][0]["code"], "io");
}
#[test]
fn import_validate_and_interpret_are_offline_and_structured() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval");
    let theme = root.join("themes/nord.json");
    let (ok, value) = call(&["import", theme.to_str().unwrap()]);
    assert!(ok);
    assert_eq!(value["result"]["theme"]["meta"]["ir_version"], 1);
    let (ok, value) = call(&["validate", theme.to_str().unwrap()]);
    assert!(ok);
    assert_ne!(value["result"]["status"], "pass");
}
