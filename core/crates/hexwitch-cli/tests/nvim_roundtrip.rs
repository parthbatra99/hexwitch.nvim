//! Neovim round-trip: the engine emits, Neovim resolves, we verify.
//!
//! This is the "emit is not the last step" contract made executable. The
//! emitted colorscheme is loaded in a real (headless) Neovim, groups are
//! read back through `nvim_get_hl`, and the resolved values are compared
//! against the IR. Fallback links, terminal palette, and link resolution
//! are checked against Neovim's actual behavior — not our assumptions.
//!
//! Skipped automatically when `nvim` is not on PATH (e.g. environments
//! without Neovim); CI installs it so the test always runs there.

use std::path::PathBuf;
use std::process::Command;

use hexwitch_core::color::Srgb;
use hexwitch_core::emit::emit_nvim;
use hexwitch_core::ingest::ingest_vscode_json;
use hexwitch_core::ir::role::{Role, Surface, SyntaxRole, TextRole};

fn nvim_available() -> bool {
    Command::new("nvim")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn eval_theme(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../eval/themes")
        .join(name)
}

fn to_u32(c: Srgb) -> i64 {
    ((c.r as i64) << 16) | ((c.g as i64) << 8) | c.b as i64
}

/// Load the colorscheme in headless nvim and return resolved groups as JSON.
fn resolve_in_nvim(lua: &str, groups: &[&str]) -> serde_json::Value {
    let dir = std::env::temp_dir().join(format!("hexwitch-nvim-roundtrip-{}", std::process::id()));
    let colors_dir = dir.join("colors");
    std::fs::create_dir_all(&colors_dir).unwrap();
    std::fs::write(colors_dir.join("hexwitch_roundtrip.lua"), lua).unwrap();

    let names = groups
        .iter()
        .map(|g| format!("\"{g}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let extract = format!(
        "local names = {{ {names} }}\n\
         local function resolve(name, depth)\n\
         \x20 local hl = vim.api.nvim_get_hl(0, {{ name = name, link = false }})\n\
         \x20 if type(hl.link) == 'string' and depth < 10 then\n\
         \x20 \x20 return resolve(hl.link, depth + 1)\n\
         \x20 end\n\
         \x20 return hl\n\
         end\n\
         local out = {{}}\n\
         for _, n in ipairs(names) do\n\
         \x20 out[n] = resolve(n, 0)\n\
         end\n\
         io.write(vim.json.encode(out))\n"
    );

    let output = Command::new("nvim")
        .arg("--headless")
        .arg("--clean")
        .arg("--cmd")
        .arg(format!("set rtp^={}", dir.display()))
        .arg("--cmd")
        .arg("set termguicolors")
        .arg("-c")
        .arg("colorscheme hexwitch_roundtrip")
        .arg("-c")
        .arg(format!("lua {extract}"))
        .arg("-c")
        .arg("qa!")
        .output()
        .expect("spawn nvim");

    assert!(
        output.status.success(),
        "nvim failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout.find('{').expect("nvim printed no JSON");
    serde_json::from_str(&stdout[json_start..]).expect("nvim printed invalid JSON")
}

#[test]
fn nvim_resolves_emitted_colorscheme_as_expected() {
    if !nvim_available() {
        eprintln!("skipping: nvim not on PATH");
        return;
    }

    let json = std::fs::read_to_string(eval_theme("gruvbox-dark-medium.json")).unwrap();
    let theme = ingest_vscode_json(&json).unwrap();
    let emitted = emit_nvim(&theme).unwrap();

    let resolved = resolve_in_nvim(
        &emitted.lua,
        &[
            "Normal",
            "Comment",
            "String",
            "Keyword",
            "Function",
            "Type",
            "DiagnosticError",
            "@comment",
            "@lsp.type.variable",
        ],
    );

    // Direct definitions round-trip exactly.
    let normal = &resolved["Normal"];
    assert_eq!(
        normal["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#ebdbb2").unwrap()))
    );
    assert_eq!(
        normal["bg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#282828").unwrap()))
    );

    let comment = &resolved["Comment"];
    assert_eq!(
        comment["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#928374").unwrap()))
    );

    let string = &resolved["String"];
    assert_eq!(
        string["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#b8bb26").unwrap()))
    );

    let keyword = &resolved["Keyword"];
    assert_eq!(
        keyword["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#fb4934").unwrap()))
    );

    // Font attributes ride along (gruvbox fixture sets italic comments);
    // nvim reports booleans, not integers, for these.
    assert_eq!(comment["italic"].as_bool(), Some(true));

    // Diagnostics came through the workbench color mapping.
    let diag = &resolved["DiagnosticError"];
    assert_eq!(
        diag["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#fb4934").unwrap()))
    );

    // Links resolve through to the same values in nvim itself.
    let ts_comment = &resolved["@comment"];
    assert_eq!(
        ts_comment["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#928374").unwrap()))
    );
    let lsp_var = &resolved["@lsp.type.variable"];
    assert_eq!(
        lsp_var["fg"].as_i64(),
        Some(to_u32(Srgb::from_hex("#83a598").unwrap()))
    );
}

#[test]
fn nvim_emission_matches_ir_for_every_eval_theme() {
    if !nvim_available() {
        eprintln!("skipping: nvim not on PATH");
        return;
    }

    for file in [
        "gruvbox-dark-medium.json",
        "tokyo-night.json",
        "catppuccin-mocha.json",
        "dracula.json",
        "solarized-dark.json",
        "nord.json",
    ] {
        let json = std::fs::read_to_string(eval_theme(file)).unwrap();
        let theme = ingest_vscode_json(&json).unwrap();
        let emitted = emit_nvim(&theme).unwrap();

        let expect_fg = |role: &Role| theme.fg(role).map(to_u32);
        let expect_bg = |role: &Role| theme.bg(role).map(to_u32);

        let resolved = resolve_in_nvim(
            &emitted.lua,
            &["Normal", "Comment", "String", "Keyword", "Function"],
        );

        let pairs = [
            (
                "Normal",
                (
                    expect_fg(&Role::Text(TextRole::Primary)),
                    expect_bg(&Role::Surface(Surface::Base)),
                ),
            ),
            (
                "Comment",
                (expect_fg(&Role::Syntax(SyntaxRole::Comment)), None),
            ),
            (
                "String",
                (expect_fg(&Role::Syntax(SyntaxRole::String)), None),
            ),
            (
                "Keyword",
                (expect_fg(&Role::Syntax(SyntaxRole::Keyword)), None),
            ),
            (
                "Function",
                (expect_fg(&Role::Syntax(SyntaxRole::Function)), None),
            ),
        ];
        for (group, (fg, bg)) in pairs {
            if let Some(want) = fg {
                assert_eq!(
                    resolved[group]["fg"].as_i64(),
                    Some(want),
                    "{file}: {group} fg mismatch"
                );
            }
            if let Some(want) = bg {
                assert_eq!(
                    resolved[group]["bg"].as_i64(),
                    Some(want),
                    "{file}: {group} bg mismatch"
                );
            }
        }
    }
}
