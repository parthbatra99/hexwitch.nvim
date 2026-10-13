use hexwitch_core::color::Srgb;
use hexwitch_core::intent::{revise_intent, IntentSpec};
use hexwitch_core::ir::*;

fn theme() -> Theme {
    let mut t = Theme::new("contract", Appearance::Dark);
    let mut s = Style::pair(Srgb::WHITE, Srgb::BLACK);
    s.bg_opacity = Some(0.4);
    s.source = Some(SourceInfo {
        origin: Origin::Generated,
        reference: None,
    });
    s.locked.insert(Property::Bg);
    t.set(Role::Interaction(InteractionRole::Selection), s);
    t
}
#[test]
fn independent_color_opacity_and_lock_survive_roundtrip() {
    let t = theme();
    assert_eq!(
        Theme::from_json_str(&t.to_json_string().unwrap()).unwrap(),
        t
    );
    let spec: IntentSpec = serde_json::from_value(serde_json::json!({"version":1,"actions":[{"op":"set_number","target":{"roles":["interaction.selection"],"property":"bg_opacity"},"value":1.0}]})).unwrap();
    let (out, _) = revise_intent(&t, &spec).unwrap();
    let s = out
        .get(&Role::Interaction(InteractionRole::Selection))
        .unwrap();
    assert_eq!(s.bg, Some(Srgb::BLACK));
    assert_eq!(s.bg_opacity, Some(1.0));
    assert!(s.locked.contains(&Property::Bg));
}
#[test]
fn sequential_unlock_changes_only_requested_property() {
    let t = theme();
    let spec: IntentSpec = serde_json::from_value(serde_json::json!({"version":1,"actions":[{"op":"unlock","target":{"roles":["interaction.selection"],"property":"bg"}},{"op":"set_color","target":{"roles":["interaction.selection"],"property":"bg"},"color":"#123456"},{"op":"set_bool","target":{"roles":["interaction.selection"],"property":"italic"},"value":false}]})).unwrap();
    let (out, _) = revise_intent(&t, &spec).unwrap();
    assert_eq!(
        out.get(&Role::Interaction(InteractionRole::Selection))
            .unwrap()
            .italic,
        Some(false)
    );
    assert_eq!(
        out.fg(&Role::Interaction(InteractionRole::Selection)),
        Some(Srgb::WHITE)
    );
    assert_eq!(
        out.get(&Role::Interaction(InteractionRole::Selection))
            .unwrap()
            .bg_opacity,
        Some(0.4)
    );
}
#[test]
fn versions_and_invalid_states_fail_explicitly() {
    let t = theme();
    let mut v = serde_json::to_value(&t).unwrap();
    v["meta"]["ir_version"] = serde_json::json!(0);
    let json = v.to_string();
    assert!(Theme::from_json_str(&json).is_err());
    assert!(Theme::migrate_v0(&json).is_ok());
    v["meta"]["ir_version"] = serde_json::json!(9);
    assert!(Theme::migrate_v0(&v.to_string()).is_err());
    let mut bad = t;
    let s = bad.styles.values_mut().next().unwrap();
    s.states.insert(Property::Bg, PropertyState::Clear);
    assert!(bad.validate().is_err());
}
#[test]
fn final_required_constraints_are_transactional() {
    let t = theme();
    let spec: IntentSpec = serde_json::from_value(serde_json::json!({"version":1,"actions":[{"op":"set_color","target":{"roles":["interaction.selection"],"property":"fg"},"color":"#000000"}],"requirements":[{"role":"interaction.selection","property":"fg","backgrounds":["#000000"],"min":4.5,"persistent":true}]})).unwrap();
    let (out, report) = revise_intent(&t, &spec).unwrap();
    assert_eq!(out, t);
    assert_eq!(report.outcome, hexwitch_core::protocol::Outcome::Infeasible);
}
#[test]
fn empty_and_arbitrary_input_is_rejected() {
    assert!(hexwitch_core::interpret::parse_model_output("{}").is_err());
    assert!(
        hexwitch_core::interpret::parse_model_output(r#"{"actions":[],"invented":true}"#).is_err()
    );
    let mut s = Style::new();
    s.blend = Some(101);
    assert!(s.validate().is_err());
    s.blend = None;
    s.fg_opacity = Some(f64::NAN);
    assert!(s.validate().is_err());
}
