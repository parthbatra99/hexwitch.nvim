//! `hexwitch` CLI — thin, composable surface over hexwitch-core.
//!
//! M0 scope: color inspection primitives (`convert`, `contrast`) so the
//! engine's math is dogfoodable from day one. Ingest/analyze/revise/emit
//! subcommands arrive with their milestones and must stay scriptable —
//! the Neovim adapter (and later VS Code/Zed) drive this same binary.

use clap::{Parser, Subcommand};
use hexwitch_core::color::{contrast_verdict, gamut_map, Oklch, Srgb};
use hexwitch_core::eval::{render_report, run_eval_dir};
use hexwitch_core::ingest::ingest_vscode_json;

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
    /// Run the eval set: ingest + analyze reference themes, compare
    /// against goldens and invariants. Non-zero exit on failure.
    Eval {
        /// Eval directory containing manifest.json. Defaults to `eval`
        /// relative to the current directory (the workspace root is `core/`).
        #[arg(default_value = "eval")]
        dir: std::path::PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Target {
    Oklch,
    Oklab,
    Srgb,
    Hex,
}

fn parse_color(s: &str) -> Result<Srgb, String> {
    Srgb::from_hex(s).map_err(|e| e.to_string())
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
            let theme = ingest_vscode_json(&source).map_err(|e| e.to_string())?;
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
        Command::Eval { dir } => {
            let outcomes = run_eval_dir(&dir).map_err(|e| format!("eval failed: {e}"))?;
            print!("{}", render_report(&outcomes));
            if outcomes.iter().any(|o| !o.passed) {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
