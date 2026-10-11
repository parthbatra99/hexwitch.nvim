use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Satisfied,
    Unchanged,
    Refused,
    Infeasible,
    NeedsClarification,
    Partial,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidInput,
    UnsupportedVersion,
    MissingTarget,
    Io,
    ProviderUnavailable,
    ProviderRefusal,
    Timeout,
    Cancelled,
    RateLimited,
    PayloadTooLarge,
    UnsupportedCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response<T> {
    pub protocol_version: u32,
    pub request_id: String,
    pub operation: String,
    pub outcome: Outcome,
    pub result: Option<T>,
    pub errors: Vec<ProtocolError>,
    pub warnings: Vec<String>,
}

impl<T> Response<T> {
    pub fn success(
        request_id: impl Into<String>,
        operation: impl Into<String>,
        outcome: Outcome,
        result: T,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id: request_id.into(),
            operation: operation.into(),
            outcome,
            result: Some(result),
            errors: vec![],
            warnings: vec![],
        }
    }
    pub fn failure(
        request_id: impl Into<String>,
        operation: impl Into<String>,
        code: ErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id: request_id.into(),
            operation: operation.into(),
            outcome: Outcome::Error,
            result: None,
            errors: vec![ProtocolError {
                code,
                message: message.into(),
            }],
            warnings: vec![],
        }
    }
}

pub fn capabilities() -> serde_json::Value {
    serde_json::json!({
        "protocol_version": PROTOCOL_VERSION,
        "engine_version": env!("CARGO_PKG_VERSION"),
        "ir_version": crate::IR_VERSION,
        "ir_features": ["property_opacity", "property_locks", "semantic_type_selectors", "semantic_priority", "independent_strikethrough"],
        "intent_features": ["include_semantic", "role_backdrops", "semantic_requirement_contexts", "rollback_report"],
        "interpretation_versions": [0,1],
        "interpretation_prompt_version": crate::interpret_v1::PROMPT_VERSION,
        "interpretation_schema_revision": 3,
        "interpretation_schema_features": ["operation_specific_nullability", "request_anchor_enum", "operation_property_enums", "numeric_delta_bounds"],
        "experiment_version": 1,
        "experiment_features": ["review_before_accept", "resumable_clarification", "optimistic_revisions", "active_lineage_context", "snapshot_hashes", "pending_candidate_export"],
        "interpretation_eval": "recorded_semantic_outcomes",
        "evaluation_features": ["state_setup", "sequential_cases", "acceptable_alternatives", "paired_consistency", "proposal_guards", "scorecard_denominators"],
        "operations": ["import", "interpret", "revise", "validate", "eval_interpretation"],
        "edit_spec_versions": [0, 1],
        "provider_transport": {"api":"openai_compatible_chat_completions","base_url_configurable":true,"url_schemes":["http","https"],"http_warning":true,"api_key_env_configurable":true,"structured_output":["json_schema","json_object","none"],"automatic_retry":false},
        "editor_adapters": [],
        "offline": true,
        "limitations": ["Scene calculations are not real-editor render verification", "Live provider and human preference evaluations are opt-in", "Cancellation is not yet supported; structured-output capability must be selected explicitly"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn success_and_error_are_machine_readable() {
        let ok = Response::success(
            "r-1",
            "revise",
            Outcome::Partial,
            serde_json::json!({"changed":1}),
        );
        let value = serde_json::to_value(ok).unwrap();
        assert_eq!(value["outcome"], "partial");
        assert_eq!(value["request_id"], "r-1");
        let err = Response::<()>::failure(
            "r-2",
            "import",
            ErrorCode::UnsupportedVersion,
            "unsupported IR version",
        );
        let value = serde_json::to_value(err).unwrap();
        assert_eq!(value["errors"][0]["code"], "unsupported_version");
        assert!(value["result"].is_null());
    }
}
