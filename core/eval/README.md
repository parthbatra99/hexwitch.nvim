# Evaluation suites

Keep seven results separate: color correctness, import fidelity, revision
correctness, interpretation quality, scene readability, adapter fidelity, human
preference/intent preservation. A schema pass is not semantic accuracy, and a
contrast pass is not a comfort or productivity result.

`manifest.json` is the legacy metric regression set. Its six theme files are
curated excerpts; do not describe them as complete faithful imports. Upstream
revision/license verification remains work before adding them to a release
fidelity corpus.

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
