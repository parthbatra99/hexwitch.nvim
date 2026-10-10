use hexwitch_core::ir::*;
use hexwitch_core::{import_path::load_theme_path, ingest::ingest_vscode_json};
#[test]
fn jsonc_alpha_reset_and_passthrough_are_explicit() {
    let t = ingest_vscode_json(r##"{
      // retained arbitrary source fields
      "colors":{"editor.background":"#000000","editor.selectionBackground":"#12345680",},
      "tokenColors":[{"scope":"comment","settings":{"foreground":"#ffffff","fontStyle":"italic"}},{"scope":"comment","settings":{"fontStyle":""}}],
      "unsupported":{"data":true}
    }"##).unwrap();
    let s = t
        .get(&Role::Interaction(InteractionRole::Selection))
        .unwrap();
    assert_eq!(s.bg_opacity, Some(128.0 / 255.0));
    assert_eq!(
        t.get(&Role::Syntax(SyntaxRole::Comment)).unwrap().italic,
        Some(false)
    );
    assert_eq!(t.extensions["vscode.source"]["unsupported"]["data"], true);
    assert!(ingest_vscode_json(r#"{"tokenColors":[{"settings":true}]}"#).is_err());
    assert!(ingest_vscode_json(r#"{"include":"other.json"}"#).is_err());
}
#[test]
fn includes_are_bounded_and_cycles_fail() {
    let root = std::env::temp_dir().join(format!("hexwitch-include-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let a = root.join("a.json");
    let b = root.join("b.json");
    std::fs::write(
        &a,
        r##"{"include":"b.json","colors":{"editor.foreground":"#ffffff"}}"##,
    )
    .unwrap();
    std::fs::write(&b, r##"{"colors":{"editor.background":"#000000"}}"##).unwrap();
    assert!(load_theme_path(&a).unwrap().base_bg().is_some());
    std::fs::write(&b, r#"{"include":"a.json"}"#).unwrap();
    assert!(load_theme_path(&a)
        .unwrap_err()
        .to_string()
        .contains("cycle"));
    std::fs::write(&b, r#"{"include":"../escape.json"}"#).unwrap();
    assert!(load_theme_path(&a)
        .unwrap_err()
        .to_string()
        .contains("unsafe"));
    std::fs::remove_file(a).unwrap();
    std::fs::remove_file(b).unwrap();
    std::fs::remove_dir(root).unwrap();
}
#[test]
fn malformed_ir_version_cannot_bypass_migration() {
    assert!(Theme::migrate_v0("[]").is_err());
    assert!(Theme::migrate_v0(r#"{"meta":{"ir_version":"future"},"styles":{}}"#).is_err());
    assert!(serde_json::from_str::<Theme>(
        r#"{"meta":{"name":"x","appearance":"dark","ir_version":999},"styles":{}}"#
    )
    .is_err());
}
