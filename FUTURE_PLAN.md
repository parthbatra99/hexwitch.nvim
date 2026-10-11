# Engine-first direction

Hexwitch currently has three surfaces: a Rust engine library, a scriptable CLI,
and a local browser playground. Editor adapters, emitters and packaging are out of
scope until engine behavior and the experiment experience are reliable.

## Next evidence gates

1. **Interpretation quality:** run independently annotated unseen requests against
   multiple compatible models; record exact prompts, responses, references,
   settings and repeated-run variance. Keep refusal/clarification, intent
   preservation, unrelated changes and sequential consistency separate.
2. **Clarification quality:** target/property answers should resolve ambiguity
   without changing opacity, typography or unrelated roles. Recent live small-model
   saturation answers violate this; retain those as negative observations.
3. **Experiment durability:** improve bundle size limits, migrations, crash recovery,
   explicit reset and reference switching confirmation. Current storage is in
   memory with manual export/restore; hashes are not signatures.
4. **Validation controls:** configurable calculated scenes, semantic context probes,
   unknown external backdrop input and comparison of required versus advisory
   checks. Do not equate calculated contrast with comfort or visual fidelity.
5. **Engine capabilities:** relational prominence, explicit identity objectives,
   richer reference inputs and bounded constraint solving. Preserve unsupported
   outcomes until these operations have testable semantics.
6. **Human evaluation:** measure surprising edits, iterations to acceptance,
   preserved loved treatments, and task-specific readability separately. No
   productivity claim follows from a preference or WCAG ratio.

## Playground refinements

- Safer new-reference/reset controls and multiple independent experiments.
- Editable samples and more role/semantic scene probes.
- Better structured questions/choices rather than free text alone.
- Provider progress and explicit cancellation with stale-response rejection.
- Experiment-to-evaluation conversion with independently authored expectations.
- Reproducible provider-run collection, private raw artifacts and model comparison.
- Accessible keyboard workflow and broader browser tests.

## Deferred

Neovim, VS Code, Zed or other editor adapters, live editor capture, artifact
emission and release/install packaging are intentionally not being developed.
Reintroduce them only after the shared engine/session contracts and quality gates
are strong enough to justify thin clients. Browser previews are not their fidelity
tests, and theme JSON import is not evidence of runtime editor conformance.
