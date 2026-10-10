# hexwitch

**A theme engine, not a theme.** Hexwitch is building an editor-independent
engine for adapting references into a personal editor theme. Today the
prototype imports a selected subset of VS Code theme JSON, applies
structured revisions, and emits Neovim colorschemes. Image references,
description synthesis, and other editors are planned, not proven capabilities.

The intended design treats a theme as a system of relationships (surfaces,
hierarchy, syntax roles, interaction states). This is the target pipeline;
not every stage or editor below is implemented yet:

```
reference (theme / image / description)
   → interpret   intent + protected choices   ("keep the background, warm the accents")
   → revise      constraint-guided, minimal-diff color work in OKLCH
   → validate    rendered scenes: selections, diffs, diagnostics, overlays
   → emit        Neovim / VS Code / Zed artifacts
```

Three design boundaries:

1. **The engine computes final colors.** Interpretation turns language into
   a structured, reviewable edit spec. Trusted user/reference colors are
   legitimate inputs; color computation remains deterministic.
2. **The IR is the product.** Canonical roles (`syntax.keyword`,
   `diagnostic.error.virtual-text`, …) separate design from editor mappings.
   The current role map is not yet a complete relationship graph.
3. **Emission needs verification.** Current tests resolve selected emitted
   highlights in real Neovim. Base-background contrast calculations are not
   a rendered-code audit or proof of overlay readability.

## Repository layout

| Path | What it is |
|---|---|
| `core/` | Rust workspace: the engine (color science, IR, analysis, revision, validation, emission) |
| `adapters/nvim/` | Thin Neovim client: reference, prompt, candidate review, apply, undo/redo |
| `adapters/nvim-legacy/` | Original v1 Lua plugin, preserved separately for reference |

## Status

**M0–M3 are working prototype slices, not production-complete capabilities.**
See [FUTURE_PLAN.md](FUTURE_PLAN.md) for acceptance gates.

- **M0 — foundations:** sRGB/OKLab/OKLCH, WCAG contrast, compositing,
  and IR v0. Gamut mapping uses deterministic fixed-hue chroma reduction;
  it is not a complete implementation of the CSS gamut-mapping specification.
- **M1 — ingest + analyze:** selected VS Code workbench colors and token
  scopes, contrast ladders, and accent budgets. The six fixtures are curated
  excerpts with independently calculated metrics, not complete pinned theme
  imports. JSONC and sandboxed includes are now supported; alpha and source
  rules are retained. Standard semantic selectors resolve per property;
  class/interface distinctions and single-modifier Neovim groups are emitted.
  Full TextMate resolution, custom semantic registries, and editor priority
  conformance remain incomplete.
- **M2 — revise + emit:** seven operations and whole-role protection,
  reports, Neovim emission, and headless round-trips. The replacement
  contrast solver searches both lightness directions and validates quantized
  crossings without claiming global optimality. Calculated scene validation
  covers overlays but explicitly marks unknown editor ordering unverified.
- **M3 — interpretation + thin Neovim client (initial slice):** natural
  language -> edit specs -> candidate review -> apply/undo/redo. The
  adapter uses the Rust binary and built-in Neovim UI, with offline
  replay tests across the full loop. Live provider behavior and broader
  interpretation accuracy still need evaluation; prebuilt binaries are
  not yet distributed.
- **M4 — portability proof (VS Code adapter).**
- **M5 — image references, candidate sets, lineage.**

## Versioned contracts

See [IR/intent/protocol v1](docs/contracts/ir-v1.md) and the
[implementation evidence ledger](docs/IMPLEMENTATION_STATUS.md). New clients
can use `hexwitch --machine --request-id r1 capabilities`. Existing IR v0 files
require explicit `hexwitch import old.json --migrate-v0 --out migrated.json`.
The legacy CLI/client loop remains available; property-targeted intent v1 is
currently exposed through machine revision.

## Using the CLI

```sh
cd core
cargo run -q -p hexwitch-cli -- contrast '#ebdbb2' '#282828'
# #ebdbb2 on #282828: 10.75:1  (AAA)

cargo run -q -p hexwitch-cli -- convert '#fb4934'
# oklch(0.65969 0.21751 30.39)

cargo run -q -p hexwitch-cli -- map 0.7 0.4 29
# chroma reduction keeps the hue, drops saturation to reach sRGB

cargo run -q -p hexwitch-cli -- eval eval
# 6 reference themes vs independently derived goldens: all pass

cargo run -q -p hexwitch-cli -- analyze eval/themes/tokyo-night.json
# contrast ladder + accent budget for any theme

cargo run -q -p hexwitch-cli -- emit eval/themes/gruvbox-dark-medium.json \
  --out ~/.config/nvim/colors/hexwitch-gruvbox.lua --report
# a real colorscheme + a report of what was skipped or collapsed

cargo run -q -p hexwitch-cli -- revise eval/themes/gruvbox-dark-medium.json \
  --spec eval/editspecs/gruvbox-focus.json \
  --out revised.json --emit-nvim colors/hexwitch-gruvbox-focus.lua
# darker base, readable comments, protected syntax — minimal diff, with a
# per-role report of every color that moved

# with an API key (OPENAI_API_KEY or OPENROUTER_API_KEY):
hexwitch interpret eval/themes/gruvbox-dark-medium.json \
  "darker background and readable comments, keep the syntax colors" \
  --provider openai --out spec.json
# shows the model's notes, writes a validated edit spec; then revise as
# above. --dry-run prints the exact prompt; --from-llm validates a saved
# model response offline.
```

## Neovim

See [the adapter setup and workflow](adapters/nvim/README.md). Configure
`adapters/nvim/` as the plugin directory, build the Rust binary, and set
`cmd` to its path. `:HexwitchReference`, `:Hexwitch`, `:HexwitchStatus`,
`:HexwitchApply`, and `:HexwitchUndo` form the working loop.

The original Lua plugin is preserved in `adapters/nvim-legacy/`; do not
load both adapters, which share the `hexwitch` module/command namespace.

## Testing the whole loop

```sh
cargo test --manifest-path core/Cargo.toml --all --locked
```

With Neovim installed, this includes the new adapter's headless tests
against the real binary: offline interpretation, revision reports,
candidate acceptance, undo/redo, branching, and failure recovery. CI
installs Neovim so these tests cannot silently skip there. This verifies
functionality, not subjective theme quality or live model accuracy.

## Design notes

Foundational decisions came from a review of perception/readability
research (display polarity, syntax-highlighting comprehension studies,
WCAG), the design systems of Solarized/Gruvbox/Nord/Catppuccin/Dracula/
Tokyo Night, and the highlighting mechanisms of the three target editors.
Two honest conclusions shaped the engine:

- There is no scientifically established "best theme" — so the engine
  optimizes for *intent fidelity* and *dependability*, and treats
  readability constraints as explicit, measurable floors rather than
  taste.
- Famous themes disagree on almost everything except structure — so the
  IR models structure (roles, hierarchy, provenance) and leaves the
  aesthetic choices to you.
