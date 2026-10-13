#[test]
fn semantic_corpus_is_balanced_unique_and_held_out() {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("../../../eval/corpus-v1.json")).unwrap();
    assert_eq!(value["version"], 1);
    assert_eq!(value["suites"].as_array().unwrap().len(), 7);
    let cases = value["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 50);
    let mut ids = std::collections::BTreeSet::new();
    let mut held = 0;
    for case in cases {
        assert!(ids.insert(case["id"].as_str().unwrap()));
        for field in ["required", "forbidden", "acceptable"] {
            assert!(!case[field].as_array().unwrap().is_empty());
        }
        assert!(!case["request"].as_str().unwrap().trim().is_empty());
        if case["split"] == "held_out" {
            held += 1;
        }
    }
    assert_eq!(held, 20);
}
