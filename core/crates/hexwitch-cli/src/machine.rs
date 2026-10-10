//! Protocol operations; all outputs remain structured and errors have stable codes.
use super::*;
use hexwitch_core::protocol::{ErrorCode, Outcome, Response};
use serde_json::{json, Value};

pub fn run(command: Command, request_id: &str) -> Response<Value> {
    let operation = match &command {
        Command::Capabilities => "capabilities",
        Command::Import { .. } => "import",
        Command::Interpret { .. } => "interpret",
        Command::Revise { .. } => "revise",
        Command::Validate { .. } => "validate",
        Command::Emit { .. } => "emit",
        _ => "unsupported",
    };
    let fail = |code, message: String| Response::failure(request_id, operation, code, message);
    let result = (|| -> Result<(Outcome, Value), (ErrorCode, String)> {
        let io = |s: String| (ErrorCode::Io, s);
        let invalid = |s: String| (ErrorCode::InvalidInput, s);
        let theme =
            |path: &std::path::Path| -> Result<hexwitch_core::ir::Theme, (ErrorCode, String)> {
                let source = read_input(path).map_err(io)?;
                let _ = source;
                hexwitch_core::import_path::load_theme_path(path).map_err(|e| {
                    let message = e.to_string();
                    let code = if message.contains("version") || message.contains("migration") {
                        ErrorCode::UnsupportedVersion
                    } else {
                        ErrorCode::InvalidInput
                    };
                    (code, message)
                })
            };
        match command {
            Command::Capabilities => {
                Ok((Outcome::Satisfied, hexwitch_core::protocol::capabilities()))
            }
            Command::Import {
                path,
                migrate_v0,
                out,
            } => {
                let imported = if migrate_v0 {
                    hexwitch_core::ir::Theme::migrate_v0(&read_input(&path).map_err(io)?)
                        .map_err(|e| (ErrorCode::UnsupportedVersion, e.to_string()))?
                } else {
                    theme(&path)?
                };
                let value = serde_json::to_value(&imported).map_err(|e| invalid(e.to_string()))?;
                write_spec_value(&value, out).map_err(io)?;
                let losses = imported.extensions.get("hexwitch.import.losses.v1");
                let outcome = if losses
                    .and_then(Value::as_array)
                    .is_some_and(|l| !l.is_empty())
                {
                    Outcome::Partial
                } else {
                    Outcome::Satisfied
                };
                Ok((
                    outcome,
                    json!({"theme":value,"losses":losses,"limitations":["Limited representative scope-stack grammar; custom semantic hierarchy/defaults require editor context"]}),
                ))
            }
            Command::Validate { path, scenes } => {
                let theme = theme(&path)?;
                let scenes = match scenes {
                    Some(p) => serde_json::from_str(&read_input(&p).map_err(io)?)
                        .map_err(|e| invalid(format!("invalid scenes: {e}")))?,
                    None => hexwitch_core::validate::initial_scenes(),
                };
                let report =
                    hexwitch_core::validate::validate_scenes(&theme, &scenes).map_err(invalid)?;
                let outcome = match report.status {
                    hexwitch_core::validate::Status::Pass => Outcome::Satisfied,
                    hexwitch_core::validate::Status::Fail => Outcome::Infeasible,
                    hexwitch_core::validate::Status::Unverified => Outcome::Partial,
                };
                Ok((outcome, json!(report)))
            }
            Command::Revise {
                path,
                spec,
                out,
                emit_nvim,
                report_json,
                ..
            } => {
                ensure_distinct(&[out.as_deref(), emit_nvim.as_deref(), report_json.as_deref()])
                    .map_err(invalid)?;
                let original = theme(&path)?;
                let raw = read_input(&spec).map_err(io)?;
                let value: Value =
                    serde_json::from_str(&raw).map_err(|e| invalid(e.to_string()))?;
                let (revised, report, outcome) = if value.get("version").is_some() {
                    let spec = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
                    let (revised, report) =
                        hexwitch_core::intent::revise_intent(&original, &spec).map_err(invalid)?;
                    let outcome = report.outcome;
                    (revised, json!(report), outcome)
                } else {
                    let spec = hexwitch_core::revise::EditSpec::from_json(&raw)
                        .map_err(|e| invalid(e.to_string()))?;
                    let (revised, report) = hexwitch_core::revise::revise(&original, &spec)
                        .map_err(|e| {
                            let code = match e {
                                hexwitch_core::revise::ReviseError::UnknownProtectRole(_)
                                | hexwitch_core::revise::ReviseError::UnknownTargetRole {
                                    ..
                                } => ErrorCode::MissingTarget,
                                _ => ErrorCode::InvalidInput,
                            };
                            (code, e.to_string())
                        })?;
                    let outcome = if !report.failed.is_empty() || !report.refused.is_empty() {
                        Outcome::Partial
                    } else if report.applied.is_empty() {
                        Outcome::Unchanged
                    } else {
                        Outcome::Satisfied
                    };
                    (revised, json!(report), outcome)
                };
                // Prepare every artifact before writing any of them.
                let emitted = emit_nvim
                    .as_ref()
                    .map(|_| hexwitch_core::emit::emit_nvim(&revised))
                    .transpose()
                    .map_err(|e| invalid(e.to_string()))?;
                if let Some(p) = emit_nvim {
                    artifact::write_atomic(&p, emitted.as_ref().unwrap().lua.as_bytes())
                        .map_err(|e| io(e.to_string()))?;
                }
                if let Some(p) = report_json {
                    artifact::write_atomic(&p, report.to_string().as_bytes())
                        .map_err(|e| io(e.to_string()))?;
                }
                let value = json!(revised);
                write_spec_value(&value, out).map_err(io)?;
                Ok((
                    outcome,
                    json!({"theme":value,"revision":report,"atomicity":"per artifact, not a multi-file transaction"}),
                ))
            }
            Command::Emit {
                path, target, out, ..
            } => {
                let theme = theme(&path)?;
                let emitted = match target {
                    Editor::Nvim => hexwitch_core::emit::emit_nvim(&theme)
                        .map_err(|e| invalid(e.to_string()))?,
                };
                if let Some(path) = out {
                    artifact::write_atomic(&path, emitted.lua.as_bytes())
                        .map_err(|e| io(e.to_string()))?;
                }
                let outcome = if emitted.report.losses.is_empty()
                    && emitted.report.unconsumed_roles.is_empty()
                    && emitted.report.skipped.is_empty()
                {
                    Outcome::Satisfied
                } else {
                    Outcome::Partial
                };
                Ok((
                    outcome,
                    json!({"artifact":emitted.lua,"report":emitted.report}),
                ))
            }
            Command::Interpret {
                path,
                from_llm,
                dry_run,
                prompt,
                provider: provider_name,
                model,
                out,
            } => {
                let theme = theme(&path)?;
                if dry_run {
                    return Ok((
                        Outcome::Unchanged,
                        json!({"system":hexwitch_core::interpret::system_prompt(),"user":hexwitch_core::interpret::user_prompt(&theme,prompt.as_deref().unwrap_or(""))}),
                    ));
                }
                let llm = if let Some(path) = from_llm {
                    hexwitch_core::interpret::parse_model_output(&read_input(&path).map_err(io)?)
                        .map_err(|e| invalid(e.to_string()))?
                } else {
                    let prompt =
                        prompt.ok_or_else(|| invalid("non-empty prompt required".into()))?;
                    if prompt.trim().is_empty() {
                        return Err(invalid("non-empty prompt required".into()));
                    }
                    let (endpoint, key, default_model) = pick_provider(provider_name.as_deref())
                        .map_err(|e| (ErrorCode::ProviderUnavailable, e))?;
                    provider::call_provider(
                        endpoint,
                        &key,
                        model.as_deref().unwrap_or(default_model),
                        &hexwitch_core::interpret::system_prompt(),
                        &hexwitch_core::interpret::user_prompt(&theme, &prompt),
                    )
                    .map_err(|e| {
                        let code = match e.code {
                            "timeout" => ErrorCode::Timeout,
                            "rate_limited" => ErrorCode::RateLimited,
                            "refusal" => ErrorCode::ProviderRefusal,
                            "payload_too_large" => ErrorCode::PayloadTooLarge,
                            _ => ErrorCode::ProviderUnavailable,
                        };
                        (code, e.to_string())
                    })?
                };
                let spec = hexwitch_core::interpret::llm_spec_to_edit_spec(&llm)
                    .map_err(|e| invalid(e.to_string()))?;
                spec.validate(&theme)
                    .map_err(|e| (ErrorCode::MissingTarget, e.to_string()))?;
                let outcome = if spec.changes.is_empty() && spec.protect.is_empty() {
                    Outcome::NeedsClarification
                } else {
                    Outcome::Satisfied
                };
                write_spec_value(&json!(spec), out).map_err(io)?;
                Ok((outcome, json!({"spec":spec,"notes":llm.notes})))
            }
            _ => Err((
                ErrorCode::UnsupportedCapability,
                "machine mode supports import/interpret/revise/validate/emit/capabilities".into(),
            )),
        }
    })();
    match result {
        Ok((outcome, value)) => Response::success(request_id, operation, outcome, value),
        Err((code, message)) => fail(code, message),
    }
}
