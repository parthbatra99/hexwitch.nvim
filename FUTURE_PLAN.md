# Hexwitch future plan

## Purpose and priority

Build an editor-independent engine that translates something a person loves—an existing theme, an image, or a description—into the theme they wish existed. Preserve the visual choices they care about while making deliberate changes, and compile the result into different editors with explicit limitations.

The remaining work is organized into six phases and six parallel workstreams, not a single queue of milestones. The engine remains the priority; adapters should exercise its contracts rather than hide gaps.

**M0–M3 are working prototype slices, not production-complete capabilities.** We have proven that the loop runs. We have not yet proven faithful theme import, reliable interpretation, comprehensive readability validation, or distinctive generation.

Priority order:

1. Make faithful adaptation dependable.
2. Build creative reference translation and synthesis.
3. Use multiple editors to pressure-test portability early.
4. Package and evaluate the product without overstating its capabilities.

The highest-value next work is **IR v1, import fidelity, scene validation, and interpretation-quality evaluations**. These determine whether Hexwitch becomes a personal visual-design engine or merely a polished interface around a recoloring prototype.

## 1. Current state

| Area | Working now | Still missing |
|---|---|---|
| Color science | sRGB, OKLab/OKLCH, contrast, basic gamut mapping | Wider edge-case coverage, precision policy, stronger independent reference tests |
| Theme representation | Roles, styles, basic provenance | Opacity, explicit inheritance/clearing, semantic modifiers, property-level protection, actual relationship graph |
| Import | Selected VS Code JSON fields and scopes | Faithful rule resolution, JSONC/includes, semantic rules, loss reporting, complete references |
| Revision | Seven operations, protection, reports | Precise solving, persistent constraints, explicit property targets, relational edits |
| Interpretation | Prompt → actions → edit spec | Strict client validation, actual-theme validation, clarification, model-quality evaluation |
| Validation | Base-background contrast and some real Neovim round-trips | Selected/diff/search/popup scenes, missing-state handling, rendered-code coverage |
| Neovim client | Reference, refinement, review, apply, undo/redo | Installed-theme capture, visual comparison, persistence, cancellation, lifecycle integration |
| Generation | Modification of an existing theme | New visual identities from descriptions, images, and mixed references |
| Other editors | Nothing emitted yet | VS Code and Zed adapters |
| Distribution | Local builds | Binaries, installation, compatibility, releases |

Existing tests establish functionality, not aesthetic quality. The six theme fixtures are useful regression samples, but they are curated excerpts—not faithful, pinned copies of the complete themes.

This is a planning baseline, not a live completion tracker. Update the status and acceptance evidence as packages land.

## 2. Parallel workstreams

Give each stream one primary owner.

| Stream | Responsibility | Main file ownership |
|---|---|---|
| A — Core contracts | IR, edit specifications, protection, shared reports | `core/crates/hexwitch-core/src/ir/`, new contract modules |
| B — Color and revision | Color math, constraint solving, generation | `core/crates/hexwitch-core/src/color/`, revision/solver modules |
| C — Evaluation and validation | Corpora, scenes, benchmarks, independent checks | `core/eval/`, validation modules, evaluation tests |
| D — Interpretation and references | Language interpretation, provider transport, images | Interpretation/provider/reference modules |
| E — Editor fidelity | Import resolution, Neovim/VS Code/Zed mappings | Import and emit modules |
| F — Product and delivery | Neovim experience, session storage, packaging | `adapters/`, release workflows |

Do not have all six streams editing `main.rs`, `lib.rs`, and `Cargo.toml`. One integration owner should wire new modules and dependencies into the workspace.

The identifiers below are proposed work-package identifiers, not existing tracker issues.

## Phase 0 — Establish a safe baseline

**Goal:** parallel work starts from one reproducible checkpoint.

### P0.1 — Checkpoint the current adapter work

- **Owner:** integration
- **Dependencies:** none

Work:

- Review and checkpoint the currently uncommitted adapter changes.
- Verify the legacy move and updated CI paths.
- Record the current test commands and supported local environment.
- Preserve user work; no broad cleanup or legacy deletion.

**Acceptance:** every workstream can branch from the same known revision.

### P0.2 — Reclassify claims and limitations

- **Owner:** C/F
- **Dependencies:** none

Work:

- Distinguish implemented behavior from intended behavior in docs.
- Replace complete-theme-import implications with the actual supported subset.
- Distinguish our gamut-mapping strategy from claiming full CSS-spec implementation.
- Stop calling a coarse contrast search globally minimal.
- State which checks use real editor resolution versus calculated colors.

**Acceptance:** documentation accurately describes the prototype without requiring someone to read its source.

### Phase 0 gate

A clean checkpoint, green tests, documented limitations, and no ambiguous ownership.

## Phase 1 — Freeze the contracts that let everyone work independently

**Goal:** agree on interfaces before building more features around them.

Keep this phase small. We need a usable v1 contract, not an enormous speculative ontology.

### P1.1 — Theme IR v1

- **Owner:** A
- **Dependencies:** P0.1

Define:

- Color values and opacity.
- Foreground, background, decoration, and typography as separate properties.
- Explicit unspecified / inherit / clear / concrete value semantics.
- Semantic modifiers without multiplying every role into dozens of variants.
- Source information separately from user protection.
- An extension/passthrough mechanism for editor-specific information.
- Version validation and explicit migration policy.

The current `Provenance` enum conflates origin and protection. A generated color can also become protected. Those need independent representation.

**Acceptance:** imports, revision, and adapters can express a translucent selection, an explicit typography reset, and a locked foreground without losing information.

### P1.2 — Intent and revision contract

- **Owner:** A/B
- **Dependencies:** P1.1’s agreed shape

Define:

- Exact role and property targets.
- Role collections and validated selectors.
- Hard locks versus soft identity preferences.
- Request-local versus persistent protection.
- Explicit unlock operations.
- Required constraints versus aesthetic objectives.
- Outcomes: `satisfied`, `unchanged`, `refused`, `infeasible`, `needs_clarification`.

Example:

> Keep the background color, but remove its transparency.

That should not require unlocking the entire style.

**Acceptance:** two sequential requests can deliberately preserve, modify, and unlock individual properties.

### P1.3 — Versioned CLI/report protocol

- **Owner:** A/F
- **Dependencies:** agreed P1.1/P1.2 shapes

Work:

- Machine-readable results for import, interpret, revise, validate, and emit.
- Stable error codes and partial-success semantics.
- Request IDs and protocol version.
- Atomic artifact writes.
- No dependence on parsing human-readable stderr.
- Lightweight capability/version discovery for clients.

**Acceptance:** a client can determine what happened without interpreting prose.

### P1.4 — Evaluation specification and corpus structure

- **Owner:** C
- **Dependencies:** none; coordinate naming with A

Define separate suites:

1. Color correctness.
2. Import fidelity.
3. Revision correctness.
4. Interpretation quality.
5. Scene readability.
6. Adapter fidelity.
7. Human preference and intent preservation.

Each fixture needs origin, license, revision/hash, purpose, expected outcomes, and known limitations.

**Acceptance:** a contributor can add a meaningful test without inventing a new format.

### Phase 1 gate

Contracts are agreed, with example documents and compatibility tests. Core consumers can then proceed in parallel.

## Phase 2 — Make adaptation trustworthy

**Goal:** changing something a user loves works reliably before creative generation expands.

### P2.1 — Strict validation at every input boundary

- **Owner:** A
- **Dependencies:** P1.1–P1.3

Concrete gaps to close:

- `LlmSpec` currently permits unknown fields and defaults missing actions to empty.
- Wildcard validation currently accepts any string ending in `.*`.
- A vocabulary-valid role is not necessarily present in the actual theme.
- Direct edit specs need numerical validation too—not just model-translated specs.
- Reject nonfinite values and enforce property ranges.
- Validate terminal indices, opacity, blend, and IR versions.
- Give consistent errors for nonexistent protection targets.

**Acceptance:** malformed input cannot become a silent no-op, a partially understood request, or a misleading success.

### P2.2 — Faithful import with explicit losses

- **Owner:** E
- **Dependencies:** P1.1

Work:

- JSONC, supported includes, and cycle handling.
- Correct scope parsing and representative scope-stack resolution.
- Rule precedence and per-property inheritance.
- Empty `fontStyle` resets.
- Semantic-token rules and modifiers.
- Preserve opacity.
- Expand workbench, terminal, search, diff, and diagnostic coverage.
- Preserve unsupported source rules or report their loss.
- Resolve current property-slot mismatches, including borders and gutter markers.

**Acceptance:** importing a supported real theme preserves its measured treatments; unsupported behavior is named rather than discarded.

Do not claim a broad TextMate implementation just because dotted prefixes work.

### P2.3 — Correct contrast and constraint solver

- **Owner:** B
- **Dependencies:** P1.2

Work:

- Replace the one-direction coarse search.
- Search permitted directions; distinguish a policy restriction from true infeasibility.
- Refine crossing points and validate rounded output.
- Report the actual best achievable result, not zero on failure.
- Solve the intended property—not both foreground and background by default.
- Handle several applicable backgrounds.
- Recheck requirements after all operations finish.
- Distinguish solver tolerance from an optimality claim.

**Acceptance:** the solver cannot claim impossible merely because it searched the wrong direction, and later edits cannot silently invalidate earlier requirements.

### P2.4 — Resolved-scene validation

- **Owner:** C
- **Dependencies:** P1.1; integrate with E’s target profiles

Initial scene set:

- Normal code and comments.
- Active and inactive selection.
- Current line.
- Search and current search.
- Added/removed/modified lines.
- Completion-menu selection.
- Floating diagnostics.
- Diagnostic decoration over syntax.
- Selected text inside a diff/search region.
- Terminal foreground/background combinations.

Include alpha composition, foreground replacement, inheritance, and editor-specific ordering.

Unknown rendering must produce `unverified`, not `pass`.

**Acceptance:** a candidate receives a context-specific validation report that can block or qualify application.

### P2.5 — Provider transport hardening

- **Owner:** D
- **Dependencies:** P1.3

Extract transport from the growing CLI file.

Work:

- Secure temporary resources or an HTTP client with equivalent guarantees.
- Credentials protected at creation—not written first and chmodded later.
- Cleanup on error and interruption.
- Redacted errors and logs.
- UTF-8-safe truncation.
- Network/process exit checking.
- Timeouts, cancellation, rate limits, provider refusals, and truncated responses.
- Mock-server tests.
- Explicit structured-output capability handling.

**Acceptance:** the complete network path is testable, failure-safe, and does not expose credentials.

### P2.6 — Interpretation-quality evaluation

- **Owner:** C/D
- **Dependencies:** P1.4; baseline can start immediately

Start with roughly 50 annotated requests, covering:

- Direct changes.
- Protection.
- Multiple changes.
- Ambiguous references, such as “that blue.”
- Strength: “slightly,” “much,” “less than before.”
- Contradictions.
- Unsupported requests.
- Repeated iteration.
- Requests to unlock earlier choices.
- Readable but still secondary information.
- Typography requests.

Judge semantic outcomes, not exact JSON or exact hex values.

Example request:

> Keep the background and make comments easier to read.

- **Required:** unchanged background; improved applicable comment readability.
- **Forbidden:** unrelated syntax changes.
- **Acceptable:** a no-op if already satisfied, with an explanation.

**Acceptance:** a reproducible model/prompt comparison, held-out cases, and a categorized failure report—not just schema-pass percentages.

### Phase 2 gate — Trustworthy adaptation

A held-out set of real references can be imported, revised, validated, and emitted while preserving protected treatments. Failures are visible and reproducible.

## Phase 3 — Build the actual personalization engine

**Goal:** escape the seven-verb recoloring tool without giving up deterministic computation.

### P3.1 — Visual identity model

- **Owner:** A/B
- **Dependencies:** P2.2/P2.6

Represent useful relationships:

- Neutral temperature and tint.
- Surface hierarchy.
- Accent families and associations.
- Chroma distribution.
- Relative prominence.
- Compact versus broad visual vocabulary.
- Strong identity anchors versus incidental choices.

This is not one identity score. Some dimensions should be preserved; others are precisely what the user wants changed.

**Acceptance:** the system can explain what it preserved and what it deliberately reinterpreted.

### P3.2 — Richer design operations

- **Owner:** B/D
- **Dependencies:** P3.1, P2.3/P2.4

Support:

- Typography changes, such as “no italics.”
- Relative prominence, such as “quieter keywords.”
- Neutral temperature without rotating every syntax hue.
- Accent replacement from a trusted user/reference anchor.
- Surface-depth adjustments.
- Theme completion for missing roles.
- Coordinated changes that respect existing constraints.

Boundary: the model may interpret references and propose relationships. The engine computes and validates final colors. User-provided colors must be legitimate inputs.

**Acceptance:** common design requests no longer collapse into arbitrary global hue shifts.

### P3.3 — Image/reference interpretation

- **Owner:** D
- **Dependencies:** P3.1; image corpus can start immediately

Work:

- Extract representative color relationships, not only dominant colors.
- Distinguish background character from focal accents.
- Preserve rare accent distribution.
- Interpret luminance/chroma structure and palette families.
- Support “inspired by” versus “harmonizes with.”
- Let users select or protect a region/color.
- Make inferred choices visible and editable.
- Handle ICC profiles, image orientation, and input size limits.
- Obtain explicit consent before transmitting a personal image to a provider.

Initial corpus: around 12 deliberately varied images, including monochrome, pastel, high-chroma, sparse warm accents, and multiple competing palettes.

**Acceptance:** results differ meaningfully between references and users can correct the interpretation.

### P3.4 — Description-to-theme synthesis

- **Owner:** B/D
- **Dependencies:** P3.1/P3.2

The engine needs a starting point other than a shipped theme.

Work:

- Compile a design brief into a complete role system.
- Generate surfaces, readable foregrounds, interactions, and diagnostics together.
- Apply identity anchors without inheriting one favorite aesthetic.
- Validate complete output before emission.

**Acceptance:** “warm paper, restrained syntax, crisp diagnostics” generates a coherent new theme—not a renamed stock palette.

### P3.5 — Candidate generation and comparison

- **Owner:** B/C/F
- **Dependencies:** P3.1–P3.4

Work:

- Produce a small set of genuinely different interpretations.
- Explain tradeoffs and deliberate differences.
- Preserve seed/config/version for reproducibility.
- Reject invalid candidates before ranking.
- Avoid selecting solely by one aggregate score.

**Acceptance:** candidates differ along meaningful user choices—not random minor color perturbations.

### P3.6 — Light/dark family generation

- **Owner:** B
- **Dependencies:** P3.1/P2.4

Work:

- Shared semantic associations and identity anchors.
- Independently solved surfaces/foregrounds.
- No mechanical inversion.
- Test each mode in its own scene matrix.

**Acceptance:** both modes remain recognizable, while each stands on its own as a usable design.

### Phase 3 gate — Creative personalization

Demonstrate three routes into the same engine:

1. Existing theme + feedback.
2. Image + interpretation.
3. Description + synthesis.

Each must be steerable through another round of feedback.

## Phase 4 — Editor fidelity and a usable product

These packages can overlap Phase 3 once IR v1 and validation contracts are stable.

### P4.1 — Capture an installed Neovim theme

- **Owner:** E/F
- **Dependencies:** P1.1/P2.2

Work:

- Capture resolved groups, links, relevant attributes, and terminal palette.
- Map to portable roles while retaining editor-specific information.
- Mark context/provider gaps explicitly.
- Treat the capture as an immutable reference document.

**Acceptance:** modifying the theme a person is already using requires no manual VS Code JSON file.

### P4.2 — Neovim fidelity and lifecycle

- **Owner:** E/F
- **Dependencies:** P2.4

Work:

- Current Tree-sitter capture names.
- LSP modifiers and priority interactions.
- Plugin-specific extension mappings.
- Proper colorscheme loading and `ColorScheme` behavior.
- Terminal lifecycle and truecolor limitations.
- Empty/default group behavior.
- Actual emitted-property loss reports, not only role-consumed reports.

**Acceptance:** the adapter behaves correctly in a normal configured editor, not only a clean headless one.

### P4.3 — VS Code adapter

- **Owner:** E
- **Dependencies:** P1.1/P2.2/P2.4

Work:

- Workbench colors, TextMate rules, semantic rules, terminal colors.
- Modifier support and explicit fallbacks.
- Package/export format.
- Real VS Code loading and targeted conformance checks.
- Same reference/brief compiled for Neovim and VS Code.

**Acceptance:** portability is demonstrated, with differences reported honestly.

### P4.4 — Zed adapter

- **Owner:** E; separate owner if available
- **Dependencies:** P1.1/P2.4

Work:

- Versioned schema.
- Syntax and semantic mappings.
- Explicit settings requirements.
- Modifier and rule-precedence tests.
- Capability/degradation reports.

**Acceptance:** emitted themes load and documented semantics match the tested editor version.

### P4.5 — Visual review and session experience

- **Owner:** F
- **Dependencies:** P1.3/P2.4; comparison UI can start with fixtures

Work:

- Before/after preview on the same code.
- Highlight which roles changed.
- Candidate switching.
- Accept/reject and request clarification.
- Progress and cancellation.
- Persistent sessions and recovery.
- Branch names/bookmarks.
- Property-level lock/unlock controls.
- Save/export and stable installed colorscheme names.

**Acceptance:** a person can converge on a theme without handling temporary files or remembering version IDs.

### Phase 4 gate — Usable alpha

A new user can start from a familiar reference, understand the proposed changes, compare them, iterate, and retain/export the result.

## Phase 5 — Distribution and evidence-driven improvement

### P5.1 — Packaging and release pipeline

- **Owner:** F
- **Dependencies:** protocol compatibility established

Work:

- macOS Apple Silicon/Intel and Linux x86_64/ARM builds.
- Windows if included in the supported product scope.
- Versioned artifacts and checksums.
- Engine/client compatibility checks.
- Install/update/uninstall instructions.
- Monorepo-friendly plugin distribution.
- No automatic credential discovery or surprising downloads.

**Acceptance:** supported users do not need a Rust development environment.

### P5.2 — Reliability and security gate

- **Owner:** A/C/F
- **Dependencies:** release candidate

Work:

- Fuzz malformed themes/specs/images.
- Resource and payload limits.
- Safe includes and file handling.
- Atomic writes and interruption recovery.
- Offline operation.
- Dependency/license inventory.
- Pinned CI actions and least-privilege permissions.
- Supported OS/editor matrix.

**Acceptance:** known failure classes have explicit tests and handling.

### P5.3 — Human evaluation

- **Owner:** C/product
- **Dependencies:** usable alpha

Measure separately:

- Did it preserve what the person loved?
- Did it understand the requested change?
- How many rounds before acceptance?
- Were unrelated changes surprising?
- Did the person keep the theme?
- Task readability and sustained comfort.

Use controlled comparisons for specific readability claims. Do not turn preference data into productivity claims.

**Acceptance:** we can identify where users struggle and improve against a stable evaluation set.

## 3. First wave: work to start in parallel immediately

| Parallel package | Start now? | Guardrail |
|---|---|---|
| IR/EditSpec/report contracts — P1.1–P1.3 | Yes | One contract/integration owner |
| Eval structure and annotated prompts — P1.4/P2.6 | Yes | Do not tune goldens to current output |
| Provider transport hardening — P2.5 | Yes | Preserve current CLI behavior until protocol integration |
| Import-fidelity corpus and investigation — P2.2 | Yes | Implementation follows the agreed IR |
| Solver regression cases — P2.3 | Yes | Tests first; coordinate property-target semantics |
| Visual comparison UX prototype — P4.5 | Yes | Use fixed fixtures; do not redefine engine contracts |

Image research/corpus work can also start immediately. Image-to-theme implementation should wait for the brief/identity model.

Do not start three adapter rewrites against the current IR. That would multiply the cost of correcting it.

## 4. Rules for parallel implementation

1. Checkpoint the current uncommitted work before creating worktrees.
2. Give every package a separate branch/worktree and explicit allowed paths.
3. Treat shared contracts as versioned APIs, even inside the monorepo.
4. Route changes to `main.rs`, `lib.rs`, manifests, and shared schemas through the integration owner.
5. Each package must deliver:
   - implementation;
   - focused tests;
   - at least one failure-case fixture;
   - limitations;
   - a compatibility note.
6. Merge contracts first, then consumers. Do not repeatedly cherry-pick unstable schema changes.
7. Keep offline regression tests separate from opt-in live-provider evaluations.

### Work-package handoff template

```text
Package:
Objective:
Dependencies:
Allowed paths:
Contract version:
Required positive tests:
Required failure tests:
Explicitly out of scope:
Acceptance gate:
```
