use hexwitch_core::color::contrast_ratio;
use hexwitch_core::ingest::load_theme;
use hexwitch_core::ir::role::{Role, Surface, SyntaxRole, TextRole};
use hexwitch_core::revise::{revise, EditSpec};

fn eval_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../eval")
        .join(rel)
}

#[test]
fn gruvbox_darker_base_readable_comments_syntax_untouched() {
    let original =
        load_theme(&std::fs::read_to_string(eval_path("themes/gruvbox-dark-medium.json")).unwrap())
            .unwrap();
    let spec = EditSpec::from_json(
        &std::fs::read_to_string(eval_path("editspecs/gruvbox-focus.json")).unwrap(),
    )
    .unwrap();

    let (revised, report) = revise(&original, &spec).unwrap();

    for role in [
        Role::Syntax(SyntaxRole::Keyword),
        Role::Syntax(SyntaxRole::String),
        Role::Syntax(SyntaxRole::Function),
        Role::Syntax(SyntaxRole::Type),
        Role::Syntax(SyntaxRole::Constant),
        Role::Syntax(SyntaxRole::Variable),
        Role::Text(TextRole::Primary),
    ] {
        assert_eq!(
            revised.fg(&role),
            original.fg(&role),
            "{role} was supposed to be protected"
        );
    }
    assert!(report.refused.is_empty(), "nothing should be refused here");

    let (old_base, new_base) = (original.base_bg().unwrap(), revised.base_bg().unwrap());
    assert!(
        new_base.relative_luminance() < old_base.relative_luminance(),
        "base should darken"
    );
    let (a, b) = (old_base.to_oklch(), new_base.to_oklch());
    assert!(
        (a.h - b.h).abs() < 0.5,
        "base hue drifted {} -> {}",
        a.h,
        b.h
    );

    let comment = revised.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap();
    let ratio = contrast_ratio(comment, new_base);
    assert!(ratio >= 4.5, "comment contrast {ratio} < 4.5");
    let (ca, cb) = (
        original
            .fg(&Role::Syntax(SyntaxRole::Comment))
            .unwrap()
            .to_oklch(),
        comment.to_oklch(),
    );
    assert!((ca.h - cb.h).abs() < 0.5, "comment hue drifted");

    let mut moved = std::collections::BTreeSet::new();
    for a in &report.applied {
        moved.insert(a.role.clone());
    }
    assert_eq!(
        moved,
        std::collections::BTreeSet::from([
            "surface.base".to_string(),
            "surface.panel".to_string(),
            "surface.float".to_string(),
        ]),
        "unexpected roles moved: {moved:?}"
    );
    for (role, style) in &original.styles {
        if moved.contains(&role.to_string()) {
            continue;
        }
        let now = revised.get(role).unwrap();
        assert_eq!(now.fg, style.fg, "{role} fg changed without being asked");
        assert_eq!(now.bg, style.bg, "{role} bg changed without being asked");
        assert_eq!(now.italic, style.italic);
        assert_eq!(now.bold, style.bold);
        assert_eq!(now.underline, style.underline);
    }
    assert_eq!(
        revised
            .get(&Role::Syntax(SyntaxRole::Keyword))
            .unwrap()
            .provenance,
        hexwitch_core::ir::style::Provenance::Locked
    );

    assert!(
        revised.base_bg().unwrap().relative_luminance()
            <= revised
                .bg(&Role::Surface(Surface::Float))
                .unwrap()
                .relative_luminance()
    );

    let fp = hexwitch_core::analyze::fingerprint(&revised).unwrap();
    assert!(fp.primary_contrast().unwrap() > 11.0);
    assert!(fp.comment_contrast().unwrap() >= 4.5);
}

#[test]
fn nord_comment_repair_requires_actual_movement() {
    let original =
        load_theme(&std::fs::read_to_string(eval_path("themes/nord.json")).unwrap()).unwrap();
    let spec = EditSpec::from_json(
        r##"{ "changes": [ { "target": "syntax.comment",
              "op": "min_contrast", "against": "surface.base", "min": 4.5 } ] }"##,
    )
    .unwrap();
    let (revised, report) = revise(&original, &spec).unwrap();

    let comment = revised.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap();
    let base = revised.base_bg().unwrap();
    assert!(contrast_ratio(comment, base) >= 4.5);
    assert_eq!(report.changed_roles(), 1, "only the comment should move");
    assert_ne!(
        comment,
        original.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap()
    );

    let (a, b) = (
        original
            .fg(&Role::Syntax(SyntaxRole::Comment))
            .unwrap()
            .to_oklch(),
        comment.to_oklch(),
    );
    assert!(
        (a.h - b.h).abs() < 1.0,
        "comment hue drifted {} -> {}",
        a.h,
        b.h
    );

    for (role, style) in &original.styles {
        if role != &Role::Syntax(SyntaxRole::Comment) {
            assert_eq!(revised.get(role), Some(style), "{role} changed");
        }
    }
}

#[test]
fn revised_theme_round_trips_through_ir_json() {
    let original =
        load_theme(&std::fs::read_to_string(eval_path("themes/tokyo-night.json")).unwrap())
            .unwrap();
    let spec = EditSpec::from_json(
        r##"{ "changes": [ { "target": "syntax.comment",
              "op": "min_contrast", "against": "surface.base", "min": 4.5 } ] }"##,
    )
    .unwrap();
    let (revised, _) = revise(&original, &spec).unwrap();
    let json = revised.to_json_string().unwrap();
    let back = load_theme(&json).unwrap();
    assert_eq!(back, revised);
}
