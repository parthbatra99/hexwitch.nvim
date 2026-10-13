use hexwitch_core::{
    ingest::load_theme,
    intent::revise_intent,
    interpret_v1::{parse_model_output, response_schema},
};
use serde_json::{json, Value};
fn response(property: &str, op: &str, number: f64) -> Value {
    json!({"version":1,"decision":"ready","notes":"Reconstructed failure: lower keyword saturation.","questions":[],"actions":[{"op":op,"roles":["syntax.keyword"],"property":property,"include_semantic":true,"scope":null,"color":null,"number":number,"boolean":null,"underline":null,"state":null,"backgrounds":null,"min":null,"direction":null,"persistent":null}]})
}
#[test]
fn strict_provider_schema_bounds_deltas_and_separates_opacity_blend() {
    let schema = response_schema();
    let branches = schema["schema"]["properties"]["actions"]["items"]["anyOf"]
        .as_array()
        .unwrap();
    for op in ["chroma", "lightness"] {
        let branch = branches
            .iter()
            .find(|b| b["properties"]["op"]["enum"][0] == op)
            .unwrap();
        assert_eq!(branch["properties"]["number"]["minimum"], -1);
        assert_eq!(branch["properties"]["number"]["maximum"], 1);
        assert_eq!(
            branch["properties"]["property"]["enum"],
            json!(["fg", "bg", "special"])
        );
    }
    let number: Vec<_> = branches
        .iter()
        .filter(|b| b["properties"]["op"]["enum"][0] == "set_number")
        .collect();
    assert_eq!(number.len(), 2);
    assert!(number
        .iter()
        .any(|b| b["properties"]["number"]["maximum"] == 1));
    assert!(number
        .iter()
        .any(|b| b["properties"]["number"]["type"] == "integer"
            && b["properties"]["property"]["enum"] == json!(["blend"])));
}
#[test]
fn absent_background_and_out_of_range_delta_are_not_color_edit_success() {
    let theme = load_theme(include_str!("../../../eval/states/dark.json")).unwrap();
    let request =
        "Make syntax.keyword fg slightly less saturated, preserving opacity and typography.";
    for bad in [
        response("bg", "lightness", -0.02),
        response("fg", "chroma", -2.0),
    ] {
        let model = parse_model_output(&bad.to_string()).unwrap();
        assert!(hexwitch_core::interpret_v1::validate(&model, &theme, request).is_err());
    }
}
#[test]
fn schema_valid_saturation_as_opacity_fails_preservation_and_target_checks() {
    let temp = std::env::temp_dir().join(format!("hexwitch-live-negative-{}", std::process::id()));
    std::fs::create_dir_all(&temp).unwrap();
    let suite = json!({"version":1,"origin":"Authored regression for live saturation/opacity confusion","license":"MIT","revision":"playground-regression-1","purpose":"Prove wrong-but-valid opacity proposals fail semantic evaluation","limitations":["Reconstructed responses, not model accuracy evidence"],"cases":[{"id":"saturation","split":"development","category":"property_confusion","request":"Make syntax.keyword fg slightly less saturated, preserving opacity and typography.","theme":"theme.json","checks":[{"check":"chroma","target":{"roles":["syntax.keyword"],"property":"fg"},"min_delta":-0.04,"max_delta":-0.005},{"check":"preserved","target":{"roles":["syntax.keyword"],"property":"fg_opacity","include_semantic":true}},{"check":"preserved","target":{"roles":["syntax.keyword"],"property":"special_opacity","include_semantic":true}},{"check":"only_changed","allowed":[{"roles":["syntax.keyword"],"property":"fg","include_semantic":true}]}]}]});
    std::fs::write(
        temp.join("theme.json"),
        include_str!("../../../eval/states/dark.json"),
    )
    .unwrap();
    std::fs::write(temp.join("suite.json"), suite.to_string()).unwrap();
    for bad in [
        response("fg_opacity", "set_number", 0.0),
        response("special_opacity", "set_number", 0.98),
    ] {
        let theme = load_theme(include_str!("../../../eval/states/dark.json")).unwrap();
        let model = parse_model_output(&bad.to_string()).unwrap();
        let parsed = hexwitch_core::interpret_v1::validate(
            &model,
            &theme,
            "Make syntax.keyword fg slightly less saturated, preserving opacity and typography.",
        )
        .unwrap();
        assert!(
            !revise_intent(&theme, &parsed.spec.unwrap())
                .unwrap()
                .1
                .rolled_back
        );
        let run = json!({"version":1,"label":"Reconstructed wrong-property sensitivity probe","provider":"none","model":"authored-reconstruction","prompt_version":"property-intent-1","kind":"canned_replay","responses":[{"case_id":"saturation","response":bad}]});
        std::fs::write(temp.join("run.json"), run.to_string()).unwrap();
        let report = hexwitch_core::interpret_eval::run_suite(
            &temp.join("suite.json"),
            &temp.join("run.json"),
        )
        .unwrap();
        assert_eq!(report.failed, 1);
        assert_eq!(
            report.cases[0].failure_class.as_deref(),
            Some("semantic_outcome")
        );
    }
    for name in ["theme.json", "suite.json", "run.json"] {
        std::fs::remove_file(temp.join(name)).unwrap();
    }
    std::fs::remove_dir(temp).unwrap();
}
