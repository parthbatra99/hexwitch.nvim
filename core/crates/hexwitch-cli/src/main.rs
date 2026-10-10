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
    /// Interpret a natural-language request as an edit spec (LLM bridge).
    /// The model never picks colors; it emits actions that are validated
    /// hard and translated deterministically.
    Interpret {
        /// Path to a theme .json file (IR or VS Code format).
        path: std::path::PathBuf,
        /// The request, e.g. "darker background, readable comments, keep
        /// the syntax colors". Required unless --from-llm is given.
        prompt: Option<String>,
        /// Print the exact prompt that would be sent and exit (no network).
        #[arg(long)]
        dry_run: bool,
        /// Offline mode: validate a saved model response instead of
        /// calling a provider.
        #[arg(long)]
        from_llm: Option<std::path::PathBuf>,
        /// Write the validated edit spec here (IR EditSpec JSON).
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        /// Provider: `openai` or `openrouter` (default: whichever key is
        /// set, OPENAI_API_KEY / OPENROUTER_API_KEY).
        #[arg(long)]
        provider: Option<String>,
        /// Model name (default gpt-4o-mini / openai/gpt-4o-mini).
        #[arg(short, long)]
        model: Option<String>,
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
        Command::Interpret {
            path,
            prompt,
            dry_run,
            from_llm,
            out,
            provider,
            model,
        } => {
            use hexwitch_core::interpret as interp;

            let theme_json = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let theme =
                hexwitch_core::ingest::load_theme(&theme_json).map_err(|e| e.to_string())?;

            // Offline validation path: no network, no prompt needed.
            if let Some(response_path) = from_llm {
                let raw = std::fs::read_to_string(&response_path)
                    .map_err(|e| format!("cannot read {}: {e}", response_path.display()))?;
                let llm =
                    interp::parse_model_output(&raw).map_err(|e| format!("model output: {e}"))?;
                let spec = interp::llm_spec_to_edit_spec(&llm)
                    .map_err(|e| format!("invalid spec: {e}"))?;
                if let Some(notes) = &llm.notes {
                    eprintln!("notes: {notes}");
                }
                write_spec(&spec, out)?;
                return Ok(());
            }

            let prompt = prompt.ok_or("a PROMPT is required (or use --from-llm)")?;
            let system = interp::system_prompt();
            let user = interp::user_prompt(&theme, &prompt);

            if dry_run {
                println!("--- system ---\n{system}\n--- user ---\n{user}");
                return Ok(());
            }

            let (endpoint, api_key, default_model) = pick_provider(provider.as_deref())?;
            let model = model.unwrap_or_else(|| default_model.to_string());

            let spec = call_provider(endpoint, &api_key, &model, &system, &user)
                .map_err(|e| e.to_string())?;
            write_spec(&spec, out)?;
        }
    }
    Ok(())
}

fn write_spec(
    spec: &hexwitch_core::revise::EditSpec,
    out: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(spec).map_err(|e| e.to_string())?;
    match out {
        Some(p) => {
            std::fs::write(&p, json).map_err(|e| format!("cannot write {}: {e}", p.display()))?;
            eprintln!("edit spec written to {}", p.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}

const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

fn pick_provider(flag: Option<&str>) -> Result<(&'static str, String, &'static str), String> {
    let openai_key = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());
    let openrouter_key = std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());
    match flag {
        Some("openai") => {
            let k = openai_key.ok_or("OPENAI_API_KEY is not set")?;
            Ok((OPENAI_ENDPOINT, k, "gpt-4o-mini"))
        }
        Some("openrouter") => {
            let k = openrouter_key.ok_or("OPENROUTER_API_KEY is not set")?;
            Ok((OPENROUTER_ENDPOINT, k, "openai/gpt-4o-mini"))
        }
        Some(other) => Err(format!("unknown provider {other:?} (openai | openrouter)")),
        None => match (openai_key, openrouter_key) {
            (Some(k), _) => Ok((OPENAI_ENDPOINT, k, "gpt-4o-mini")),
            (None, Some(k)) => Ok((OPENROUTER_ENDPOINT, k, "openai/gpt-4o-mini")),
            (None, None) => Err(
                "no provider key found: set OPENAI_API_KEY or OPENROUTER_API_KEY, or pass --provider"
                    .to_string(),
            ),
        },
    }
}

/// Call the provider via curl (no async runtime dependency). The API key
/// lives in a 0600 header file, never in argv. One retry on validation
/// failure, with the validation error shown to the model.
fn call_provider(
    endpoint: &str,
    api_key: &str,
    model: &str,
    system: &str,
    user: &str,
) -> Result<hexwitch_core::revise::EditSpec, String> {
    use hexwitch_core::interpret as interp;

    let mut messages = vec![
        serde_json::json!({ "role": "system", "content": system }),
        serde_json::json!({ "role": "user", "content": user }),
    ];

    let tmp = std::env::temp_dir().join(format!("hexwitch-interpret-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let header_path = tmp.join("headers");
    let payload_path = tmp.join("payload.json");
    let response_path = tmp.join("response.json");

    // Header file keeps the key out of argv (`ps` visibility).
    std::fs::write(
        &header_path,
        format!("Content-Type: application/json\nAuthorization: Bearer {api_key}"),
    )
    .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&header_path, std::fs::Permissions::from_mode(0o600));
    }

    let cleanup = || {
        let _ = std::fs::remove_file(&header_path);
        let _ = std::fs::remove_file(&payload_path);
        let _ = std::fs::remove_file(&response_path);
        let _ = std::fs::remove_dir(&tmp);
    };

    let result = (|| -> Result<hexwitch_core::revise::EditSpec, String> {
        for attempt in 1..=2 {
            let payload = serde_json::json!({
                "model": model,
                "temperature": 0,
                "messages": messages,
                "response_format": {
                    "type": "json_schema",
                    "json_schema": interp::response_schema(),
                },
            });
            std::fs::write(&payload_path, payload.to_string()).map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(&response_path);

            let status = std::process::Command::new("curl")
                .arg("-sS")
                .arg("-m")
                .arg("120")
                .arg("-X")
                .arg("POST")
                .arg(endpoint)
                .arg("-H")
                .arg(format!("@{}", header_path.display()))
                .arg("--data-binary")
                .arg(format!("@{}", payload_path.display()))
                .arg("-o")
                .arg(response_path.display().to_string())
                .arg("-w")
                .arg("%{http_code}")
                .output()
                .map_err(|e| format!("cannot run curl: {e} (is it installed?)"))?;
            let code = String::from_utf8_lossy(&status.stdout).trim().to_string();
            let body = std::fs::read_to_string(&response_path).unwrap_or_default();
            if !code.starts_with('2') {
                return Err(format!(
                    "provider returned HTTP {code}: {}",
                    truncate(&body, 400)
                ));
            }

            let content = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| {
                    v.pointer("/choices/0/message/content")
                        .and_then(|c| c.as_str())
                        .map(str::to_string)
                })
                .ok_or_else(|| {
                    format!("no message content in response: {}", truncate(&body, 400))
                })?;

            match interp::parse_model_output(&content)
                .and_then(|llm| interp::llm_spec_to_edit_spec(&llm).map(|spec| (llm, spec)))
            {
                Ok((llm, spec)) => {
                    if let Some(notes) = &llm.notes {
                        eprintln!("notes: {notes}");
                    }
                    return Ok(spec);
                }
                Err(e) if attempt == 1 => {
                    eprintln!("attempt 1 failed validation ({e}); asking the model to correct it");
                    messages.push(serde_json::json!({ "role": "assistant", "content": content }));
                    messages.push(serde_json::json!({
                        "role": "user",
                        "content": format!("Your output failed validation: {e}. Return a corrected JSON that matches the schema exactly.")
                    }));
                }
                Err(e) => {
                    return Err(format!(
                        "model output invalid after retry: {e}\nraw output: {content}"
                    ))
                }
            }
        }
        unreachable!("loop returns in both branches")
    })();

    cleanup();
    result
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
