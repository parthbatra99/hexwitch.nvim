# Status

Scope: the Rust engine, the local browser playground and the evaluation suites.
Editor adapters, emitters and release packaging are out of scope. Direction and
next steps are in [FUTURE_PLAN.md](../FUTURE_PLAN.md).

## Working today

| Area | What exists |
|---|---|
| Color and IR | Deterministic sRGB/OKLCH, gamut mapping, contrast on rounded output, per-property opacity, states and locks, independent typography |
| Import | IR and theme JSONC, bounded includes, per-property lexical and semantic precedence, class/interface retention, reported losses, pinned Dark+ fixtures with hashes and licenses |
| Revision | Intent v1, minimal targeted edits, semantic targeting, role-referenced backdrops, persistent constraints, rollback when a required floor fails |
| Interpretation | OpenAI-compatible transport, strict per-operation schema, trusted user color anchors, clarification and refusal, local preflight |
| Experiments | Review before accept, undo/redo over accepted lineage, stale-revision rejection, resumable questions, export and restore with recomputed reports |
| Playground | Preview with nudges, property diff, locks, calculated scenes, contrast checker, provider registry, live status with cancel, inspector, replay and matrix runner |
| Evaluation | 74 authored cases over 50 corpus requests, state variants, sequences, 3 paired checks, 13 known-bad probes |

Offline verification is listed in the README. The authored matrix passes 74/74
cases and 3/3 pairs, but those are hand-written replays: they test the evaluator,
not a model. The held_out labels have been inspected, so they are not a blind
holdout either.

## Live model quality is not settled

Browser runs against a small model on a compatible gateway handled several
requests correctly: removing italics, raising comment contrast to 4.5 while
keeping the background, darkening the background without touching opacity, and
asking a question about an ambiguous "blue". Only bundled synthetic references
and prompts were sent.

The full live smoke still fails. After a saturation clarification, the model has
proposed opacity edits or a missing background target instead of a foreground
chroma change. Invalid actions are refused locally, but a valid wrong edit can
reach review; the semantic evaluator catches it, the schema does not. Those
failures are kept as regression probes. Details are in
[playground.md](playground.md#live-model-runs).

## Not done

Durable recovery, richer scenes and code samples, independent annotations on
unseen requests, comparing several models, relational identity operations, human
preference and readability studies, and any editor runtime conformance.
