use hexwitch_core::color::Srgb;
use hexwitch_core::interpret_v1::{self, Decision, ModelInterpretation};
use hexwitch_core::ir::*;
use hexwitch_core::protocol::Outcome;
use serde_json::{json, Value};

fn theme() -> Theme {
    let mut t = Theme::new("interpretation", Appearance::Dark);
    t.set(Role::Surface(Surface::Base), Style::bg(Srgb::BLACK));
    t.set(
        Role::Syntax(SyntaxRole::Comment),
        Style {
            italic: Some(true),
            ..Style::fg(Srgb::new(70, 70, 70))
        },
    );
    t.set(
        Role::Syntax(SyntaxRole::Keyword),
        Style::fg(Srgb::new(120, 140, 220)),
    );
    t.set(
        Role::Interaction(InteractionRole::Selection),
        Style {
            bg_opacity: Some(0.3),
            ..Style::bg(Srgb::new(40, 60, 80))
        },
    );
    t
}
fn action(op: &str, roles: &[&str], property: &str) -> Value {
    json!({"op":op,"roles":roles,"property":property,"include_semantic":false,"scope":null,"color":null,"number":null,"boolean":null,"underline":null,"state":null,"backgrounds":null,"min":null,"direction":null,"persistent":null})
}
fn response(actions: Vec<Value>) -> ModelInterpretation {
    serde_json::from_value(json!({"version":1,"decision":"ready","notes":"A precise property edit.","questions":[],"actions":actions})).unwrap()
}
#[test]
fn opaque_background_does_not_require_color_unlock() {
    let mut t = theme();
    t.styles
        .get_mut(&Role::Surface(Surface::Base))
        .unwrap()
        .bg_opacity = Some(0.4);
    let response = interpret_v1::parse_model_output(include_str!(
        "../../../eval/llm/property-opaque-background.json"
    ))
    .unwrap();
    let interpreted = interpret_v1::validate(
        &response,
        &t,
        "Keep the background color, but remove its transparency.",
    )
    .unwrap();
    let (out, _) = hexwitch_core::intent::revise_intent(&t, &interpreted.spec.unwrap()).unwrap();
    assert_eq!(out.base_bg(), t.base_bg());
    assert_eq!(
        out.get(&Role::Surface(Surface::Base)).unwrap().bg_opacity,
        Some(1.0)
    );
    assert!(out
        .get(&Role::Surface(Surface::Base))
        .unwrap()
        .locked
        .contains(&Property::Bg));
}
#[test]
fn ambiguity_and_unsupported_are_explicit_and_non_mutating() {
    let clarified =
        interpret_v1::parse_model_output(include_str!("../../../eval/llm/clarify-blue.json"))
            .unwrap();
    let result = interpret_v1::validate(&clarified, &theme(), "Make that blue quieter.").unwrap();
    assert_eq!(result.outcome, Outcome::NeedsClarification);
    assert!(result.spec.is_none());
    let mut bad = clarified.clone();
    bad.actions =
        vec![serde_json::from_value(action("unlock", &["syntax.keyword"], "fg")).unwrap()];
    assert!(interpret_v1::validate(&bad, &theme(), "Make that blue quieter.").is_err());
    let unsupported = ModelInterpretation {
        version: 1,
        decision: Decision::Unsupported,
        notes: "Font size is an editor setting, not a theme property.".into(),
        questions: vec![],
        actions: vec![],
    };
    assert_eq!(
        interpret_v1::validate(&unsupported, &theme(), "Make the font larger.")
            .unwrap()
            .outcome,
        Outcome::Refused
    );
}
#[test]
fn user_anchors_are_admitted_but_model_and_inventory_colors_are_not() {
    let bad = interpret_v1::parse_model_output(include_str!(
        "../../../eval/llm/invalid-invented-anchor.json"
    ))
    .unwrap();
    assert!(interpret_v1::validate(&bad, &theme(), "Warmer keywords.")
        .unwrap_err()
        .contains("not explicitly supplied"));
    assert!(interpret_v1::validate(&bad, &theme(), "Use #8844cc for keywords.").is_ok());
    assert!(interpret_v1::validate(&bad, &theme(), "Use #8844ccjunk for keywords.").is_err());
    assert!(interpret_v1::trusted_colors("emoji 🧙 #abc, '#123456'")
        .contains(&Srgb::from_hex("#aabbcc").unwrap()));
}
#[test]
fn malformed_and_missing_properties_do_not_become_empty_success() {
    assert!(interpret_v1::parse_model_output("{}").is_err());
    let mut missing_field = action("unlock", &["syntax.keyword"], "fg");
    missing_field.as_object_mut().unwrap().remove("number");
    assert!(interpret_v1::parse_model_output(&json!({"version":1,"decision":"ready","notes":"unlock","questions":[],"actions":[missing_field]}).to_string()).is_err());
    assert!(interpret_v1::trusted_colors("#12345680").is_empty());
    assert!(interpret_v1::parse_model_output(r#"{/* comment */ \"version\":1}"#).is_err());
    let mut value = action("set_bool", &["syntax.comment"], "italic");
    value["boolean"] = json!(false);
    value["number"] = json!(0.1);
    assert!(
        interpret_v1::validate(&response(vec![value]), &theme(), "No comment italics.")
            .unwrap_err()
            .contains("irrelevant")
    );
    let mut missing = action("set_bool", &["syntax.nonexistent"], "italic");
    missing["boolean"] = json!(false);
    assert!(interpret_v1::validate(&response(vec![missing]), &theme(), "No italics.").is_err());
    let mut invalid = action("set_number", &["surface.base"], "bg_opacity");
    invalid["number"] = json!(2);
    assert!(interpret_v1::validate(&response(vec![invalid]), &theme(), "Opaque.").is_err());
}
#[test]
fn dynamic_contrast_constraints_follow_later_surface_changes() {
    let t = theme();
    let mut contrast = action("contrast", &["syntax.comment"], "fg");
    contrast["backgrounds"] = json!([{"surface":"surface.base","overlays":[]}]);
    contrast["min"] = json!(4.5);
    contrast["direction"] = json!("both");
    contrast["persistent"] = json!(true);
    let interpreted =
        interpret_v1::validate(&response(vec![contrast]), &t, "Make comments readable.").unwrap();
    let (first, _) = hexwitch_core::intent::revise_intent(&t, &interpreted.spec.unwrap()).unwrap();
    let mut background = action("set_color", &["surface.base"], "bg");
    background["color"] = json!(first.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap());
    let request = format!(
        "Set background to {}.",
        first
            .fg(&Role::Syntax(SyntaxRole::Comment))
            .unwrap()
            .to_hex()
    );
    let interpreted =
        interpret_v1::validate(&response(vec![background]), &first, &request).unwrap();
    assert_eq!(interpreted.outcome, Outcome::Infeasible);
    let (second, _) =
        hexwitch_core::intent::revise_intent(&first, &interpreted.spec.unwrap()).unwrap();
    assert_eq!(second, first);
}
#[test]
fn later_edits_cannot_invalidate_earlier_requested_floor() {
    let mut contrast = action("contrast", &["syntax.comment"], "fg");
    contrast["backgrounds"] = json!([{"surface":"surface.base","overlays":[]}]);
    contrast["min"] = json!(4.5);
    contrast["direction"] = json!("both");
    contrast["persistent"] = json!(false);
    let mut erase = action("set_color", &["syntax.comment"], "fg");
    erase["color"] = json!("#000000");
    let interpreted = interpret_v1::validate(
        &response(vec![contrast, erase]),
        &theme(),
        "Make comments readable but use #000000.",
    )
    .unwrap();
    assert_eq!(interpreted.outcome, Outcome::Infeasible);
}
#[test]
fn contrast_includes_semantic_patches_with_inherited_colors() {
    let mut t = theme();
    t.extensions
        .insert("vscode.semanticHighlighting".into(), json!(true));
    t.modifiers.push(SemanticStyle {
        role: Role::Syntax(SyntaxRole::Comment),
        modifiers: std::collections::BTreeSet::from(["documentation".into()]),
        language: None,
        style: Style {
            italic: Some(true),
            ..Style::new()
        },
        token_type: Some("comment".into()),
        priority: 0,
    });
    let mut contrast = action("contrast", &["syntax.comment"], "fg");
    contrast["include_semantic"] = json!(true);
    contrast["backgrounds"] = json!([{"surface":"surface.base","overlays":[]}]);
    contrast["min"] = json!(4.5);
    contrast["direction"] = json!("both");
    contrast["persistent"] = json!(false);
    let interpreted = interpret_v1::validate(
        &response(vec![contrast]),
        &t,
        "Make comments readable including documentation.",
    )
    .unwrap();
    assert_eq!(interpreted.outcome, Outcome::Satisfied);
    let (out, report) =
        hexwitch_core::intent::revise_intent(&t, &interpreted.spec.unwrap()).unwrap();
    assert!(out.modifiers[0].style.fg.is_some());
    assert_eq!(out.modifiers[0].style.italic, Some(true));
    assert!(report
        .requirements
        .iter()
        .all(|r| r.outcome == Outcome::Satisfied));
}
#[test]
fn combined_semantic_color_and_opacity_cannot_claim_independent_passes() {
    let mut t = theme();
    t.extensions
        .insert("vscode.semanticHighlighting".into(), json!(true));
    t.styles
        .get_mut(&Role::Syntax(SyntaxRole::Comment))
        .unwrap()
        .fg = Some(Srgb::WHITE);
    for (modifier, style) in [
        (
            "readonly",
            Style {
                fg: Some(Srgb::from_hex("#888888").unwrap()),
                ..Style::new()
            },
        ),
        (
            "static",
            Style {
                fg_opacity: Some(0.7),
                ..Style::new()
            },
        ),
    ] {
        t.modifiers.push(SemanticStyle {
            role: Role::Syntax(SyntaxRole::Comment),
            modifiers: std::collections::BTreeSet::from([modifier.into()]),
            language: None,
            style,
            token_type: Some("comment".into()),
            priority: 0,
        });
    }
    let spec:hexwitch_core::intent::IntentSpec=serde_json::from_value(json!({"version":1,"actions":[],"requirements":[{"role":"syntax.comment","property":"fg","backgrounds":[{"surface":"surface.base","overlays":[]}],"min":4.5,"include_semantic":true}]})).unwrap();
    let (out, report) = hexwitch_core::intent::revise_intent(&t, &spec).unwrap();
    assert_eq!(out, t);
    assert!(report.rolled_back);
    assert!(report
        .requirements
        .iter()
        .any(|r| r.outcome == Outcome::Infeasible
            && r.semantic_context
                .as_ref()
                .is_some_and(|c| c.modifiers.len() == 2)));
}
#[test]
fn request_local_locks_do_not_persist_and_unlocks_are_property_scoped() {
    let t = theme();
    let mut lock = action("lock", &["syntax.keyword"], "fg");
    lock["scope"] = json!("request");
    let mut shift = action("lightness", &["syntax.keyword"], "fg");
    shift["number"] = json!(0.04);
    let interpreted = interpret_v1::validate(
        &response(vec![lock, shift.clone()]),
        &t,
        "Keep the keyword foreground during this request.",
    )
    .unwrap();
    assert_eq!(interpreted.outcome, Outcome::Refused);
    let (first, _) = hexwitch_core::intent::revise_intent(&t, &interpreted.spec.unwrap()).unwrap();
    assert_eq!(first.styles, t.styles);
    assert_eq!(
        interpret_v1::validate(
            &response(vec![shift]),
            &first,
            "Make keywords brighter now."
        )
        .unwrap()
        .outcome,
        Outcome::Satisfied
    );
}
#[test]
fn prompt_contains_properties_locks_context_and_trusted_anchors() {
    let mut t = theme();
    t.styles
        .get_mut(&Role::Syntax(SyntaxRole::Comment))
        .unwrap()
        .locked
        .insert(Property::Italic);
    let context = interpret_v1::IterationContext {
        previous_requests: vec!["slightly darker".into()],
        previous_actions: vec![],
    };
    let prompt = interpret_v1::user_prompt(&t, "Use #123456.", &context).unwrap();
    let invalid = interpret_v1::IterationContext {
        previous_requests: vec![],
        previous_actions: vec![hexwitch_core::intent::Action::Lightness {
            target: hexwitch_core::intent::Target {
                roles: vec!["surface.base".into()],
                property: Property::Bg,
                include_semantic: false,
            },
            delta: f64::NAN,
        }],
    };
    assert!(interpret_v1::user_prompt(&t, "Again.", &invalid).is_err());
    for text in [
        "locked",
        "italic",
        "previous_requests",
        "#123456",
        "bg_opacity",
    ] {
        assert!(prompt.contains(text), "{text}");
    }
    assert!(
        interpret_v1::response_schema()["schema"]["properties"]["actions"]["items"]
            ["additionalProperties"]
            == false
    );
}
