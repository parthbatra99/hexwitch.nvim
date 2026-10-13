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
fn semantic_include_rules_keep_parent_properties_and_child_precedence() {
    let root =
        std::env::temp_dir().join(format!("hexwitch-semantic-include-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let parent = root.join("parent.json");
    let child = root.join("child.json");
    std::fs::write(&parent,r##"{"semanticHighlighting":true,"semanticTokenColors":{"variable.readonly":{"foreground":"#123456","bold":true},"*.static":"#ff0000"}}"##).unwrap();
    std::fs::write(&child,r##"{"include":"parent.json","semanticHighlighting":false,"semanticTokenColors":{"variable.readonly":{"italic":true},"*.async":"#00ff00"}}"##).unwrap();
    let t = load_theme_path(&child).unwrap();
    let role = Role::Syntax(SyntaxRole::Variable);
    let context = hexwitch_core::semantic::SemanticContext {
        token_type: "variable".into(),
        modifiers: std::collections::BTreeSet::from([
            "readonly".into(),
            "static".into(),
            "async".into(),
        ]),
        language: None,
    };
    let result =
        hexwitch_core::semantic::resolve_semantic(&t, &role, &context, t.get(&role).unwrap())
            .unwrap();
    assert_eq!(result.style.bold, Some(true));
    assert_eq!(result.style.italic, Some(true));
    assert_eq!(
        result.style.fg,
        Some(hexwitch_core::color::Srgb::from_hex("#123456").unwrap())
    );
    assert_eq!(t.extensions["vscode.semanticHighlighting"], true);
    std::fs::remove_file(parent).unwrap();
    std::fs::remove_file(child).unwrap();
    std::fs::remove_dir(root).unwrap();
}
#[test]
fn malformed_unmapped_or_shadowed_settings_do_not_disappear() {
    for bad in [
        r##"{"colors":{"future.color":"not-a-color"}}"##,
        r##"{"tokenColors":[{"scope":"meta.unmapped","settings":{"foreground":"#zzzzzz"}}]}"##,
        r##"{"tokenColors":[{"scope":"comment","settings":{"fontStyle":"oblique"}}]}"##,
    ] {
        assert!(ingest_vscode_json(bad).is_err(), "{bad}");
    }
    let short = ingest_vscode_json(
        r##"{"colors":{"editor.background":"#abc","editor.selectionBackground":"#1234"}}"##,
    )
    .unwrap();
    assert_eq!(
        short.base_bg(),
        Some(hexwitch_core::color::Srgb::from_hex("#aabbcc").unwrap())
    );
    assert_eq!(
        short
            .get(&Role::Interaction(InteractionRole::Selection))
            .unwrap()
            .bg_opacity,
        Some(68.0 / 255.0)
    );
}
#[test]
fn complete_pinned_dark_plus_loads_with_explicit_scope_and_custom_type_losses() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../eval/import/vscode-1.95.0/dark_plus.json");
    let theme = load_theme_path(&path).unwrap();
    assert_eq!(
        theme.base_bg(),
        Some(hexwitch_core::color::Srgb::from_hex("#1e1e1e").unwrap())
    );
    assert_eq!(
        theme.fg(&Role::Syntax(SyntaxRole::Comment)),
        Some(hexwitch_core::color::Srgb::from_hex("#6a9955").unwrap())
    );
    let source = &theme.extensions["vscode.source"];
    assert!(source["tokenColors"].as_array().unwrap().len() > 50);
    for (stack, hex) in [
        (["source.ts", "entity.name.function.ts"], "#dcdcaa"),
        (["source.ts", "entity.name.type.ts"], "#4ec9b0"),
        (["source.ts", "variable.other.readwrite.ts"], "#9cdcfe"),
    ] {
        let scopes = stack.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let resolved =
            hexwitch_core::scope::resolve_scope_stack(source, &scopes, &Style::new()).unwrap();
        assert_eq!(
            resolved.style.fg,
            Some(hexwitch_core::color::Srgb::from_hex(hex).unwrap())
        );
    }
    let losses = theme.extensions["hexwitch.import.losses.v1"]
        .as_array()
        .unwrap();
    assert!(losses
        .iter()
        .any(|l| l["property"] == "semanticTokenColors.newOperator"));
    assert!(losses.iter().any(|l| l.get("source_stacks").is_some()));
}
#[test]
fn actual_scope_probe_preserves_property_precedence_and_qualifies_unknown_rules() {
    let source = serde_json::json!({"tokenColors":[{"scope":"comment","settings":{"foreground":"#eeeeee","fontStyle":"italic"}},{"scope":"source.rust comment.line","settings":{"fontStyle":""}},{"scope":"comment - string","settings":{"foreground":"#ff0000"}}]});
    let result = hexwitch_core::scope::resolve_scope_stack(
        &source,
        &[
            "source.rust".into(),
            "comment.line.double-slash.rust".into(),
        ],
        &Style::new(),
    )
    .unwrap();
    assert_eq!(
        result.style.fg,
        Some(hexwitch_core::color::Srgb::from_hex("#eeeeee").unwrap())
    );
    assert_eq!(result.style.italic, Some(false));
    assert!(!result.limitations.is_empty());
    assert_eq!(result.property_rules[&Property::Italic], 1);
}
#[test]
fn malformed_ir_version_cannot_bypass_migration() {
    assert!(Theme::migrate_v0("[]").is_err());
    assert!(ingest_vscode_json(r#"{"colors":{},"hexwitchSemanticLayers":[]}"#).is_err());
    assert!(ingest_vscode_json(
        r##"{"colors":{},"semanticTokenColors":{"variable":"#fff","variable":"#000"}}"##
    )
    .is_err());
    assert!(Theme::migrate_v0(r#"{"meta":{"ir_version":"future"},"styles":{}}"#).is_err());
    assert!(serde_json::from_str::<Theme>(
        r#"{"meta":{"name":"x","appearance":"dark","ir_version":999},"styles":{}}"#
    )
    .is_err());
}
