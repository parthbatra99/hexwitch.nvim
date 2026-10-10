//! `hexwitch` CLI — thin, composable surface over hexwitch-core.
//!
//! M0 scope: color inspection primitives (`convert`, `contrast`) so the
//! engine's math is dogfoodable from day one. Ingest/analyze/revise/emit
//! subcommands arrive with their milestones and must stay scriptable —
//! the Neovim adapter (and later VS Code/Zed) drive this same binary.

use clap::{Parser, Subcommand};
use hexwitch_core::color::{contrast_verdict, gamut_map, Oklch, Srgb};
use hexwitch_core::eval::{render_report, run_eval_dir};

#[derive(Parser)]
#[command(
    name = "hexwitch",
    version,
    about = "Theme engine: reference in, identity-preserving themes out",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert a color between representations.
    Convert {
        /// Color as #rrggbb.
        color: String,
        /// Target representation.
        #[arg(short, long, value_enum, default_value_t = Target::Oklch)]
        to: Target,
    },
    /// WCAG contrast between two colors, with verdicts.
    Contrast {
        /// Foreground as #rrggbb.
        foreground: String,
        /// Background as #rrggbb.
        background: String,
        /// Emit machine-readable JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Gamut-map an OKLCH color into sRGB (chroma reduction, hue kept).
    Map {
        /// OKLCH lightness, 0-1.
        l: f64,
        /// OKLCH chroma, >= 0.
        c: f64,
        /// OKLCH hue in degrees.
        h: f64,
    },
    /// Ingest a VS Code-format theme JSON and print its fingerprint
    /// (contrast ladder, accent budget).
    Analyze {
        /// Path to a VS Code theme .json file.
        path: std::path::PathBuf,
        /// Emit the full fingerprint as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Emit an editor artifact from a theme JSON file.
    Emit {
        /// Path to a theme .json file (VS Code format or IR format).
        path: std::path::PathBuf,
        /// Target editor.
        #[arg(short, long, value_enum, default_value_t = Editor::Nvim)]
        target: Editor,
        /// Write to this path instead of stdout.
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        /// Print the emit report to stderr.
        #[arg(long)]
        report: bool,
    },
    /// Run the eval set: ingest + analyze reference themes, compare
    /// against goldens and invariants. Non-zero exit on failure.
    Eval {
        /// Eval directory containing manifest.json. Defaults to `eval`
        /// relative to the current directory (the workspace root is `core/`).
        #[arg(default_value = "eval")]
        dir: std::path::PathBuf,
    },
    /// Apply an edit spec to a theme: protected roles survive, changes
    /// apply minimally, everything else stays bit-identical.
    Revise {
        /// Path to a theme .json file (IR or VS Code format).
        path: std::path::PathBuf,
        /// Path to an edit-spec .json file.
        #[arg(short, long)]
        spec: std::path::PathBuf,
        /// Write the revised theme (IR JSON) here instead of stdout.
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        /// Also emit a Neovim colorscheme to this path.
        #[arg(long)]
        emit_nvim: Option<std::path::PathBuf>,
        /// Print the revision report even on success.
        #[arg(long)]
        report: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Target {
    Oklch,
    Oklab,
    Srgb,
    Hex,
}

/// Editor emission targets (grows with M4: VS Code, Zed).
#[derive(clap::ValueEnum, Clone, Copy)]
enum Editor {
    Nvim,
}

fn parse_color(s: &str) -> Result<Srgb, String> {
    Srgb::from_hex(s).map_err(|e| e.to_string())
}

fn summarize_report(report: &hexwitch_core::emit::EmitReport) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "emit report: {} defined, {} linked, {} skipped, {} unconsumed IR role(s)\n",
        report.defined.len(),
        report.linked.len(),
        report.skipped.len(),
        report.unconsumed_roles.len()
    ));
    for (group, role) in report.skipped.iter().take(10) {
        s.push_str(&format!("  skipped {group}: IR lacks {role}\n"));
    }
    for role in report.unconsumed_roles.iter().take(10) {
        s.push_str(&format!(
            "  unconsumed: {role} (no mapping in this target yet)\n"
        ));
    }
    s
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli.command) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Convert { color, to } => {
            let c = parse_color(&color)?;
            match to {
                Target::Oklch => {
                    let v = c.to_oklch();
                    println!("oklch({:.5} {:.5} {:.2})", v.l, v.c, v.h);
                }
                Target::Oklab => {
                    let v = c.to_oklab();
                    println!("oklab({:.5} {:.5} {:.5})", v.l, v.a, v.b);
                }
                Target::Srgb => {
                    let lin = c.to_linear();
                    println!("linear-srgb({:.5} {:.5} {:.5})", lin.r, lin.g, lin.b);
                }
                Target::Hex => println!("{}", c.to_hex()),
            }
        }
        Command::Contrast {
            foreground,
            background,
            json,
        } => {
            let fg = parse_color(&foreground)?;
            let bg = parse_color(&background)?;
            let (ratio, verdict) = contrast_verdict(fg, bg);
            if json {
                let out = serde_json::json!({
                    "foreground": fg.to_hex(),
                    "background": bg.to_hex(),
                    "ratio": (ratio * 10000.0).round() / 10000.0,
                    "verdict": verdict.label(),
                    "wcag": { "aa_normal": ratio >= 4.5, "aaa_normal": ratio >= 7.0, "aa_large": ratio >= 3.0 },
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!("{fg} on {bg}: {ratio:.2}:1  ({})", verdict.label());
            }
        }
        Command::Map { l, c, h } => {
            let mapped = gamut_map(Oklch::new(l, c, h));
            let back = mapped.to_oklch();
            println!(
                "{}  (oklch l={:.4} c={:.4} h={:.2}; chroma reduced by {:.1}%)",
                mapped.to_hex(),
                back.l,
                back.c,
                back.h,
                (1.0 - back.c / c.max(f64::EPSILON)) * 100.0
            );
        }
        Command::Analyze { path, json } => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let theme = hexwitch_core::ingest::load_theme(&source).map_err(|e| e.to_string())?;
            let fp = hexwitch_core::analyze::fingerprint(&theme).map_err(|e| e.to_string())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&fp).unwrap());
            } else {
                println!(
                    "{} — {} — base {} — {} syntax roles",
                    fp.name,
                    match fp.appearance {
                        hexwitch_core::ir::theme::Appearance::Dark => "dark",
                        hexwitch_core::ir::theme::Appearance::Light => "light",
                    },
                    fp.base_bg,
                    fp.syntax_roles
                );
                println!("contrast ladder (vs base):");
                for rung in fp.contrast_ladder.iter().take(12) {
                    println!("  {:<32} {:>6.2}:1", rung.role, rung.ratio);
                }
                println!(
                    "accents: max chroma {:.3}, lightness spread {:.3}",
                    fp.max_accent_chroma, fp.accent_lightness_spread
                );
                for accent in fp.accents.iter().take(8) {
                    println!(
                        "  {:<32} l={:.3} c={:.3} h={:6.1}",
                        accent.role, accent.l, accent.c, accent.h
                    );
                }
            }
        }
        Command::Emit {
            path,
            target,
            out,
            report,
        } => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let theme = hexwitch_core::ingest::load_theme(&source).map_err(|e| e.to_string())?;
            let emitted = match target {
                Editor::Nvim => {
                    hexwitch_core::emit::emit_nvim(&theme).map_err(|e| e.to_string())?
                }
            };
            match out {
                Some(p) => {
                    if let Some(parent) = p.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
                    }
                    std::fs::write(&p, emitted.lua)
                        .map_err(|e| format!("cannot write {}: {e}", p.display()))?;
                    if report {
                        eprintln!("{}", summarize_report(&emitted.report));
                    }
                }
                None => {
                    print!("{}", emitted.lua);
                    if report {
                        eprintln!("{}", summarize_report(&emitted.report));
                    }
                }
            }
        }
        Command::Eval { dir } => {
            let outcomes = run_eval_dir(&dir).map_err(|e| format!("eval failed: {e}"))?;
            print!("{}", render_report(&outcomes));
            if outcomes.iter().any(|o| !o.passed) {
                std::process::exit(1);
            }
        }
        Command::Revise {
            path,
            spec,
            out,
            emit_nvim,
            report,
        } => {
            let theme_json = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let theme = hexwitch_core::ingest::load_theme(&theme_json)
                .map_err(|e| format!("cannot load theme: {e}"))?;
            let spec_json = std::fs::read_to_string(&spec)
                .map_err(|e| format!("cannot read {}: {e}", spec.display()))?;
            let spec = hexwitch_core::revise::EditSpec::from_json(&spec_json)
                .map_err(|e| format!("invalid edit spec: {e}"))?;

            let (revised, revision) = hexwitch_core::revise::revise(&theme, &spec)
                .map_err(|e| format!("revision refused: {e}"))?;

            eprint!("{}", revision.summary());
            if !revision.failed.is_empty() || report {
                eprintln!(
                    "({} role(s) failed; failures keep their original colors)",
                    revision.failed.len()
                );
            }

            if let Some(nvim_out) = emit_nvim {
                let emitted =
                    hexwitch_core::emit::emit_nvim(&revised).map_err(|e| e.to_string())?;
                if let Some(parent) = nvim_out.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
                }
                std::fs::write(&nvim_out, emitted.lua)
                    .map_err(|e| format!("cannot write {}: {e}", nvim_out.display()))?;
                eprintln!(
                    "emitted {} ({} defined, {} linked, {} skipped)",
                    nvim_out.display(),
                    emitted.report.defined.len(),
                    emitted.report.linked.len(),
                    emitted.report.skipped.len()
                );
            }

            let revised_json = revised.to_json_string().map_err(|e| e.to_string())?;
            match out {
                Some(p) => {
                    std::fs::write(&p, revised_json)
                        .map_err(|e| format!("cannot write {}: {e}", p.display()))?;
                    eprintln!("revised theme written to {}", p.display());
                }
                None => println!("{revised_json}"),
            }
        }
    }
    Ok(())
}
