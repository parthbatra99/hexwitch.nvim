# hexwitch

**A theme engine, not a theme.** Hexwitch turns something you love — an
existing theme, a wallpaper, a description — into a coherent, readable
editor environment, and keeps it recognizable as *the thing you loved*
while you refine it.

Most theme generators hand you a pile of hex codes. Hexwitch treats a
theme as a system of relationships (surfaces, hierarchy, syntax roles,
interaction states) and compiles that system into concrete editors:

```
reference (theme / image / description)
   → interpret   intent + protected choices   ("keep the background, warm the accents")
   → revise      constraint-guided, minimal-diff color work in OKLCH
   → validate    rendered scenes: selections, diffs, diagnostics, overlays
   → emit        Neovim / VS Code / Zed artifacts
```

Three rules the engine does not break:

1. **The LLM never picks colors.** Interpretation turns language into a
   structured, reviewable edit spec. Color computation is deterministic.
2. **The IR is the product.** One canonical role graph
   (`syntax.keyword`, `diagnostic.error.virtual_text`, …) that all
   adapters lower into editor-native highlighting.
3. **Emit is not the last step.** What gets validated is what the adapter
   actually outputs — after fallbacks, quantization, and alpha compositing.

## Repository layout

| Path | What it is |
|---|---|
| `core/` | Rust workspace: the engine (color science, IR, analysis, revision, validation, emission) |
| `adapters/nvim/` | Neovim frontend (legacy Lua plugin, being replaced by a thin client of the engine) |

## Status

- **M0 — foundations (in progress):** color science (sRGB/OKLab/OKLCH, WCAG
  contrast, gamut mapping, compositing) and IR v0, with tests.
- **M1 — ingest + analyze:** import real themes, compute identity
  fingerprints (contrast ladders, hue/chroma budgets).
- **M2 — revise + emit:** edit specs with protected roles; first Neovim
  colorscheme emission.
- **M3 — interpretation bridge + thin Neovim UI.**
- **M4 — portability proof (VS Code adapter).**
- **M5 — image references, candidate sets, lineage.**

## Using the CLI (M0)

```sh
cd core
cargo run -p hexwitch-cli -- contrast #ebdbb2 #282828
# #ebdbb2 on #282828: 10.75:1  (AAA)

cargo run -p hexwitch-cli -- convert #fb4934
# oklch(0.65250 0.18319 27.95)

cargo run -p hexwitch-cli -- map 0.7 0.4 29
# chroma reduction keeps the hue, drops saturation to reach sRGB
```

## The old Neovim plugin

The original Lua plugin (LLM-prompt → 16 hex colors) lives under
`adapters/nvim/` unchanged for now. It will shrink into a thin frontend —
pick a reference, write a prompt, preview, iterate — as M3 lands.

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
