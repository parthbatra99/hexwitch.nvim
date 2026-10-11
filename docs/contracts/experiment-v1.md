# Experiment v1

`hexwitch_core::experiment::Experiment` is provider-, file- and editor-independent.
The loopback browser host calls this same library; it does not parse CLI output.

## State and revision

An experiment has accepted/reference `versions`, an active `cursor`, a monotonically
increasing `revision`, optional `candidate` or `clarification`, a `last_result`,
and a bounded event log. A candidate and clarification cannot coexist.

Every mutating HTTP request supplies the revision it observed. Imports,
accept/reject, undo/redo, interpretation and restore invalidate old revisions.
A provider call snapshots reference/context and releases the session lock during
network I/O, then checks revision again before retaining its result. A stale
response cannot mutate a newer reference. Only one live request is allowed at a
time; it is not automatically retried or canceled.

## Revision and review

`propose` runs `revise_intent` and calculated scene validation. The accepted theme
is unchanged until `accept`. Required-floor rollback, refusal, infeasibility and
no-op results retain reports without an acceptable candidate. Partial candidates
can still contain refused operations; inspect their report.

Acceptance of failed/unverified calculated scenes requires `allow_unverified=true`.
This acknowledgement is not evidence of semantic intent or readability. A
structurally valid but unrelated model action can still reach review; keyword
heuristics do not establish user intent. The browser shows property and semantic
patch differences, including lock-only changes.

Undo/redo moves the active cursor. Accepting a new branch removes abandoned redo
versions but retains the audit log. Language context is built only from active
accepted lineage, never rejected proposals, undone actions or abandoned branches.

## Clarification

Pending clarification retains user-authored request, model questions/notes and
revision. Continuation combines original user request with user answer. Model
questions are separate untrusted prompt data: they must not grant `set_color`
anchor authority. Success clears the question and creates a candidate or qualified
result. Parse/intent validation errors leave the question available for correction.
A fresh reference invalidates pending answers.

## Export and restore

Bundle fields: `version=1`, engine/prompt versions, accepted versions with intent
lineage, SHA-256 theme hashes, cursor, pending candidate/question, last result,
events and limitations. The host stores no credentials in these fields. Live event
records include model identifier, system/user prompts and schema; logs and model
output may contain personal content and must be reviewed before sharing.

Restore validates limits and IR, verifies snapshot hashes, and reproduces a pending
candidate from current reference + intent. It recomputes candidate report/scenes
rather than trusting uploaded pass flags. A pending question is validated and
rebased onto a new revision. Hashes detect accidental snapshot changes; they do
not authenticate provenance. Historical labels/events are data, not instructions.

Limits: 128 versions, 512 events, 8 MiB inbound HTTP body, existing engine/prompt
bounds. This is a local experiment format, not a signed benchmark or a migration
framework. A large exported session may exceed the inbound restore limit.

## Local HTTP boundary

The host binds `127.0.0.1`, rejects other Host values, and requires exact Origin,
JSON content type and a random per-process token for POST. Static assets and
bootstrap use GET; no CORS grants or arbitrary file-serving routes exist. Header,
body, connection-count and read-time bounds apply. Trusted local processes can
read the token: this is not an authentication boundary against local malware.

Browser sample rendering remains a portable lexical-role visualization. Semantic
patches are inspectable and tested by the engine, not inferred from the sample's
JavaScript tokens. Calculated validation is authoritative for its declared profile,
not proof of any editor's overlay ordering or font rendering.
