use hexwitch_core::{
    experiment::{Bundle, Experiment},
    ingest::load_theme,
    intent::IntentSpec,
    interpret_v1::{parse_model_output, trusted_colors},
    ir::{Property, Role},
};
use serde_json::{json, Value};
fn session() -> Experiment {
    Experiment::new(load_theme(include_str!("../../../eval/states/dark.json")).unwrap()).unwrap()
}
fn intent(value: Value) -> IntentSpec {
    serde_json::from_value(value).unwrap()
}
fn darker() -> IntentSpec {
    intent(
        json!({"version":1,"actions":[{"op":"lightness","target":{"roles":["surface.base"],"property":"bg"},"delta":-0.02}]}),
    )
}
fn model(op: &str, role: &str, property: &str, field: &str, value: Value) -> Value {
    let mut action = json!({"op":op,"roles":[role],"property":property,"include_semantic":false,"scope":null,"color":null,"number":null,"boolean":null,"underline":null,"state":null,"backgrounds":null,"min":null,"direction":null,"persistent":null});
    action[field] = value;
    json!({"version":1,"decision":"ready","notes":"Change only the requested property.","questions":[],"actions":[action]})
}
fn clarification() -> hexwitch_core::interpret_v1::ModelInterpretation {
    parse_model_output(&json!({"version":1,"decision":"needs_clarification","notes":"Which target?","questions":["Did you mean comments #112233 or keywords?"],"actions":[]}).to_string()).unwrap()
}
#[test]
fn review_is_non_mutating_and_darker_preserves_opacity_and_every_other_slot() {
    let mut s = session();
    let before = s.current().clone();
    let response = model("lightness", "surface.base", "bg", "number", json!(-0.02));
    s.interpret(
        "Make the background darker".into(),
        parse_model_output(&response.to_string()).unwrap(),
        0,
    )
    .unwrap();
    assert_eq!(json!(s.current()), json!(before));
    assert!(s.clarification.is_none());
    let candidate = &s.candidate.as_ref().unwrap().theme;
    let base: Role = "surface.base".parse().unwrap();
    assert!(candidate.bg(&base).unwrap().to_oklch().l < before.bg(&base).unwrap().to_oklch().l);
    assert_eq!(
        candidate.get(&base).unwrap().bg_opacity,
        before.get(&base).unwrap().bg_opacity
    );
    for (role, style) in &before.styles {
        if role != &base {
            assert_eq!(candidate.get(role).unwrap(), style);
        }
    }
    assert!(s.accept(1, false).is_err());
    s.accept(1, true).unwrap();
    assert_eq!(s.versions.len(), 2);
    assert!(s.check_revision(1).is_err());
}
#[test]
fn clarification_is_resumable_user_only_and_invalidated_by_reference_change() {
    let mut s = session();
    s.interpret("Make blue less intense".into(), clarification(), 0)
        .unwrap();
    let request = s
        .answer_request("Comments, keep everything else.", 1)
        .unwrap();
    assert!(trusted_colors(&request).is_empty());
    let malicious = model(
        "set_color",
        "syntax.comment",
        "fg",
        "color",
        json!("#112233"),
    );
    assert!(s
        .interpret(
            request.clone(),
            parse_model_output(&malicious.to_string()).unwrap(),
            1
        )
        .is_err());
    assert_eq!(s.revision, 1);
    assert!(s.clarification.is_some());
    let good = model("chroma", "syntax.comment", "fg", "number", json!(-0.02));
    s.interpret(request, parse_model_output(&good.to_string()).unwrap(), 1)
        .unwrap();
    assert!(s.clarification.is_none());
    assert!(s.candidate.is_some());
    s.import(
        load_theme(include_str!("../../../eval/states/light.json")).unwrap(),
        2,
    )
    .unwrap();
    assert!(s.answer_request("Comments", 1).is_err());
    assert!(s.answer_request("Comments", 3).is_err());
}
#[test]
fn undo_redo_and_new_branch_use_only_active_context() {
    let mut s = session();
    s.propose(darker(), "Darker once".into(), 0).unwrap();
    s.accept(1, true).unwrap();
    s.propose(darker(), "Darker twice".into(), 2).unwrap();
    s.accept(3, true).unwrap();
    s.navigate(false, 4).unwrap();
    assert_eq!(s.context().previous_requests, vec!["Darker once"]);
    s.navigate(true, 5).unwrap();
    assert_eq!(s.context().previous_requests.len(), 2);
    s.navigate(false, 6).unwrap();
    s.propose(darker(), "New branch".into(), 7).unwrap();
    s.accept(8, true).unwrap();
    assert_eq!(
        s.context().previous_requests,
        vec!["Darker once", "New branch"]
    );
    assert!(s.navigate(true, 9).is_err());
}
#[test]
fn rejected_candidates_do_not_influence_history_or_context() {
    let mut s = session();
    s.propose(darker(), "Never accepted".into(), 0).unwrap();
    assert!(s.propose(darker(), "Overlapping".into(), 1).is_err());
    assert!(s.navigate(false, 1).is_err());
    s.reject(1).unwrap();
    assert!(s.context().previous_requests.is_empty());
    assert_eq!(s.versions.len(), 1);
}
#[test]
fn persistent_locks_refuse_without_an_acceptable_noop_and_unlock_is_scoped() {
    let mut s = session();
    s.propose(intent(json!({"version":1,"actions":[{"op":"lock","target":{"roles":["surface.base"],"property":"bg"},"scope":"persistent"}]})),"Protect base".into(),0).unwrap();
    s.accept(1, true).unwrap();
    s.propose(darker(), "Refused".into(), 2).unwrap();
    assert!(s.candidate.is_none());
    assert_eq!(s.last_result.as_ref().unwrap()["outcome"], "refused");
    assert!(s.accept(3, true).is_err());
    s.propose(intent(json!({"version":1,"actions":[{"op":"unlock","target":{"roles":["surface.base"],"property":"bg"}}]})),"Unlock base".into(),3).unwrap();
    let base: Role = "surface.base".parse().unwrap();
    assert!(!s
        .candidate
        .as_ref()
        .unwrap()
        .theme
        .get(&base)
        .unwrap()
        .is_locked(Property::Bg));
}
#[test]
fn infeasible_required_floor_rolls_back_and_exposes_result() {
    let mut s = session();
    let before = json!(s.current());
    s.propose(intent(json!({"version":1,"actions":[],"requirements":[{"role":"syntax.comment","property":"fg","backgrounds":[{"surface":"surface.base","overlays":[]}],"min":21}]})),"Impossible floor".into(),0).unwrap();
    assert!(s.candidate.is_none());
    assert_eq!(
        s.last_result.as_ref().unwrap()["report"]["rolled_back"],
        true
    );
    assert_eq!(json!(s.current()), before);
}
#[test]
fn export_restores_pending_candidates_but_recomputes_untrusted_reports() {
    let mut s = session();
    s.propose(darker(), "Darker".into(), 0).unwrap();
    let mut bundle = s.bundle();
    bundle.candidate.as_mut().unwrap().validation = json!({"status":"pass"});
    let bytes = serde_json::to_string(&bundle).unwrap();
    let mut other = session();
    other
        .restore(serde_json::from_str(&bytes).unwrap(), 0)
        .unwrap();
    assert!(other.candidate.is_some());
    assert!(other.accept(1, false).is_err());
    other.accept(1, true).unwrap();
    assert_eq!(other.context().previous_requests, vec!["Darker"]);
    let mut wrong_lineage = other.bundle();
    wrong_lineage.versions[1]
        .spec
        .as_mut()
        .unwrap()
        .actions
        .clear();
    assert!(other.restore(wrong_lineage, 2).is_err());
    let mut changed: Bundle = serde_json::from_str(&bytes).unwrap();
    changed.versions[0].theme.meta.name = "tampered".into();
    assert!(other.restore(changed, 2).is_err());
    let mut forged: Bundle = serde_json::from_str(&bytes).unwrap();
    forged.candidate.as_mut().unwrap().theme.meta.name = "tampered".into();
    assert!(other.restore(forged, 2).is_err());
}
#[test]
fn export_restores_pending_clarification_and_rebases_revision() {
    let mut s = session();
    s.interpret("Blue?".into(), clarification(), 0).unwrap();
    let mut other = session();
    other.import(s.current().clone(), 0).unwrap();
    other.restore(s.bundle(), 1).unwrap();
    assert_eq!(other.clarification.as_ref().unwrap().revision, 2);
    assert!(other.answer_request("Comments", 2).is_ok());
}
#[test]
fn semantic_and_lock_only_changes_are_visible_in_property_diff() {
    let mut s = session();
    s.propose(intent(json!({"version":1,"actions":[{"op":"set_bool","target":{"roles":["*"],"property":"italic","include_semantic":true},"value":false}]})),"No italics".into(),0).unwrap();
    let view = s.view().unwrap();
    assert!(view["changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| !c["semantic_rule"].is_null()));
    s.reject(1).unwrap();
    s.propose(intent(json!({"version":1,"actions":[{"op":"lock","target":{"roles":["surface.base"],"property":"bg"}}]})),"Lock".into(),2).unwrap();
    assert_eq!(s.view().unwrap()["changes"][0]["locked_after"], true);
}
