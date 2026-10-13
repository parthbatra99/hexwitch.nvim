# Experiment v1

`hexwitch_core::experiment::Experiment` does not depend on a provider, the file
system or an editor. The loopback browser host calls this library directly; it
does not parse CLI output.

## State and revision

An experiment holds accepted/reference `versions`, an active `cursor`, a
`revision` that only increases, an optional `candidate` or `clarification` (never
both), a `last_result` and a bounded event log.

Every mutating HTTP request sends the revision it saw. Imports, accept/reject,
undo/redo, interpretation and restore all invalidate older revisions. A provider
call snapshots the reference and context, releases the session lock during network
I/O, and checks the revision again before keeping its result, so a stale response
cannot change a newer reference. Only one live request runs at a time. It is never
retried automatically; the host can cancel it through `POST /api/live/cancel`.

## Review

`propose` runs `revise_intent` and calculated scene validation. The accepted theme
does not change until `accept`. Rollback after a failed required floor, refusal,
infeasibility and no-op results keep their reports but produce no candidate. A
partial candidate can still contain refused operations, so read its report.

Accepting a candidate whose calculated scenes failed or are unverified needs
`allow_unverified=true`. That acknowledgement says nothing about intent or
readability. A structurally valid but unrelated model action can still reach
review, because keyword heuristics cannot establish what the user meant. The
browser shows property and semantic patch differences, including changes that
only touch locks.

Undo and redo move the cursor. Accepting a new branch drops the abandoned redo
versions but keeps them in the audit log. Language context comes only from the
active accepted lineage, never from rejected proposals, undone actions or
abandoned branches.

## Clarification

A pending clarification keeps the user's request, the model's questions and
notes, and the revision. Continuing sends the original request together with the
user's answer. Model questions are untrusted prompt data and cannot grant
`set_color` anchor authority. On success the question is cleared and the result
is a candidate or a qualified outcome. Parse or intent validation errors keep the
question open so the user can correct the answer. A new reference invalidates
pending answers.

## Export and restore

Bundle fields: `version=1`, engine and prompt versions, accepted versions with
their intent lineage, SHA-256 theme hashes, cursor, pending candidate or question,
last result, events and limitations. The host writes no credentials into a
bundle. Live events record the model identifier, system and user prompts and the
schema. Logs and model output can contain personal content, so review a bundle
before sharing it.

Restore checks limits and the IR, verifies snapshot hashes, and rebuilds any
pending candidate from the current reference plus its intent. It recomputes the
candidate report and scenes instead of trusting uploaded pass flags. A pending
question is validated and moved onto a new revision. Hashes catch accidental
changes to snapshots but do not prove where a bundle came from, and labels and
events inside it are treated as data, not instructions.

Limits: 128 versions, 512 events, an 8 MiB inbound HTTP body, plus the engine and
prompt bounds. This is a local format, not a signed benchmark or a migration
framework, and a large exported session can exceed the restore limit.

## Local HTTP boundary

The host binds `127.0.0.1`, rejects any other Host value, and requires the exact
Origin, a JSON content type and a random per-process token on every POST. Static
assets and bootstrap use GET. There are no CORS grants and no route that serves
arbitrary files. Header, body, connection-count and read-time limits apply. Local
processes with enough access can read the token, so this does not protect against
local malware.

The browser sample is a portable visualization of lexical roles. Semantic patches
are tested by the engine, not inferred from the sample's JavaScript tokens.
Calculated validation holds for its declared profile only, and does not prove how
any editor orders overlays or renders fonts.
