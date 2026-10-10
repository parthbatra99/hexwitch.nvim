# Evaluation suites

Keep seven results separate: color correctness, import fidelity, revision
correctness, interpretation quality, scene readability, adapter fidelity, human
preference/intent preservation. A schema pass is not semantic accuracy, and a
contrast pass is not a comfort or productivity result.

`manifest.json` is the legacy metric regression set. Its six theme files are
curated excerpts; do not describe them as complete faithful imports. Upstream
revision/license verification remains work before adding them to a release
fidelity corpus. `import/manifest.json` adds a complete pinned Dark+ reference
and included base with original upstream MIT notice, plus authored semantic
success/failure fixtures. Explicit scope probes are not full editor rendering.

`corpus-v1.json` contains 50 authored annotated requests: 30 development and 20
held-out. Required, forbidden, and acceptable outcomes are semantic judgments,
not exact model JSON or exact hex goldens. Do not tune held-out expectations to
the current engine. Each model run should record provider/model/prompt hash,
engine version, input reference hash, seed/config, raw response stored privately,
validated interpretation, final revision/scene reports, and categorized failures.
Offline fixtures and opt-in network runs are separate.

For each new fixture, record:

```json
{
  "version":1,
  "id":"unique-id",
  "suite":"import_fidelity",
  "origin":"source URL or authored locally",
  "license":"verified SPDX expression or unknown (not release eligible)",
  "revision":"upstream commit/tag or authored revision",
  "sha256":"hash of exact input bytes",
  "purpose":"what behavior this fixture distinguishes",
  "required":["semantic outcome"],
  "forbidden":["regression"],
  "acceptable":["qualified no-op or failure"],
  "limitations":["what this fixture cannot demonstrate"]
}
```

This is the initial contribution specification, not yet a complete enforced
fixture schema or a completed model comparison. Independent reviewers should
annotate outcomes without seeing the system output.

Import fixture metadata and exact byte hashes are enforced offline:

```sh
python3 core/eval/verify-fixtures.py
python3 -m unittest discover -s core/eval -p 'test_fixture_metadata.py'
```

The validator rejects missing fields, unknown metadata, changed hashes and path
escapes; CI runs it alongside the Rust suites. The six legacy excerpts still
have unverified upstream license provenance and are not promoted to this suite.

## Executable interpretation evaluation

`interpretation-suite-v1.json` defines 12 machine-checkable cases from the
50-request corpus (eight development, four held-out). Predicates judge preservation,
allowed changes, locks, typography resets, opacity, trusted anchors, contrast,
strength and clarification. `interpretation-contract-replay.json` is deliberately
hand-authored, not live model output. Rebuild it with build-contract-replay.py.

```sh
hexwitch --machine eval-interpretation core/eval/interpretation-suite-v1.json \
  --run core/eval/interpretation-contract-replay.json
```

To compare a model, collect responses separately with explicit provider consent,
label run.kind=recorded_provider and record provider/model/prompt_version. Supply
case_id and raw response strings or decoded JSON. The evaluator does not call a
network provider; absent responses are failures, not silently skipped cases.
Reports include hashes and failure classes; run provenance is caller-supplied.
Do not describe the canned 12/12 as model accuracy or full held-out coverage.

## Human evaluation protocol (not yet run)

Ask separately: preserved loved treatments, understood requested change, rounds
to acceptance, surprising unrelated changes, retained theme at follow-up,
readability on a specified task, and sustained comfort. Record baseline reference,
requested changes, protected properties, candidate order (randomized), acceptance,
and qualitative reason. Obtain consent and minimize personal source/image data.

Specific readability claims need controlled matched code/tasks, counterbalanced
conditions, measured error/time outcomes, and recorded accessibility/display
conditions. Preference and theme retention do not justify productivity claims.
No aggregate score should hide identity preservation versus readability tradeoffs.
