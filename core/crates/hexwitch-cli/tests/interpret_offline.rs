use std::process::Command;

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_hexwitch")
}

fn eval_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../eval")
        .join(rel)
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(exe())
        .args(args)
        .output()
        .expect("spawn hexwitch");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn dry_run_prints_the_exact_prompt() {
    let (ok, stdout, _) = run(&[
        "interpret",
        &eval_path("themes/gruvbox-dark-medium.json")
            .display()
            .to_string(),
        "--dry-run",
        "make the comments readable but keep the vibe",
    ]);
    assert!(ok);
    assert!(stdout.contains("--- system ---"));
    assert!(stdout.contains("never invent colors"));
    assert!(stdout.contains("--- user ---"));
    assert!(stdout.contains("syntax.comment fg=#928374"));
    assert!(stdout.contains("surface.base bg=#282828"));
    assert!(stdout.contains("make the comments readable but keep the vibe"));
}

#[test]
fn offline_canned_response_flows_through_the_whole_engine() {
    let dir = std::env::temp_dir().join(format!("hexwitch-interpret-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec_path = dir.join("spec.json");
    let theme_path = dir.join("revised.json");

    let (ok, _stdout, stderr) = run(&[
        "interpret",
        &eval_path("themes/gruvbox-dark-medium.json")
            .display()
            .to_string(),
        "--from-llm",
        &eval_path("llm/darker-readable-comments.json")
            .display()
            .to_string(),
        "--out",
        &spec_path.display().to_string(),
    ]);
    assert!(ok, "interpret failed: {stderr}");
    assert!(
        stderr.contains("notes:"),
        "model notes should surface: {stderr}"
    );

    let spec_json = std::fs::read_to_string(&spec_path).unwrap();
    assert!(spec_json.contains("\"protect\""));
    assert!(spec_json.contains("\"min_contrast\""));

    let (ok, _stdout, stderr) = run(&[
        "revise",
        &eval_path("themes/gruvbox-dark-medium.json")
            .display()
            .to_string(),
        "--spec",
        &spec_path.display().to_string(),
        "--out",
        &theme_path.display().to_string(),
    ]);
    assert!(ok, "revise failed: {stderr}");
    assert!(stderr.contains("moved"));

    let revised =
        hexwitch_core::ingest::load_theme(&std::fs::read_to_string(&theme_path).unwrap()).unwrap();
    assert_eq!(
        revised
            .fg(&hexwitch_core::ir::Role::Syntax(
                hexwitch_core::ir::SyntaxRole::Keyword
            ))
            .unwrap()
            .to_hex(),
        "#fb4934"
    );

    let comment = revised
        .fg(&hexwitch_core::ir::role::Role::Syntax(
            hexwitch_core::ir::role::SyntaxRole::Comment,
        ))
        .unwrap();
    assert!(hexwitch_core::color::contrast_ratio(comment, revised.base_bg().unwrap()) >= 4.5);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn invalid_model_output_is_rejected_with_precision() {
    let dir = std::env::temp_dir().join(format!("hexwitch-interpret-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.json");
    std::fs::write(
        &bad,
        r#"{ "notes": "tried my best", "actions": [ { "action": "recolor", "target": "syntax.comment", "amount": null, "against": null, "min": null, "shift": null } ] }"#,
    )
    .unwrap();

    let (ok, _, stderr) = run(&[
        "interpret",
        &eval_path("themes/gruvbox-dark-medium.json")
            .display()
            .to_string(),
        "--from-llm",
        &bad.display().to_string(),
        "--out",
        &dir.join("spec.json").display().to_string(),
    ]);
    assert!(!ok, "invalid action must fail the command");
    assert!(
        stderr.contains("unknown action"),
        "error should name the problem: {stderr}"
    );
    assert!(
        !dir.join("spec.json").exists(),
        "no spec may be written on failure"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fenced_and_enveloped_responses_both_parse() {
    let dir = std::env::temp_dir().join(format!("hexwitch-interpret-fence-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fenced = dir.join("fenced.json");
    std::fs::write(
        &fenced,
        "```json\n{ \"notes\": null, \"actions\": [] }\n```",
    )
    .unwrap();
    let (ok, stdout, _) = run(&[
        "interpret",
        &eval_path("themes/gruvbox-dark-medium.json")
            .display()
            .to_string(),
        "--from-llm",
        &fenced.display().to_string(),
    ]);
    assert!(ok);
    assert!(
        stdout.contains("\"changes\""),
        "empty spec still serializes: {stdout}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
