//! The thesis demo, executable: "gruvbox, darker background, readable
//! comments, don't touch the syntax" must produce a theme that still *is*
//! gruvbox — protected roles bit-identical, comments repaired, and the
//! result loadable in real Neovim with exactly the revised colors.

use std::process::Command;

use hexwitch_core::color::{contrast_ratio, Srgb};
use hexwitch_core::emit::emit_nvim;
use hexwitch_core::ingest::load_theme;
use hexwitch_core::ir::role::{Role, Surface, SyntaxRole, TextRole};
use hexwitch_core::revise::{revise, EditSpec};

fn eval_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../eval")
        .join(rel)
}

fn nvim_available() -> bool {
    Command::new("nvim")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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

    // --- protection: all syntax accents bit-identical ---
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

    // --- base actually got darker ---
    let (old_base, new_base) = (original.base_bg().unwrap(), revised.base_bg().unwrap());
    assert!(
        new_base.relative_luminance() < old_base.relative_luminance(),
        "base should darken"
    );
    // Identity: darker is an OKLCH-L move; hue must not drift.
    let (a, b) = (old_base.to_oklch(), new_base.to_oklch());
    assert!(
        (a.h - b.h).abs() < 0.5,
        "base hue drifted {} -> {}",
        a.h,
        b.h
    );

    // --- comments now readable against the revised base ---
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

    // --- minimal diff: only the surfaces moved. The comment solve turned
    // out to be a no-op: darkening the base already lifted the comment
    // from 4.02:1 past 4.5:1, and minimal-diff does not move what is
    // already satisfied. ---
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
    // Everything else keeps its exact colors. Protected roles additionally
    // carry a `Locked` provenance stamp — that's the protection record,
    // not a visual change.
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

    // --- surfaces keep their ladder ordering ---
    assert!(
        revised.base_bg().unwrap().relative_luminance()
            <= revised
                .bg(&Role::Surface(Surface::Float))
                .unwrap()
                .relative_luminance()
    );

    // --- the revised theme still analyzes coherently ---
    let fp = hexwitch_core::analyze::fingerprint(&revised).unwrap();
    assert!(fp.primary_contrast().unwrap() > 11.0); // darker base raised it
    assert!(fp.comment_contrast().unwrap() >= 4.5);

    // --- end-to-end: emit and load in real Neovim ---
    if nvim_available() {
        let emitted = emit_nvim(&revised).unwrap();
        let dir = std::env::temp_dir().join(format!("hexwitch-revise-{}", std::process::id()));
        let colors = dir.join("colors");
        std::fs::create_dir_all(&colors).unwrap();
        std::fs::write(colors.join("hexwitch_revise_demo.lua"), &emitted.lua).unwrap();

        let extract = "local out = {}\n\
             \x20 for _, n in ipairs({\"Comment\", \"Keyword\", \"Normal\"}) do\n\
             \x20   local hl = vim.api.nvim_get_hl(0, { name = n, link = false })\n\
             \x20   out[n] = hl\n\
             \x20 end\n\
             \x20 io.write(vim.json.encode(out))";
        let output = Command::new("nvim")
            .arg("--headless")
            .arg("--clean")
            .arg("--cmd")
            .arg(format!("set rtp^={}", dir.display()))
            .arg("--cmd")
            .arg("set termguicolors")
            .arg("-c")
            .arg("colorscheme hexwitch_revise_demo")
            .arg("-c")
            .arg(format!("lua {extract}"))
            .arg("-c")
            .arg("qa!")
            .output()
            .unwrap();
        assert!(output.status.success(), "nvim failed to load emitted file");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json = &stdout[stdout.find('{').unwrap()..];
        let resolved: serde_json::Value = serde_json::from_str(json).unwrap();

        let to_u32 = |c: Srgb| ((c.r as i64) << 16) | ((c.g as i64) << 8) | c.b as i64;
        assert_eq!(
            resolved["Comment"]["fg"].as_i64(),
            Some(to_u32(comment)),
            "revised comment color did not survive emission"
        );
        assert_eq!(
            resolved["Keyword"]["fg"].as_i64(),
            Some(to_u32(
                original.fg(&Role::Syntax(SyntaxRole::Keyword)).unwrap()
            )),
            "protected keyword color did not survive emission"
        );
        assert_eq!(
            resolved["Normal"]["bg"].as_i64(),
            Some(to_u32(new_base)),
            "revised base did not survive emission"
        );
    } else {
        eprintln!("skipping nvim load check: nvim not on PATH");
    }
}

#[test]
fn nord_comment_repair_requires_actual_movement() {
    // Nord's comment sits at 2.44:1 — repairing it must actually move the
    // color (unlike gruvbox, where a darker base fixes comments for free),
    // while every other role stays bit-identical.
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

    // Hue identity survives the repair.
    let (a, b) = (
        original
            .fg(&Role::Syntax(SyntaxRole::Comment))
            .unwrap()
            .to_oklch(),
        comment.to_oklch(),
    );
    assert!(
        (a.h - b.h).abs() < 0.5,
        "comment hue drifted {} -> {}",
        a.h,
        b.h
    );

    // Everything else bit-identical.
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
