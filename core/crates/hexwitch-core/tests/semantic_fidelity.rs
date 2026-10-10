use hexwitch_core::color::Srgb;
use hexwitch_core::ingest::ingest_vscode_json;
use hexwitch_core::ir::*;
use hexwitch_core::semantic::{resolve_semantic, SemanticContext};
use std::collections::BTreeSet;

fn theme() -> Theme {
    ingest_vscode_json(include_str!(
        "../../../eval/import/semantic-precedence.jsonc"
    ))
    .unwrap()
}
fn context(token_type: &str, modifiers: &[&str], language: Option<&str>) -> SemanticContext {
    SemanticContext {
        token_type: token_type.into(),
        modifiers: modifiers.iter().map(|s| s.to_string()).collect(),
        language: language.map(str::to_string),
    }
}
fn resolved(theme: &Theme, kind: &str, modifiers: &[&str], language: Option<&str>) -> Style {
    let role = hexwitch_core::semantic::portable_role(kind).unwrap();
    let base = theme.get(&role).unwrap();
    resolve_semantic(theme, &role, &context(kind, modifiers, language), base)
        .unwrap()
        .style
}
#[test]
fn lexical_base_is_not_replaced_by_semantic_rules() {
    let t = theme();
    assert_eq!(
        t.fg(&Role::Syntax(SyntaxRole::Variable)),
        Some(Srgb::from_hex("#bb9966").unwrap())
    );
    assert_eq!(
        resolved(&t, "variable", &[], None).fg,
        Some(Srgb::from_hex("#112233").unwrap())
    );
    assert_eq!(resolved(&t, "variable", &[], None).italic, Some(false));
    assert_eq!(
        Theme::from_json_str(&t.to_json_string().unwrap()).unwrap(),
        t
    );
}
#[test]
fn selector_specificity_and_omission_resolve_per_property() {
    let t = theme();
    let rust = resolved(&t, "variable", &["readonly"], Some("rust"));
    assert_eq!(rust.fg, Some(Srgb::from_hex("#a1b2c3").unwrap()));
    assert_eq!(rust.fg_opacity, Some(128.0 / 255.0));
    assert_eq!(rust.bold, Some(true));
    assert_eq!(rust.italic, Some(true));
    let other = resolved(&t, "variable", &["readonly"], Some("python"));
    assert_eq!(other.fg, Some(Srgb::from_hex("#334455").unwrap()));
    assert_eq!(other.italic, Some(false));
    let declaration = resolved(&t, "variable", &["readonly", "declaration"], Some("rust"));
    assert_eq!(declaration.fg, Some(Srgb::from_hex("#ccddee").unwrap()));
    assert_eq!(
        declaration.fg_opacity,
        Some(1.0),
        "opaque rule inherited unrelated rule alpha"
    );
    assert_eq!(declaration.italic, Some(true));
}
#[test]
fn ties_follow_source_order_not_sorted_map_keys() {
    let t = theme();
    let s = resolved(&t, "variable", &["async", "static"], None);
    assert_eq!(s.fg, Some(Srgb::from_hex("#22bb44").unwrap()));
    let round = Theme::from_json_str(&t.to_json_string().unwrap()).unwrap();
    assert_eq!(resolved(&round, "variable", &["async", "static"], None), s);
}
#[test]
fn class_and_interface_do_not_collapse_and_decorations_coexist() {
    let t = theme();
    assert_eq!(
        resolved(&t, "class", &[], None).fg,
        Some(Srgb::from_hex("#6688cc").unwrap())
    );
    assert_eq!(
        resolved(&t, "interface", &[], None).fg,
        Some(Srgb::from_hex("#cc8866").unwrap())
    );
    let function = resolved(&t, "function", &[], None);
    assert_eq!(
        function.bold,
        Some(false),
        "fontStyle did not reset individual bold"
    );
    assert_eq!(function.underline, Some(UnderlineStyle::Single));
    assert_eq!(function.strikethrough, Some(true));
}
#[test]
fn explicit_disable_and_unknown_enablement_are_not_hidden() {
    let mut t = theme();
    t.extensions.insert(
        "vscode.semanticHighlighting".into(),
        serde_json::json!(false),
    );
    let role = Role::Syntax(SyntaxRole::Variable);
    assert_eq!(
        resolved(&t, "variable", &[], None),
        t.get(&role).unwrap().clone()
    );
    t.extensions.remove("vscode.semanticHighlighting");
    assert!(!resolve_semantic(
        &t,
        &role,
        &context("variable", &[], None),
        t.get(&role).unwrap()
    )
    .unwrap()
    .limitations
    .is_empty());
}
#[test]
fn malformed_semantic_input_fails_instead_of_becoming_a_noop() {
    assert!(
        ingest_vscode_json(include_str!("../../../eval/import/malformed-semantic.json")).is_err()
    );
    for bad in [
        r##"{"semanticTokenColors":{"variable":{"italic":"yes"}}}"##,
        r##"{"semanticTokenColors":{"variable":{"foreground":"#xyzxyz"}}}"##,
        r##"{"semanticTokenColors":{"variable":{"fontStyle":"oblique"}}}"##,
        r##"{"semanticTokenColors":{},"semanticHighlighting":"yes"}"##,
    ] {
        assert!(ingest_vscode_json(bad).is_err(), "{bad}");
    }
    assert!(context("variable", &["bad.modifier"], None)
        .validate()
        .is_err());
}
#[test]
fn unsupported_semantics_are_named_and_source_retained() {
    let t = theme();
    assert!(t.extensions["hexwitch.import.losses.v1"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["property"] == "syntax.comment" && l.get("source_stacks").is_some()));
    let losses = t.extensions["hexwitch.import.losses.v1"]
        .as_array()
        .unwrap();
    assert!(losses
        .iter()
        .any(|l| l["property"] == "semanticTokenColors.customType.native"));
    assert!(losses
        .iter()
        .any(|l| l["property"] == "semanticTokenColors.property.background"));
    assert_eq!(
        t.extensions["vscode.source"]["semanticTokenColors"]["customType.native"]["foreground"],
        "#123456"
    );
}
#[test]
fn revisions_can_include_semantic_overrides_without_touching_colors() {
    let t = theme();
    let spec:hexwitch_core::intent::IntentSpec = serde_json::from_value(serde_json::json!({"version":1,"actions":[{"op":"set_bool","target":{"roles":["*"],"property":"italic","include_semantic":true},"value":false}]})).unwrap();
    let (out, report) = hexwitch_core::intent::revise_intent(&t, &spec).unwrap();
    for (role, style) in &out.styles {
        assert_eq!(style.fg, t.get(role).unwrap().fg);
        assert_eq!(style.bg, t.get(role).unwrap().bg);
    }
    assert!(out.modifiers.iter().all(|r| r.style.italic == Some(false)));
    assert!(report.properties.iter().any(|r| r.semantic_rule.is_some()));
    assert_eq!(
        resolved(&out, "variable", &["readonly"], Some("rust")).italic,
        Some(false)
    );
    let mut locked = t.clone();
    let index = locked
        .modifiers
        .iter()
        .position(|r| r.language.as_deref() == Some("rust"))
        .unwrap();
    locked.modifiers[index]
        .style
        .locked
        .insert(Property::Italic);
    let (out, report) = hexwitch_core::intent::revise_intent(&locked, &spec).unwrap();
    assert_eq!(out.modifiers[index].style.italic, Some(true));
    assert!(report
        .properties
        .iter()
        .any(|r| r.semantic_rule == Some(index)
            && r.outcome == hexwitch_core::protocol::Outcome::Refused));
}
#[test]
fn semantic_scene_uses_effective_alpha_and_enablement() {
    let t = theme();
    let scene = hexwitch_core::validate::Scene {
        name: "semantic readonly".into(),
        foreground: Role::Syntax(SyntaxRole::Variable),
        surface: Role::Surface(Surface::Base),
        overlays: vec![],
        min_contrast: 4.5,
        ordering_verified: false,
        semantic: Some(context("variable", &["readonly"], Some("rust"))),
    };
    let report =
        hexwitch_core::validate::validate_scenes(&t, std::slice::from_ref(&scene)).unwrap();
    let s = resolved(&t, "variable", &["readonly"], Some("rust"));
    let expected = hexwitch_core::color::composite_over(
        s.fg.unwrap(),
        s.fg_opacity.unwrap(),
        t.base_bg().unwrap(),
    );
    assert_eq!(report.scenes[0].foreground, Some(expected));
    let mut unknown = t;
    unknown.extensions.remove("vscode.semanticHighlighting");
    assert_eq!(
        hexwitch_core::validate::validate_scenes(&unknown, &[scene])
            .unwrap()
            .status,
        hexwitch_core::validate::Status::Unverified
    );
}
#[test]
fn expanded_interactions_terminal_and_property_slots_are_correct() {
    let t = theme();
    assert_eq!(
        t.get(&Role::Interaction(InteractionRole::Selection))
            .unwrap()
            .fg,
        Some(Srgb::WHITE)
    );
    assert!(t
        .get(&Role::Interaction(InteractionRole::SearchCurrent))
        .unwrap()
        .bg_opacity
        .is_some());
    assert_eq!(
        t.get(&Role::Surface(Surface::Border)).unwrap().fg,
        Some(Srgb::from_hex("#506070").unwrap())
    );
    assert_eq!(
        t.get(&Role::Diff {
            kind: DiffKind::Removed,
            part: DiffPart::Gutter
        })
        .unwrap()
        .fg,
        Some(Srgb::from_hex("#cc5566").unwrap())
    );
    assert_eq!(
        t.fg(&Role::Terminal(4)),
        Some(Srgb::from_hex("#4080c0").unwrap())
    );
    let terminal: Role = "other.terminal-default".parse().unwrap();
    assert_eq!(t.bg(&terminal), Some(Srgb::from_hex("#081018").unwrap()));
    let report =
        hexwitch_core::validate::validate_scenes(&t, &hexwitch_core::validate::initial_scenes())
            .unwrap();
    let slot = report
        .scenes
        .iter()
        .find(|s| s.name == "terminal/4")
        .unwrap();
    assert_eq!(slot.background, t.bg(&terminal));
}
#[test]
fn generic_modifiers_still_work_without_type_metadata() {
    let mut t = Theme::new("generic", Appearance::Dark);
    let role = Role::Syntax(SyntaxRole::Variable);
    t.set(role.clone(), Style::fg(Srgb::WHITE));
    t.modifiers.push(SemanticStyle {
        role: role.clone(),
        modifiers: BTreeSet::from(["readonly".into()]),
        language: None,
        style: Style {
            italic: Some(true),
            ..Style::new()
        },
        token_type: None,
        priority: 0,
    });
    assert_eq!(
        resolved(&t, "variable", &["readonly"], None).italic,
        Some(true)
    );
}
