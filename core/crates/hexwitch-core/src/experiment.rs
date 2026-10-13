use crate::intent::{revise_intent, IntentSpec};
use crate::interpret_v1::{self, Decision, IterationContext, ModelInterpretation};
use crate::ir::{Property, Style, Theme};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Version {
    pub theme: Theme,
    pub label: String,
    pub spec: Option<IntentSpec>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub theme: Theme,
    pub label: String,
    pub spec: IntentSpec,
    pub report: Value,
    pub validation: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clarification {
    pub request: String,
    pub questions: Vec<String>,
    pub notes: String,
    pub revision: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub kind: String,
    pub request: Option<String>,
    pub data: Value,
}
#[derive(Debug, Clone, Serialize)]
pub struct Experiment {
    pub revision: u64,
    pub versions: Vec<Version>,
    pub cursor: usize,
    pub candidate: Option<Candidate>,
    pub clarification: Option<Clarification>,
    pub last_result: Option<Value>,
    pub events: Vec<Event>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub version: u32,
    pub engine_version: String,
    pub prompt_version: String,
    pub versions: Vec<Version>,
    pub theme_hashes: Vec<String>,
    pub cursor: usize,
    pub candidate: Option<Candidate>,
    pub clarification: Option<Clarification>,
    pub last_result: Option<Value>,
    pub events: Vec<Event>,
    pub limitations: Vec<String>,
}
fn hash(theme: &Theme) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(theme).expect("validated IR serializes"))
    )
}
fn resolved(theme: &Theme) -> std::collections::BTreeMap<String, Style> {
    theme
        .styles
        .keys()
        .filter_map(|role| {
            crate::validate::resolved_style(theme, role).map(|style| (role.to_string(), style))
        })
        .collect()
}
fn changes(
    role: &str,
    semantic_rule: Option<usize>,
    before: &Style,
    after: &Style,
    result: &mut Vec<Value>,
) {
    for property in Property::ALL {
        let mut a = Style::new();
        a.copy_property_from(before, property);
        let mut b = Style::new();
        b.copy_property_from(after, property);
        if a != b || before.is_locked(property) != after.is_locked(property) {
            result.push(json!({"role":role,"semantic_rule":semantic_rule,"property":property,"before":a,"after":b,"locked_before":before.is_locked(property),"locked_after":after.is_locked(property)}));
        }
    }
}
impl Experiment {
    pub fn new(theme: Theme) -> Result<Self, String> {
        theme.validate()?;
        Ok(Self {
            revision: 0,
            versions: vec![Version {
                theme,
                label: "Reference".into(),
                spec: None,
            }],
            cursor: 0,
            candidate: None,
            clarification: None,
            last_result: None,
            events: vec![],
        })
    }
    pub fn current(&self) -> &Theme {
        &self.versions[self.cursor].theme
    }
    pub fn check_revision(&self, revision: u64) -> Result<(), String> {
        if self.revision != revision {
            Err("stale experiment revision; reload before continuing".into())
        } else {
            Ok(())
        }
    }
    pub fn check_prompt(&self, revision: u64) -> Result<(), String> {
        self.check_revision(revision)?;
        if self.candidate.is_some() {
            return Err("accept or reject the candidate before another request".into());
        }
        Ok(())
    }
    pub fn record(&mut self, kind: &str, request: Option<String>, data: Value) {
        self.events.push(Event {
            kind: kind.into(),
            request,
            data,
        });
        if self.events.len() > 512 {
            self.events.remove(0);
        }
    }
    pub fn import(&mut self, theme: Theme, revision: u64) -> Result<(), String> {
        self.check_revision(revision)?;
        theme.validate()?;
        self.versions = vec![Version {
            label: format!("Reference: {}", theme.meta.name),
            theme,
            spec: None,
        }];
        self.cursor = 0;
        self.candidate = None;
        self.clarification = None;
        self.last_result = None;
        self.events.clear();
        self.revision += 1;
        self.record("import", None, json!({"theme_hash":hash(self.current())}));
        Ok(())
    }
    pub fn propose(
        &mut self,
        spec: IntentSpec,
        label: String,
        revision: u64,
    ) -> Result<Value, String> {
        self.check_prompt(revision)?;
        if label.len() > 8192 {
            return Err("edit label exceeds 8192 bytes".into());
        }
        let (theme, report) = revise_intent(self.current(), &spec)?;
        let report = json!(report);
        let validation = json!(crate::validate::validate_scenes(
            &theme,
            &crate::validate::initial_scenes()
        )?);
        self.revision += 1;
        self.clarification = None;
        self.last_result =
            Some(json!({"outcome":report["outcome"],"report":report,"validation":validation}));
        self.record("revision", Some(label.clone()), json!({"spec":spec,"report":report,"reference_hash":hash(self.current()),"candidate_hash":hash(&theme)}));
        if report["rolled_back"] != true
            && matches!(report["outcome"].as_str(), Some("satisfied" | "partial"))
        {
            self.candidate = Some(Candidate {
                theme,
                label,
                spec,
                report,
                validation,
            });
        }
        Ok(self.last_result.clone().unwrap())
    }
    pub fn context(&self) -> IterationContext {
        let mut context = IterationContext::default();
        for version in self.versions.iter().take(self.cursor + 1).skip(1) {
            context.previous_requests.push(version.label.clone());
            if let Some(spec) = &version.spec {
                context.previous_actions.extend(spec.actions.clone());
            }
        }
        if context.previous_requests.len() > 16 {
            context
                .previous_requests
                .drain(..context.previous_requests.len() - 16);
        }
        if context.previous_actions.len() > 64 {
            context
                .previous_actions
                .drain(..context.previous_actions.len() - 64);
        }
        context
    }
    pub fn interpret(
        &mut self,
        request: String,
        model: ModelInterpretation,
        revision: u64,
    ) -> Result<Value, String> {
        self.check_prompt(revision)?;
        let result = interpret_v1::validate(&model, self.current(), &request)?;
        let data = json!(result);
        if result.decision == Decision::NeedsClarification {
            self.revision += 1;
            self.clarification = Some(Clarification {
                request: request.clone(),
                questions: result.questions,
                notes: result.notes,
                revision: self.revision,
            });
            self.last_result = Some(data.clone());
        } else if let Some(spec) = result.spec {
            self.propose(spec, request.clone(), revision)?;
        } else {
            self.revision += 1;
            self.clarification = None;
            self.last_result = Some(data.clone());
        }
        self.record(
            "interpretation",
            Some(request),
            json!({"model_response":model,"result":data}),
        );
        Ok(data)
    }
    pub fn answer_request(&self, answer: &str, revision: u64) -> Result<String, String> {
        self.check_revision(revision)?;
        let pending = self
            .clarification
            .as_ref()
            .ok_or("no pending clarification")?;
        if answer.trim().is_empty() || answer.len() > 2048 {
            return Err("clarification answer must be 1..2048 bytes".into());
        }
        let request = format!(
            "Original request: {}\nUser answer: {}",
            pending.request, answer
        );
        if request.len() > 8192 {
            return Err("combined request exceeds interpretation limit".into());
        }
        Ok(request)
    }
    pub fn accept(&mut self, revision: u64, allow_unverified: bool) -> Result<(), String> {
        self.check_revision(revision)?;
        let candidate = self.candidate.as_ref().ok_or("no candidate")?;
        if candidate.validation["status"] != "pass" && !allow_unverified {
            return Err(
                "candidate has failed or unverified scenes; explicit acceptance required".into(),
            );
        }
        if self.cursor >= 127 {
            return Err("session version limit; export and start a new experiment".into());
        }
        let candidate = self.candidate.take().unwrap();
        self.record("accept", Some(candidate.label.clone()), json!({"spec":candidate.spec,"validation":candidate.validation,"allow_unverified":allow_unverified}));
        self.versions.truncate(self.cursor + 1);
        self.versions.push(Version {
            theme: candidate.theme,
            label: candidate.label,
            spec: Some(candidate.spec),
        });
        self.cursor = self.versions.len() - 1;
        self.revision += 1;
        Ok(())
    }
    pub fn reject(&mut self, revision: u64) -> Result<(), String> {
        self.check_revision(revision)?;
        self.candidate = None;
        self.clarification = None;
        self.last_result = None;
        self.revision += 1;
        self.record("reject", None, json!({}));
        Ok(())
    }
    pub fn navigate(&mut self, redo: bool, revision: u64) -> Result<(), String> {
        self.check_revision(revision)?;
        if self.candidate.is_some() || self.clarification.is_some() {
            return Err("resolve candidate/clarification before navigating".into());
        }
        if redo {
            if self.cursor + 1 >= self.versions.len() {
                return Err("nothing to redo".into());
            }
            self.cursor += 1;
        } else {
            if self.cursor == 0 {
                return Err("nothing to undo".into());
            }
            self.cursor -= 1;
        }
        self.last_result = None;
        self.revision += 1;
        self.record(
            if redo { "redo" } else { "undo" },
            None,
            json!({"cursor":self.cursor}),
        );
        Ok(())
    }
    pub fn bundle(&self) -> Bundle {
        Bundle {
            version: 1,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            prompt_version: interpret_v1::PROMPT_VERSION.into(),
            versions: self.versions.clone(),
            theme_hashes: self.versions.iter().map(|v| hash(&v.theme)).collect(),
            cursor: self.cursor,
            candidate: self.candidate.clone(),
            clarification: self.clarification.clone(),
            last_result: self.last_result.clone(),
            events: self.events.clone(),
            limitations: vec![
                "Browser preview is a portable role visualization, not editor conformance".into(),
                "Prompts/responses may contain personal data; review before sharing".into(),
                "SHA-256 hashes detect snapshot changes, not authenticity; logs are untrusted data"
                    .into(),
            ],
        }
    }
    pub fn restore(&mut self, bundle: Bundle, revision: u64) -> Result<(), String> {
        self.check_revision(revision)?;
        if bundle.version != 1
            || bundle.versions.is_empty()
            || bundle.versions.len() > 128
            || bundle.cursor >= bundle.versions.len()
            || bundle.events.len() > 512
            || bundle.theme_hashes.len() != bundle.versions.len()
            || (bundle.candidate.is_some() && bundle.clarification.is_some())
        {
            return Err("invalid experiment bundle".into());
        }
        for (index, (version, expected)) in
            bundle.versions.iter().zip(&bundle.theme_hashes).enumerate()
        {
            version.theme.validate()?;
            if version.label.len() > 8192 || hash(&version.theme) != *expected {
                return Err("invalid snapshot label or hash".into());
            }
            if index == 0 {
                if version.spec.is_some() {
                    return Err("reference snapshot cannot have a revision spec".into());
                }
            } else {
                let spec = version
                    .spec
                    .as_ref()
                    .ok_or("accepted snapshot requires its revision spec")?;
                let (replayed, report) = revise_intent(&bundle.versions[index - 1].theme, spec)?;
                if report.rolled_back || hash(&replayed) != *expected {
                    return Err("accepted snapshot does not match its revision lineage".into());
                }
            }
        }
        let reference = &bundle.versions[bundle.cursor].theme;
        let candidate = if let Some(mut candidate) = bundle.candidate {
            let (theme, report) = revise_intent(reference, &candidate.spec)?;
            if hash(&theme) != hash(&candidate.theme)
                || report.rolled_back
                || !matches!(
                    report.outcome,
                    crate::protocol::Outcome::Satisfied | crate::protocol::Outcome::Partial
                )
                || candidate.label.len() > 8192
            {
                return Err("candidate cannot be reproduced from bundle intent".into());
            }
            candidate.report = json!(report);
            candidate.validation = json!(crate::validate::validate_scenes(
                &theme,
                &crate::validate::initial_scenes()
            )?);
            Some(candidate)
        } else {
            None
        };
        if let Some(pending) = &bundle.clarification {
            interpret_v1::validate(
                &ModelInterpretation {
                    version: 1,
                    decision: Decision::NeedsClarification,
                    notes: pending.notes.clone(),
                    questions: pending.questions.clone(),
                    actions: vec![],
                },
                reference,
                &pending.request,
            )?;
        }
        self.versions = bundle.versions;
        self.cursor = bundle.cursor;
        self.events = bundle.events;
        self.candidate = candidate;
        self.clarification = bundle.clarification;
        self.last_result = bundle.last_result;
        self.revision += 1;
        if let Some(pending) = &mut self.clarification {
            pending.revision = self.revision;
        }
        Ok(())
    }
    pub fn view(&self) -> Result<Value, String> {
        let current = self.current();
        let validation =
            crate::validate::validate_scenes(current, &crate::validate::initial_scenes())?;
        let analysis = crate::analyze::fingerprint(current).ok();
        let mut diff = vec![];
        if let Some(candidate) = &self.candidate {
            for (role, before) in &current.styles {
                if let Some(after) = candidate.theme.get(role) {
                    changes(&role.to_string(), None, before, after, &mut diff);
                }
            }
            for (index, before) in current.modifiers.iter().enumerate() {
                if let Some(after) = candidate.theme.modifiers.get(index) {
                    changes(
                        &before.role.to_string(),
                        Some(index),
                        &before.style,
                        &after.style,
                        &mut diff,
                    );
                }
            }
        }
        Ok(
            json!({"revision":self.revision,"current":current,"resolved":resolved(current),"candidate_resolved":self.candidate.as_ref().map(|c|resolved(&c.theme)),"candidate":self.candidate,"clarification":self.clarification,"last_result":self.last_result,"changes":diff,"analysis":analysis,"validation":validation,"history":self.versions.iter().enumerate().map(|(id,v)|json!({"id":id,"label":v.label,"active":id==self.cursor})).collect::<Vec<_>>(),"events":self.events,"can_undo":self.cursor>0,"can_redo":self.cursor+1<self.versions.len()}),
        )
    }
}
