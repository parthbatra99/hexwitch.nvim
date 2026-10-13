mod api_config;
mod artifact;
mod machine;
mod playground;
mod provider;
mod providers;
mod references;

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
    #[arg(help = "Versioned JSON response envelope (never parse human stderr)")]
    #[arg(long, global = true)]
    machine: bool,
    #[arg(long, global = true, default_value = "local")]
    request_id: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Start the loopback-only interactive engine playground")]
    Playground {
        #[arg(long, default_value_t = 7331)]
        port: u16,
        #[arg(
            long,
            help = "Where saved providers live (default: $HEXWITCH_PROVIDERS_FILE or ~/.config/hexwitch/providers.json). Key values are never written"
        )]
        providers_file: Option<std::path::PathBuf>,
        #[command(flatten)]
        api: api_config::ApiOptions,
    },
    #[command(about = "Discover protocol, engine, and supported operation versions")]
    Capabilities,
    #[command(about = "Import source data into validated IR (migration must be explicit)")]
    Import {
        path: std::path::PathBuf,
        #[arg(long)]
        migrate_v0: bool,
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
    },
    #[command(about = "Validate calculated scenes; unknown rendering is unverified, not pass")]
    Validate {
        path: std::path::PathBuf,
        #[arg(long)]
        scenes: Option<std::path::PathBuf>,
    },
    #[command(about = "Judge recorded v1 model responses against semantic outcomes (offline)")]
    EvalInterpretation {
        suite: std::path::PathBuf,
        #[arg(long)]
        run: std::path::PathBuf,
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
    },
    #[command(about = "Convert a color between representations")]
    Convert {
        #[arg(help = "Color as #rrggbb")]
        color: String,
        #[arg(help = "Target representation")]
        #[arg(short, long, value_enum, default_value_t = Target::Oklch)]
        to: Target,
    },
    #[command(about = "WCAG contrast between two colors, with verdicts")]
    Contrast {
        #[arg(help = "Foreground as #rrggbb")]
        foreground: String,
        #[arg(help = "Background as #rrggbb")]
        background: String,
        #[arg(help = "Emit machine-readable JSON instead of text")]
        #[arg(long)]
        json: bool,
    },
    #[command(about = "Gamut-map an OKLCH color into sRGB (chroma reduction, hue kept)")]
    Map {
        #[arg(help = "OKLCH lightness, 0-1")]
        l: f64,
        #[arg(help = "OKLCH chroma, >= 0")]
        c: f64,
        #[arg(help = "OKLCH hue in degrees")]
        h: f64,
    },
    #[command(
        about = "Ingest a VS Code-format theme JSON and print its fingerprint (contrast ladder, accent budget)"
    )]
    Analyze {
        #[arg(help = "Path to a VS Code theme .json file")]
        path: std::path::PathBuf,
        #[arg(help = "Emit the full fingerprint as JSON")]
        #[arg(long)]
        json: bool,
    },
    #[command(
        about = "Run the eval set: ingest + analyze reference themes, compare against goldens and invariants. Non-zero exit on failure"
    )]
    Eval {
        #[arg(
            help = "Eval directory containing manifest.json. Defaults to `eval` relative to the current directory (the workspace root is `core/`)"
        )]
        #[arg(default_value = "eval")]
        dir: std::path::PathBuf,
    },
    #[command(
        about = "Apply an edit spec to a theme: protected roles survive, changes apply minimally, everything else stays bit-identical"
    )]
    Revise {
        #[arg(help = "Path to a theme .json file (IR or VS Code format)")]
        path: std::path::PathBuf,
        #[arg(help = "Path to an edit-spec .json file")]
        #[arg(short, long)]
        spec: std::path::PathBuf,
        #[arg(help = "Write the revised theme (IR JSON) here instead of stdout")]
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        #[arg(help = "Print the revision report even on success")]
        #[arg(long)]
        report: bool,
        #[arg(help = "Write a machine-readable engine revision report")]
        #[arg(long)]
        report_json: Option<std::path::PathBuf>,
    },
    #[command(
        about = "Interpret a natural-language request as an edit spec (LLM bridge). The model never picks colors; it emits actions that are validated hard and translated deterministically"
    )]
    Interpret {
        #[arg(help = "Path to a theme .json file (IR or VS Code format)")]
        path: std::path::PathBuf,
        #[arg(
            help = "The request, e.g. \"darker background, readable comments, keep the syntax colors\". Required unless --from-llm is given"
        )]
        prompt: Option<String>,
        #[arg(help = "Print the exact prompt that would be sent and exit (no network)")]
        #[arg(long)]
        dry_run: bool,
        #[arg(
            help = "Offline mode: validate a saved model response instead of calling a provider"
        )]
        #[arg(long)]
        from_llm: Option<std::path::PathBuf>,
        #[arg(help = "Write the validated edit spec here (IR EditSpec JSON)")]
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        #[command(flatten)]
        api: api_config::ApiOptions,
        #[arg(help = "Property-aware interpretation with explicit clarification outcomes")]
        #[arg(long)]
        v1: bool,
        #[arg(help = "Reviewed prior requests/actions for relative iteration (v1 only)")]
        #[arg(long, requires = "v1")]
        context: Option<std::path::PathBuf>,
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
    if cli.machine {
        machine_main(cli.command, &cli.request_id);
        return;
    }
    if let Err(err) = run(cli.command) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Playground {
            port,
            providers_file,
            api,
        } => playground::serve(port, api, providers_file)?,
        Command::Capabilities => println!("{}", hexwitch_core::protocol::capabilities()),
        Command::Import {
            path,
            migrate_v0,
            out,
        } => {
            let raw = read_input(&path)?;
            let theme = if migrate_v0 {
                hexwitch_core::ir::Theme::migrate_v0(&raw).map_err(|e| e.to_string())?
            } else {
                hexwitch_core::import_path::load_theme_path(&path).map_err(|e| e.to_string())?
            };
            let value = serde_json::to_value(theme).map_err(|e| e.to_string())?;
            if out.is_none() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
                );
            }
            write_spec_value(&value, out)?;
        }
        Command::Validate { path, scenes } => {
            let theme = hexwitch_core::ingest::load_theme(&read_input(&path)?)
                .map_err(|e| e.to_string())?;
            let scenes = match scenes {
                Some(p) => serde_json::from_str(&read_input(&p)?)
                    .map_err(|e| format!("invalid scenes: {e}"))?,
                None => hexwitch_core::validate::initial_scenes(),
            };
            let report = hexwitch_core::validate::validate_scenes(&theme, &scenes)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
        }
        Command::EvalInterpretation { suite, run, out } => {
            let report = hexwitch_core::interpret_eval::run_suite(&suite, &run)?;
            let value = serde_json::to_value(&report).map_err(|e| e.to_string())?;
            write_spec_value(&value, out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            if report.failed > 0 || report.pair_failures > 0 {
                return Err(format!(
                    "{} interpretation cases and {} pair checks failed",
                    report.failed, report.pair_failures
                ));
            }
        }
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
            if !l.is_finite()
                || !(0.0..=1.0).contains(&l)
                || !c.is_finite()
                || !(0.0..=1.0).contains(&c)
                || !h.is_finite()
            {
                return Err("invalid OKLCH parameters".into());
            }
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
            let source = read_input(&path)?;
            let _ = source;
            let theme =
                hexwitch_core::import_path::load_theme_path(&path).map_err(|e| e.to_string())?;
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
        Command::Revise {
            path,
            spec,
            out,
            report,
            report_json,
        } => {
            ensure_distinct(&[out.as_deref(), report_json.as_deref()])?;
            let theme_json = read_input(&path)?;
            let _ = theme_json;
            let theme = hexwitch_core::import_path::load_theme_path(&path)
                .map_err(|e| format!("cannot load theme: {e}"))?;
            let spec_json = read_input(&spec)?;
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

            if let Some(path) = report_json {
                let json = serde_json::to_string_pretty(&revision).map_err(|e| e.to_string())?;
                artifact::write_atomic(&path, json.as_bytes())
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
            }
            let revised_json = revised.to_json_string().map_err(|e| e.to_string())?;
            match out {
                Some(p) => {
                    artifact::write_atomic(&p, revised_json.as_bytes())
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
            api,
            v1,
            context,
        } => {
            if v1 {
                return run_interpret_v1(path, prompt, dry_run, from_llm, out, api, context);
            }
            use hexwitch_core::interpret as interp;

            let theme_json = read_input(&path)?;
            let _ = theme_json;
            let theme =
                hexwitch_core::import_path::load_theme_path(&path).map_err(|e| e.to_string())?;

            if let Some(response_path) = from_llm {
                let raw = read_input(&response_path)?;
                let llm =
                    interp::parse_model_output(&raw).map_err(|e| format!("model output: {e}"))?;
                let spec = interp::llm_spec_to_edit_spec(&llm)
                    .map_err(|e| format!("invalid spec: {e}"))?;
                spec.validate(&theme)
                    .map_err(|e| format!("invalid theme targets: {e}"))?;
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

            let connection = api.resolve()?;
            let llm = provider::call_provider(
                &connection.endpoint,
                &connection.key,
                &connection.model,
                &system,
                &user,
                connection.format,
            )
            .map_err(|e| e.to_string())?;
            let spec = interp::llm_spec_to_edit_spec(&llm).map_err(|e| e.to_string())?;
            spec.validate(&theme).map_err(|e| e.to_string())?;
            if let Some(notes) = llm.notes {
                eprintln!("notes: {notes}");
            }
            write_spec(&spec, out)?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_interpret_v1(
    path: std::path::PathBuf,
    prompt: Option<String>,
    dry_run: bool,
    from_llm: Option<std::path::PathBuf>,
    out: Option<std::path::PathBuf>,
    api: api_config::ApiOptions,
    context: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let theme = hexwitch_core::import_path::load_theme_path(&path).map_err(|e| e.to_string())?;
    let prompt=prompt.ok_or("v1 interpretation requires the original request, including for offline trusted-color validation")?;
    let context = match context {
        Some(p) => serde_json::from_str(&read_input(&p)?).map_err(|e| e.to_string())?,
        None => hexwitch_core::interpret_v1::IterationContext::default(),
    };
    let user = hexwitch_core::interpret_v1::user_prompt(&theme, &prompt, &context)?;
    let system = hexwitch_core::interpret_v1::system_prompt();
    if dry_run {
        println!("--- system ---\n{system}\n--- user ---\n{user}");
        return Ok(());
    }
    let response = match from_llm {
        Some(p) => hexwitch_core::interpret_v1::parse_model_output(&read_input(&p)?)?,
        None => {
            let connection = api.resolve()?;
            provider::call_provider_v1(
                &connection.endpoint,
                &connection.key,
                &connection.model,
                &system,
                &user,
                connection.format,
                &prompt,
            )
            .map_err(|e| e.to_string())?
        }
    };
    let interpretation = hexwitch_core::interpret_v1::validate(&response, &theme, &prompt)?;
    if let (Some(spec), Some(path)) = (&interpretation.spec, out) {
        write_spec_value(
            &serde_json::to_value(spec).map_err(|e| e.to_string())?,
            Some(path),
        )?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&interpretation).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn read_input(path: &std::path::Path) -> Result<String, String> {
    let metadata =
        std::fs::metadata(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    if metadata.len() > 8 * 1024 * 1024 {
        return Err("input exceeds 8 MiB limit".into());
    }
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    use std::io::Read;
    let mut raw = String::new();
    file.take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > 8 * 1024 * 1024 {
        return Err("input exceeds 8 MiB limit".into());
    }
    Ok(raw)
}

fn ensure_distinct(paths: &[Option<&std::path::Path>]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for path in paths.iter().flatten() {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(path)
        };
        let mut normalized = std::path::PathBuf::new();
        for component in absolute.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other.as_os_str()),
            }
        }
        if let Some(parent) = normalized.parent() {
            if let Ok(real) = parent.canonicalize() {
                normalized = real.join(normalized.file_name().unwrap());
            }
        }
        if !seen.insert(normalized) {
            return Err("artifact output paths must be distinct".into());
        }
    }
    Ok(())
}

fn write_spec_value(
    value: &serde_json::Value,
    out: Option<std::path::PathBuf>,
) -> Result<(), String> {
    if let Some(path) = out {
        artifact::write_atomic(
            &path,
            serde_json::to_string_pretty(value)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn machine_main(command: Command, request_id: &str) {
    let response = machine::run(command, request_id);
    let failed = response.outcome == hexwitch_core::protocol::Outcome::Error;
    println!(
        "{}",
        serde_json::to_string(&response).expect("finite validated protocol response")
    );
    if failed {
        std::process::exit(1);
    }
}

fn write_spec(
    spec: &hexwitch_core::revise::EditSpec,
    out: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(spec).map_err(|e| e.to_string())?;
    match out {
        Some(p) => {
            artifact::write_atomic(&p, json.as_bytes())
                .map_err(|e| format!("cannot write {}: {e}", p.display()))?;
            eprintln!("edit spec written to {}", p.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}
