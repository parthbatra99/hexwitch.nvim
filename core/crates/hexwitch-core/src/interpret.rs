//! Interpretation bridge: natural language -> edit spec.
//!
//! The contract, established in the first-principles design:
//!
//! * **The LLM never picks colors.** It emits a flat action list
//!   ([`LlmSpec`]) constrained by a JSON schema (for structured-output
//!   providers) and translated *deterministically* into an [`EditSpec`].
//!   All color work stays in the engine.
//! * **Validation is hard.** Unknown actions, missing fields, nonsense
//!   ratios — all rejected with precise errors, never repaired by guess.
//! * **Offline-testable.** Prompt building and translation are pure
//!   functions; only the provider call (in the CLI) touches the network.
//!
//! The wire format is deliberately flatter than [`EditSpec`]: a single
//! action list where `protect` is just another action. Flat + nullable
//! fields map cleanly onto strict structured-output schemas, which means
//! providers can enforce the shape *server-side*, and we enforce it again
//! client-side.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use thiserror::Error;

use crate::analyze::fingerprint;
use crate::color::Srgb;
use crate::ir::role::Role;
use crate::ir::theme::Theme;
use crate::revise::{Against, Change, EditSpec, Op};

/// Valid action names, mirrored in the JSON schema and the system prompt.
pub const ACTIONS: &[&str] = &[
    "protect",
    "lighter",
    "darker",
    "saturate",
    "desaturate",
    "hue",
    "min_contrast",
];

/// One action as emitted by the model. All optional fields are nullable
/// so the shape fits strict structured-output schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmAction {
    pub action: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub against: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shift: Option<f64>,
}

/// The model's full response: an action list plus free-form notes the UI
/// can surface to the user ("here is how I read your request").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub actions: Vec<LlmAction>,
}

#[derive(Debug, Error, PartialEq)]
pub enum InterpretError {
    #[error("action #{index} (`{action}`): {message}")]
    InvalidAction {
        index: usize,
        action: String,
        message: String,
    },
    #[error("could not parse model output as JSON: {0}")]
    BadJson(String),
}

impl From<serde_json::Error> for InterpretError {
    fn from(e: serde_json::Error) -> Self {
        InterpretError::BadJson(e.to_string())
    }
}

/// Translate the model's flat action list into an engine edit spec.
/// Deterministic, total on valid input, precise on invalid input.
pub fn llm_spec_to_edit_spec(spec: &LlmSpec) -> Result<EditSpec, InterpretError> {
    let mut out = EditSpec::default();
    for (index, a) in spec.actions.iter().enumerate() {
        let invalid = |message: String| InterpretError::InvalidAction {
            index,
            action: a.action.clone(),
            message,
        };
        let require_amount = |v: Option<f64>| -> Result<f64, InterpretError> {
            v.ok_or_else(|| {
                invalid(
                    "requires `amount` (OKLCH lightness/chroma delta, 0-1; typically 0.02-0.08)"
                        .to_string(),
                )
            })
            .and_then(|n| {
                if !(0.0..=1.0).contains(&n) {
                    Err(invalid(format!("`amount` {n} outside 0-1")))
                } else {
                    Ok(n)
                }
            })
        };
        let require_target = |t: &str| -> Result<(), InterpretError> {
            let ok = t == "*" || t.ends_with(".*") || t.parse::<Role>().is_ok();
            if ok {
                Ok(())
            } else {
                Err(invalid(format!(
                    "`target` {t:?} is not a role name or wildcard"
                )))
            }
        };

        require_target(&a.target)?;

        let change = |op: Op| Change {
            target: a.target.clone(),
            op,
        };
        match a.action.as_str() {
            "protect" => out.protect.push(a.target.clone()),
            "lighter" => out.changes.push(change(Op::Lighter {
                amount: require_amount(a.amount)?,
            })),
            "darker" => out.changes.push(change(Op::Darker {
                amount: require_amount(a.amount)?,
            })),
            "saturate" => out.changes.push(change(Op::Saturate {
                amount: require_amount(a.amount)?,
            })),
            "desaturate" => out.changes.push(change(Op::Desaturate {
                amount: require_amount(a.amount)?,
            })),
            "hue" => {
                let shift = a.shift.ok_or_else(|| {
                    invalid("`hue` requires `shift` (degrees, e.g. 10)".to_string())
                })?;
                if !(-180.0..=180.0).contains(&shift) {
                    return Err(invalid(format!("`shift` {shift} outside -180..180")));
                }
                out.changes.push(change(Op::Hue { shift }));
            }
            "min_contrast" => {
                let min = a.min.ok_or_else(|| {
                    invalid("`min_contrast` requires `min` (WCAG ratio, typically 4.5)".to_string())
                })?;
                if !(1.0..=21.0).contains(&min) {
                    return Err(invalid(format!("`min` {min} outside 1-21")));
                }
                let against = a.against.as_deref().ok_or_else(|| {
                    invalid(
                        "`min_contrast` requires `against` (a role name or #rrggbb)".to_string(),
                    )
                })?;
                let parsed = Against::parse(against).ok_or_else(|| {
                    invalid(format!(
                        "`against` {against:?} is neither a role nor #rrggbb"
                    ))
                })?;
                out.changes.push(change(Op::MinContrast {
                    against: parsed,
                    min,
                }));
            }
            other => {
                return Err(invalid(format!(
                    "unknown action {other:?}; valid actions: {ACTIONS:?}"
                )))
            }
        }
    }
    Ok(out)
}

/// Extract an [`LlmSpec`] from raw model output: either the action JSON
/// itself (optionally fenced) or a full chat-completion envelope.
pub fn parse_model_output(raw: &str) -> Result<LlmSpec, InterpretError> {
    let trimmed = raw.trim();
    let unfenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);

    // Try the spec directly first, then a full response envelope.
    if let Ok(spec) = serde_json::from_str::<LlmSpec>(unfenced) {
        return Ok(spec);
    }
    let v: serde_json::Value = serde_json::from_str(unfenced)?;
    if let Some(content) = v
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
    {
        return Ok(serde_json::from_str(content.trim())?);
    }
    serde_json::from_value(v).map_err(InterpretError::from)
}

/// The JSON schema sent to structured-output providers, enforcing the
/// flat action shape server-side.
pub fn response_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "hexwitch_edit_spec",
        "strict": true,
        "schema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["notes", "actions"],
            "properties": {
                "notes": { "type": ["string", "null"] },
                "actions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["action", "target", "amount", "against", "min", "shift"],
                        "properties": {
                            "action": { "type": "string", "enum": ACTIONS },
                            "target": { "type": "string" },
                            "amount": { "type": ["number", "null"] },
                            "against": { "type": ["string", "null"] },
                            "min": { "type": ["number", "null"] },
                            "shift": { "type": ["number", "null"] }
                        }
                    }
                }
            }
        }
    })
}

/// The system prompt: the interpretation contract.
pub fn system_prompt() -> String {
    r#"You are the interpretation layer of the hexwitch theme engine. You translate a user's request about their editor theme into a structured edit spec. You never invent colors: every action is a named operation on a theme role, and a deterministic engine performs it.

Valid actions:
- protect (no fields): the role must not change in this or future revisions.
- lighter / darker (amount): raise/lower OKLCH lightness. amount is 0-1; use 0.02-0.08 for subtle steps.
- saturate / desaturate (amount): raise/lower OKLCH chroma. Same scale.
- hue (shift): rotate hue in degrees. Use small values (5-15).
- min_contrast (against, min): raise the target's contrast against a role or #rrggbb color until it reaches min (WCAG ratio; 4.5 is the normal-text floor, 7 is enhanced). The engine moves lightness minimally and preserves hue.

Targets are exact role names from the inventory (e.g. "syntax.comment") or wildcards ("surface.*", "*"). Unmentioned roles never change, so the smallest spec that satisfies the request is the best one.

Translation guidance:
- "keep X" / "don't touch X" -> a protect action for X.
- "readable comments" -> min_contrast on the comment role, against the base surface, min 4.5.
- "darker/lighter background" -> darker/lighter on surface.base (and usually surface.panel / surface.float to keep the surface ladder).
- "more muted/less colorful" -> desaturate on the accent roles the user means; protect surfaces.
- Temperature ("warmer"/"cooler") has no direct operation. Only if the user clearly means specific roles, express it as small hue shifts on those roles (toward 20-100 degrees for warm, 230-280 for cool). If the request is ambiguous, make no hue changes and explain in notes.
- If the request cannot be expressed with these actions, emit an empty action list and explain in notes.

Always answer with JSON matching the schema exactly. Put a one-sentence summary of how you read the request in notes."#
        .to_string()
}

/// The user prompt: theme inventory + measured state + the request.
pub fn user_prompt(theme: &Theme, request: &str) -> String {
    let mut inventory = String::from("Theme role inventory (all targets must come from here):\n");
    for (role, style) in &theme.styles {
        let locked = if style.provenance == crate::ir::style::Provenance::Locked {
            "  [locked]"
        } else {
            ""
        };
        let fg = style.fg.map(|c: Srgb| format!("fg={}", c.to_hex()));
        let bg = style.bg.map(|c: Srgb| format!("bg={}", c.to_hex()));
        let color = fg.or(bg).unwrap_or_else(|| "(no color)".into());
        let _ = writeln!(inventory, "  {} {}{}", role, color, locked);
    }

    let measured = match fingerprint(theme) {
        Ok(fp) => {
            let ladder: Vec<String> = fp
                .contrast_ladder
                .iter()
                .take(8)
                .map(|rc| format!("  {} {:.2}:1", rc.role, rc.ratio))
                .collect();
            format!(
                "Measured state (WCAG contrast vs base {}):\n{}\n  accents: max chroma {:.3}, lightness spread {:.3}",
                fp.base_bg,
                ladder.join("\n"),
                fp.max_accent_chroma,
                fp.accent_lightness_spread
            )
        }
        Err(e) => format!("(fingerprint unavailable: {e})"),
    };

    format!(
        "{inventory}\n{measured}\n\nUser request:\n\"{request}\"\n\nTranslate the request into the edit spec."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::role::{Surface, SyntaxRole, TextRole};
    use crate::ir::style::Style;
    use crate::ir::theme::Appearance;

    fn theme() -> Theme {
        let mut t = Theme::new("gruvbox-ish", Appearance::Dark);
        t.set(
            Role::Surface(Surface::Base),
            Style::bg(Srgb::from_hex("#282828").unwrap()),
        );
        t.set(
            Role::Text(TextRole::Primary),
            Style::fg(Srgb::from_hex("#ebdbb2").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Comment),
            Style::fg(Srgb::from_hex("#928374").unwrap()),
        );
        t.set(
            Role::Syntax(SyntaxRole::Keyword),
            Style::fg(Srgb::from_hex("#fb4934").unwrap()),
        );
        t
    }

    #[test]
    fn flat_actions_convert_to_edit_spec() {
        let llm = LlmSpec {
            notes: Some("darker base, fix comments".into()),
            actions: vec![
                LlmAction {
                    action: "protect".into(),
                    target: "syntax.keyword".into(),
                    amount: None,
                    against: None,
                    min: None,
                    shift: None,
                },
                LlmAction {
                    action: "darker".into(),
                    target: "surface.base".into(),
                    amount: Some(0.04),
                    against: None,
                    min: None,
                    shift: None,
                },
                LlmAction {
                    action: "min_contrast".into(),
                    target: "syntax.comment".into(),
                    amount: None,
                    against: Some("surface.base".into()),
                    min: Some(4.5),
                    shift: None,
                },
            ],
        };
        let spec = llm_spec_to_edit_spec(&llm).unwrap();
        assert_eq!(spec.protect, ["syntax.keyword"]);
        assert_eq!(spec.changes.len(), 2);
        assert_eq!(spec.changes[0].op, Op::Darker { amount: 0.04 });
        assert!(matches!(
            &spec.changes[1].op,
            Op::MinContrast { min: 4.5, .. }
        ));
    }

    #[test]
    fn wildcards_pass_but_garbage_targets_fail() {
        let mk = |target: &str| LlmSpec {
            notes: None,
            actions: vec![LlmAction {
                action: "darker".into(),
                target: target.into(),
                amount: Some(0.02),
                against: None,
                min: None,
                shift: None,
            }],
        };
        assert!(llm_spec_to_edit_spec(&mk("surface.*")).is_ok());
        assert!(llm_spec_to_edit_spec(&mk("*")).is_ok());
        let err = llm_spec_to_edit_spec(&mk("make-it-nicer")).unwrap_err();
        assert!(err.to_string().contains("not a role name"));
    }

    #[test]
    fn validation_is_precise() {
        let action = |action: &str,
                      amount: Option<f64>,
                      min: Option<f64>,
                      shift: Option<f64>,
                      against: Option<&str>|
         -> LlmSpec {
            LlmSpec {
                notes: None,
                actions: vec![LlmAction {
                    action: action.into(),
                    target: "surface.base".into(),
                    amount,
                    against: against.map(String::from),
                    min,
                    shift,
                }],
            }
        };
        // missing amount
        let e = llm_spec_to_edit_spec(&action("darker", None, None, None, None)).unwrap_err();
        assert!(e.to_string().contains("requires `amount`"));
        // amount out of range
        let e = llm_spec_to_edit_spec(&action("darker", Some(1.5), None, None, None)).unwrap_err();
        assert!(e.to_string().contains("outside 0-1"));
        // unknown action
        let e = llm_spec_to_edit_spec(&action("recolor", None, None, None, None)).unwrap_err();
        assert!(e.to_string().contains("unknown action"));
        // min_contrast incomplete
        let e = llm_spec_to_edit_spec(&action("min_contrast", None, None, None, None)).unwrap_err();
        assert!(e.to_string().contains("requires `min`"));
        let e = llm_spec_to_edit_spec(&action("min_contrast", None, Some(4.5), None, None))
            .unwrap_err();
        assert!(e.to_string().contains("requires `against`"));
        // bad against / bad min / bad shift
        let e = llm_spec_to_edit_spec(&action("min_contrast", None, Some(4.5), None, Some("blue")))
            .unwrap_err();
        assert!(e.to_string().contains("neither a role nor"));
        let e = llm_spec_to_edit_spec(&action(
            "min_contrast",
            None,
            Some(99.0),
            None,
            Some("#ffffff"),
        ))
        .unwrap_err();
        assert!(e.to_string().contains("outside 1-21"));
        let e = llm_spec_to_edit_spec(&action("hue", None, None, Some(400.0), None)).unwrap_err();
        assert!(e.to_string().contains("outside -180..180"));
    }

    #[test]
    fn model_output_parses_from_all_shapes() {
        let direct = r#"{"notes":"x","actions":[{"action":"protect","target":"surface.base","amount":null,"against":null,"min":null,"shift":null}]}"#;
        assert!(parse_model_output(direct).is_ok());
        let fenced = "```json\n{ \"notes\": null, \"actions\": [] }\n```";
        assert_eq!(parse_model_output(fenced).unwrap().actions.len(), 0);
        let envelope = r#"{"choices":[{"message":{"content":"{\"notes\":null,\"actions\":[]}"}}]}"#;
        assert!(parse_model_output(envelope).is_ok());
        assert!(parse_model_output("not json at all").is_err());
    }

    #[test]
    fn prompts_carry_inventory_and_request() {
        let sys = system_prompt();
        assert!(sys.contains("never invent colors"));
        assert!(sys.contains("min_contrast"));

        let user = user_prompt(&theme(), "make comments readable");
        assert!(user.contains("syntax.comment fg=#928374"));
        assert!(user.contains("surface.base bg=#282828"));
        assert!(user.contains("make comments readable"));
        assert!(user.contains("Measured state"));
    }

    #[test]
    fn schema_is_strict_shaped() {
        let s = response_schema();
        assert_eq!(s["strict"], true);
        let props = &s["schema"]["properties"]["actions"]["items"]["properties"];
        assert_eq!(
            props["action"]["enum"].as_array().unwrap().len(),
            ACTIONS.len()
        );
        assert!(props["amount"]["type"].is_array()); // nullable
        let required = s["schema"]["properties"]["actions"]["items"]["required"]
            .as_array()
            .unwrap();
        assert!(required.iter().any(|r| r == "against"));
    }

    #[test]
    fn converted_specs_revise_cleanly() {
        // The full offline chain: model output -> edit spec -> revise.
        let raw = r#"{"notes":"darker and readable","actions":[
            {"action":"protect","target":"syntax.keyword","amount":null,"against":null,"min":null,"shift":null},
            {"action":"darker","target":"surface.base","amount":0.04,"against":null,"min":null,"shift":null},
            {"action":"min_contrast","target":"syntax.comment","against":"surface.base","min":4.5,"amount":null,"shift":null}
        ]}"#;
        let llm = parse_model_output(raw).unwrap();
        let spec = llm_spec_to_edit_spec(&llm).unwrap();
        let (revised, report) = crate::revise::revise(&theme(), &spec).unwrap();
        assert!(
            crate::color::contrast_ratio(
                revised.fg(&Role::Syntax(SyntaxRole::Comment)).unwrap(),
                revised.base_bg().unwrap()
            ) >= 4.5
        );
        assert_eq!(
            revised.fg(&Role::Syntax(SyntaxRole::Keyword)),
            theme().fg(&Role::Syntax(SyntaxRole::Keyword))
        );
        assert!(!report.applied.is_empty());
    }
}
