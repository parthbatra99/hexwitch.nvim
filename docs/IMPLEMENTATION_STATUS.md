# Implementation status

Scope as of 2026-10-11: **engine, local browser playground, evaluations**.
Active branch: `work/engine-playground`. Changes are local; no editor adapter or
packaging is included. Older adapter work remains in Git history only.

## Delivered

| Area | Current evidence |
|---|---|
| Color/IR | Deterministic sRGB/OKLCH, gamut mapping, rounded-output contrast, property opacity/state/locks and independent typography |
| Reference import | IR/theme JSONC, bounded includes, per-property lexical/semantic precedence, class/interface retention, qualified losses, pinned Dark+ fixtures/hashes/licenses |
| Revision | Explicit intent v1, minimal targeted edits, semantic targeting, role-referenced backdrops, persistent constraints, final required-floor rollback |
| Interpretation | Compatible API transport, required nullable fields, operation property/value schema, trusted user anchors, explanation/clarification/refusal and local preflight |
| Experiments | Review-before-accept, active-lineage history, undo/redo, stale revision rejection, resumable questions, snapshot hashes, pending-state export/restore with recomputed reports |
| Playground | Before/after sample, controls, lexical/semantic property diff, locks, calculated scenes, color lab, prompt/replay/live modes, inspector and matrix runner |
| Evaluation | 74 authored cases across 50 corpus IDs, state/context variants/sequences, 3 pairs, 13 known-bad probes, wrong-but-valid opacity negative regression |

## Verification

- 153 Rust tests pass, including 9 session tests, 4 loopback playground integrations
  and 3 reconstructed live-failure regression probes.
- 7 Python metadata/rebuild tests pass; exact fixture hashes verified.
- rustfmt and clippy checks pass; browser JavaScript parses.
- Headless Chrome offline smoke passes through real browser controls, preview,
  acceptance/history, semantic diff, matrix and clarification continuation.
- The authored matrix passes 74/74 and 3/3 pairs. This is canned contract evidence,
  not model accuracy. Original held_out labels have been inspected.

## Live observations: incomplete quality gate

Real browser calls to the configured compatible gateway/small model correctly
handled no-italics, comments >=4.5 with background preservation, darker background
with unchanged opacity, and ambiguous-blue questions in multiple bounded runs.
Only bundled synthetic references/requests were sent; no personal code/config.

The full live smoke remains failing: after saturation clarification the model has
proposed opacity edits or absent background targets rather than foreground chroma.
One separate retry hit a transport error. Invalid actions are refused; valid-but-wrong
edits can reach review and are caught by semantic test expectations, not
by structural validation. Exact failure shapes are reconstructed in regression
probes without claiming authenticated/full provider artifacts.

Provider-schema bounds and prompt guidance were improved, but no general model
quality or trustworthy-adaptation gate is declared complete. Details and commands:
[engine-playground.md](packages/engine-playground.md).

## Removed and deferred

Removed current/legacy editor clients, editor emitters, installed-highlight tests,
editor launcher and editor CI jobs. Personal editor configuration is untouched.
No release/install packaging was added. Deferred: dynamic provider configuration,
cancellation, durable recovery, richer scenes/code samples, independent unseen
annotations, broad model comparison, relational identity objectives, human
preference/readability studies and runtime editor conformance.
