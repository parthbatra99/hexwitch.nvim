# Future-plan implementation evidence

Planning source: `FUTURE_PLAN.md`. This file is the live evidence ledger.

## Checkpoints

- Baseline `fce4841`: preserved original adapter relocation and thin-client work;
  84 offline tests green, Rust format/clippy green, Neovim 0.12.5 locally.
- First-wave implementation: `c7de02c` on `work/p1-contracts`.
- Import/semantic fidelity checkpoint: `b72fabe` on `work/p2-import-fidelity`.
- Property-aware interpretation checkpoint: `abca8f7` on `work/p2-interpretation-quality`.
- Active branch: `work/p2-eval-matrix`. Delegated branches/worktrees were created
  from the baseline, but all agent launches failed before execution because no
  deployment was available. No delegated package is counted as implemented.

## Packages delivered or in progress

| Package | State | Evidence / remaining gate |
|---|---|---|
| P0.1 | Checkpointed | Baseline commit; original legacy tree retained; local engine/Neovim tests run; legacy dependency suite still CI-only |
| P0.2 | Delivered | README explicitly classifies prototype/import/gamut/search/rendering limits |
| P1.1 | Initial implementation | IR v1 opacity, independent property locks/origin, states, modifiers, relationships and passthrough; explicit v0 migration; contract tests. Consumer fidelity remains limited; direct Theme serde deserialization also validates |
| P1.2 | Initial implementation | `intent.rs`: exact property targets, role collections, local/persistent locks, unlock, required contrast rollback, preferences reported separately. Optional include_semantic targets and independent strikethrough supported. Preferences are not yet solver objectives; relational edit operations pending |
| P1.3 | Initial implementation | `--machine --request-id`, stable envelope/codes, capabilities, import/interpret/revise/validate/emit; atomic single-artifact replacement and output collision checks. Full client adoption and multi-file recovery pending |
| P1.4 | Initial specification | Seven-suite contribution format and 50-request semantic corpus; Import corpus metadata/path/hash validation enforced in CI; original excerpt licenses remain unverified |
| P2.1 | Partial | Strict model fields/actions, selector validation, actual-theme target validation, numerical validation, opacity/blend/version checks; Duplicate JSON keys and malformed unmapped/shadowed settings rejected; wider fuzz/resource security gate still pending |
| P2.2 | Partial | JSONC lexer; bounded sandboxed include API; alpha and source retained; duplicate rule property merging, typography resets; border/gutter slots corrected. Limited representative dotted/ancestor resolution and per-property precedence implemented; unsupported source keys/selectors named in loss records. Standard semantic selectors/type distinctions, source-order/property precedence and single-modifier Neovim lowering implemented; complete pinned Dark+ plus include/license and explicit stack probes added. Multi-reference held-out corpus, custom registries and real VS Code/LSP priority conformance remain pending |
| P2.3 | Initial implementation | Bidirectional fixed-hue lightness search, quantized crossing refinement, actual best achieved, multiple backdrop/alpha solver; final legacy requirement recheck; transactional v1 requirements, dynamic role/overlay backdrops and bounded semantic-combination floor checks. Rollback is explicit in reports. No global optimality claim |
| P2.4 | Initial implementation | Calculated scene matrix, inheritance, alpha, foreground replacement, explicit unverified states. Semantic scene contexts and explicit terminal default/background mapping added; real target priority validation and terminal background-combination matrix remain pending |
| P2.5 | Partial | Extracted curl transport sends config/credential/payload over stdin only, no disk/argv credentials; redirects disabled, timeout/exit/status/size/refusal/truncation checks, redacted errors, real TCP mock test. Interrupt/cancellation ownership and structured-output fallback profiles remain pending |
| P2.6 | Executable evaluation foundation | Property-aware interpretation v1 with strict boundaries, explicit clarification/unsupported decisions, trusted opaque user anchors, preflight feasibility and iteration context. Original 12-case suite retained; 74-case matrix now covers all 50 request IDs, 15 state/context variants, eight sequential turns and three paired checks. Thirteen known-bad mutations verify evaluator sensitivity. Reports distinguish proposed operations, preservation/readability/strength, missing checks and edit/clarification/refusal buckets. Checked-in runs are canned replay; inspected held_out labels are not independent unseen evidence. Live model comparison and independent labels remain pending |
| P3.1–P3.6 | Pending | Identity model, richer design relations, image handling, synthesis, candidates, light/dark families |
| P4.1–P4.5 | Pending | Existing thin-client loop retained; explicit saved-version application guards fixed. Opt-in HexwitchV1 uses versioned responses, clarification display and rollback blocking, with real-binary headless replay tests. Capture/comparison/persistence/cancellation and new-editor acceptance still pending |
| P5.1–P5.3 | Pending | CI token scope/timeouts improved; release matrix, fuzz/security release gate and human evaluation not delivered |

## Verification

- `cargo test --manifest-path core/Cargo.toml --all --locked`: 143 Rust tests pass; 7 Python metadata/corpus/rebuild tests pass.
- `cargo fmt --manifest-path core/Cargo.toml --all --check`: pass.
- `cargo clippy --manifest-path core/Cargo.toml --all-targets --locked -- -D warnings`: pass.
- Real local Neovim round-trips, thin client lifecycle regression and TCP mock transport included.
- `python3 core/eval/verify-fixtures.py`: exact hashes and metadata verified for 5 import artifacts and 3 evaluation states.
- No live VS Code conformance, live provider evaluation or human preference study performed.

## Gates

No phase 1/2/3/4/5 gate is declared complete. Unit tests prove concrete behavior,
not aesthetic quality or live editor/provider conformance. Keep provider tests
mocked and human/live-provider evidence opt-in.
