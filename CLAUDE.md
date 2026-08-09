# Hexwitch

AI-powered colorscheme generator for Neovim. Describe a vibe in plain English
→ an OpenAI-compatible LLM returns a 16-color palette → the plugin applies it as
a live colorscheme (highlights, Tree-sitter, LSP diagnostics, terminal colors).

## Architecture (post-simplification)

The plugin is deliberately small. One AI client, one mental model: you talk to
the AI for both generation **and** iteration.

```
lua/hexwitch/
├── init.lua                 Public API: setup/generate/refine/save/load/undo/redo
├── commands.lua             :Hexwitch subcommand handlers
├── health.lua               :checkhealth hexwitch
├── undo.lua                 Undo/redo (routes through theme.apply)
├── config/
│   ├── init.lua             Validate + merge config (incremental over current)
│   └── schema.lua           Defaults + typed config (base_url, api_key, …)
├── ai/
│   ├── init.lua             Orchestrate generate + refine; flat get_provider_info
│   ├── client.lua           The single OpenAI-compatible HTTP client (plenary.curl)
│   └── prompts.lua          SYSTEM_PROMPT, build_theme/refinement prompts, SCHEMA
├── theme/
│   ├── init.lua             Thin facade
│   ├── applier.lua          Validate + nvim_set_hl loop + terminal colors; tracks current theme
│   └── storage.lua          Save/load/list/delete themes as JSON (respects themes_dir)
├── storage/state.lua        History, undo/redo stacks, current-theme holder, stats
├── ui/
│   ├── init.lua             UI facade
│   └── telescope/           Prompt, browser, status, notifications (telescope is the only UI)
├── plugin/hexwitch.lua      :Hexwitch command + <Plug> mappings (in plugin/, not lua/)
└── utils/                   color, notify, logger, validate, vim, system
```

### Key design points

- **One client, configurable `base_url`.** `ai/client.lua` POSTs to
  `base_url .. "/chat/completions"`. Default `https://api.openai.com/v1`.
  Point it at OpenRouter (`https://openrouter.ai/api/v1`), Ollama
  (`http://localhost:11434/v1`), LM Studio, etc. Set `structured_output = false`
  for endpoints that don't support JSON-schema structured output — the client
  then falls back to prompt-driven JSON + `extract_json`.
- **Refinement is AI-driven.** `:Hexwitch refine <change>` re-prompts the model
  with the *current theme + the change* (`prompts.build_refinement_prompt`). There
  is no local color-math tweak UI. The AI owns generation and iteration.
- **The applier is the single chokepoint for the "current theme".**
  `theme/applier.lua` calls `state.set_current_theme(data)` at the end of every
  `apply()`, so generate / refine / load / undo / redo / import all keep the live
  theme tracked. `storage.save()` serializes that real palette (it no longer
  re-derives colors from highlight groups).
- **Single source of truth for the schema.** `prompts.SCHEMA` / `SCHEMA_NAME`
  is used by both the request body and (conceptually) the applier's validation.

### Configuration

```lua
require("hexwitch").setup({
  base_url = "https://api.openai.com/v1",  -- any OpenAI-compatible endpoint
  api_key  = nil,                          -- defaults to $OPENAI_API_KEY / $HEXWITCH_API_KEY
  model    = "gpt-4o-mini",
  temperature    = 0.7,
  timeout        = 30000,                  -- ms
  structured_output = true,                -- false for endpoints w/o JSON-schema support
  -- max_tokens = 1024,                    -- optional
  save_themes    = true,
  themes_dir     = vim.fn.stdpath("data") .. "/hexwitch",
  contrast_threshold = 4.5,
  debug = false,
})
```

`ai_provider` / `openai_api_key` (pre-simplification) are gone — `base_url` and
`api_key` replace them. `config.setup()` merges incrementally over the current
config, so repeated calls are additive.

## Commands

Umbrella: `:Hexwitch [description]` generates; subcommands: `refine`, `random`,
`save`, `load`, `browse`, `delete`, `export`, `import`, `history`,
`clear-history`, `undo`, `redo`, `status`, `logs`, `help`. Standalone
convenience commands (`:HexwitchRefine`, `:HexwitchRandom`, `:HexwitchBrowse`,
…`) also exist.

## Testing

Run the suite with [plenary.nvim](https://github.com/nvim-lua/plenary.nvim)
(test deps live in `~/.local/share/nvim/lazy/`; `tests/minimal_init.lua` adds
plenary **and** telescope to the rtp):

```bash
# Full suite
nvim --headless --noplugin -u tests/minimal_init.lua \
  -c "PlenaryBustedDirectory tests/hexwitch/ { minimal_init = 'tests/minimal_init.lua' }"

# Single file
nvim --headless --noplugin -u tests/minimal_init.lua \
  -c "PlenaryBustedFile tests/hexwitch/config_spec.lua" -c "qa"

# Load-graph smoke (no network)
nvim --headless --noplugin -u tests/minimal_init.lua \
  -c "lua require('hexwitch').setup({api_key='x'})" -c "qa"
```

### Test structure

```
tests/
├── minimal_init.lua           plenary + telescope on rtp; tests/ on package.path
├── fixtures/
│   └── helpers.lua            valid/refined themes, mock_curl, completion builders, setup_config, invalid_color_themes
└── hexwitch/
    ├── client_spec.lua        unified client: payload, base_url URL, schema, structured_output toggle, error paths, refine request
    ├── config_spec.lua        defaults, incremental merge, base_url validation, env fallback
    ├── refine_spec.lua        AI refine flow: re-prompts with current theme, history, public callback forwarding
    ├── applier_spec.lua       apply highlights/terminal colors, set_current_theme chokepoint, validation
    ├── storage_spec.lua       real-palette round-trip, themes_dir, list excludes state.json, path traversal
    ├── prompts_spec.lua       SCHEMA single-source, theme/refinement prompt builders, accessibility
    ├── security_spec.lua      clipboard import, theme validation, command-injection guards
    └── ui_spec.lua            telescope modules load, facade surface
```

### Current status: 70 passing / 0 failing

| Spec     | Tests | Covers |
|----------|-------|--------|
| client   | 19    | payload, base_url URL, schema, structured_output toggle, error/parse paths, refine request, is_available |
| config   | 11    | defaults, incremental merge, base_url validation, env fallback |
| refine   | 8     | ai.generate/refine, public generate/refine callback forwarding, get_provider_info |
| storage  | 7     | real-palette round-trip, themes_dir, list excludes state.json, path traversal |
| applier  | 6     | apply highlights + terminal colors, set_current_theme chokepoint, color validation |
| security | 6     | clipboard import, theme validation, system-command guards |
| prompts  | 6     | SCHEMA shape, theme/refinement prompt builders, accessibility, presets |
| ui       | 7     | telescope modules load, facade surface, removed-sliders absent |

### Test conventions

- **Shared helpers** live in `tests/fixtures/helpers.lua`, required as
  `require("fixtures.helpers")` (`tests/` is on `package.path` via `minimal_init`).
  Use `helpers.mock_curl(responder)`, `helpers.completion(theme)`,
  `helpers.setup_config(opts)`, `helpers.valid_theme`.
- **AI tests set `vim.g.hexwitch_test_sync_mode = true`** so the client runs its
  response handler inline (busted can assert synchronously).
  `vim.g.hexwitch_test_plenary_unavailable = true` simulates a missing dependency.
- **Never reload `hexwitch.config` in a test** (`package.loaded[...] = nil`) —
  other modules hold the original instance and you get split-brain. Reset state
  via `config.setup(vim.deepcopy(schema.defaults))` on the shared instance.
- **`config.setup()` is incremental** (merges over current); `api_key` is only
  resolved from `$OPENAI_API_KEY`/`$HEXWITCH_API_KEY` when neither the call nor
  the current config provides one. `helpers.setup_config` passes an explicit key
  so tests don't leak the real env key.
- **The applier validates colors, not `name`.** `storage.load()` returns nothing
  — assert its side effects (`vim.g.colors_name`, highlights). `state.json`
  shares `themes_dir`; `storage.list` excludes it.

## CI

GitHub Actions runs the plenary suite across Neovim 0.9.5 / 0.10 / nightly on
Ubuntu and macOS, plus luacheck. All tests must pass on `main`.
