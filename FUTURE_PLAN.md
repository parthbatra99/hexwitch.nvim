# Engine-first direction

Hexwitch has three parts: a Rust engine library, a scriptable CLI and a local
browser playground. Editor adapters, emitters and packaging stay out of scope
until the engine and the experiment workflow are reliable.

## Next evidence gates

1. Interpretation quality: run independently annotated, unseen requests against
   several compatible models. Record exact prompts, responses, references,
   settings and the variance across repeated runs. Report refusal and
   clarification, intent preservation, unrelated changes and sequential
   consistency separately.
2. Clarification quality: an answer about the target or property should resolve
   the ambiguity without changing opacity, typography or unrelated roles. Recent
   small-model answers to the saturation question break this rule; keep them as
   negative observations.
3. Experiment durability: raise bundle size limits, add migrations, crash
   recovery, an explicit reset and a confirmation before switching references.
   Sessions live in memory today, with manual export and restore, and hashes are
   not signatures.
4. Validation controls: configurable calculated scenes, semantic context probes,
   input for unknown external backdrops, and a comparison of required against
   advisory checks. Calculated contrast is not comfort or visual fidelity.
5. Engine capabilities: relational prominence, explicit identity objectives,
   richer reference inputs and bounded constraint solving. Keep returning
   unsupported for these until each operation has testable semantics.
6. Human evaluation: measure surprising edits, iterations to acceptance,
   preserved treatments the user liked, and task-specific readability as separate
   results. Neither a preference nor a WCAG ratio supports a productivity claim.

## Playground refinements

- Safer controls for new references and reset, and more than one experiment per host.
- Editable samples and more role and semantic scene probes.
- Structured questions with choices, not only free text.
- Turning an experiment into an evaluation case with independently written expectations.
- Reproducible collection of provider runs, private raw artifacts and model comparison.
- A complete keyboard workflow and wider browser test coverage.

## Deferred

Adapters for Neovim, VS Code, Zed or other editors, live editor capture, artifact
emission and release packaging are not being developed. Bring them back only when
the shared engine and session contracts and the quality gates are strong enough
to justify thin clients. Browser previews do not test editor fidelity, and
importing theme JSON does not show that an editor renders it the same way.
