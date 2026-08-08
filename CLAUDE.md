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
├── minimal_init.lua           plenary + telescope on rtp
├── fixtures/                  test_helpers, mock_responses, mock_themes
└── hexwitch/
    ├── ai_spec.lua            client: payload, schema, error paths (mock plenary.curl)
    ├── config_spec.lua        defaults, merge, base_url validation
    ├── error_handling_spec.lua network/API/theme/storage/dependency edge cases
    ├── security_spec.lua      path traversal, theme validation, clipboard import
    ├── theme_spec.lua         apply + storage (save/load/list/delete round-trip)
    ├── ui_spec.lua            telescope UI modules load + behave
    └── workflow_spec.lua      end-to-end generate → save → load → delete
```

### Current status: 98 passing / 0 failing

| Spec          | Tests | Covers |
|---------------|-------|--------|
| config        | 4     | defaults, incremental merge, base_url validation |
| ai            | 11    | request payload, schema name, key/plenary/HTTP/parse errors |
| theme         | 24    | apply, terminal colors, storage round-trip with the *real* palette |
| ui            | 11    | telescope input/browser/status modules |
| error_handling| 23    | network, malformed responses, storage, dependencies |
| workflow      | 12    | full generate→save→load→delete, config updates |
| security      | 13    | sanitization, path traversal, theme/clipboard validation |

### Test conventions

- **Mock `plenary.curl`** for any AI test (`package.loaded["plenary.curl"] = {post = ...}`).
  The client honors `vim.g.hexwitch_test_sync_mode = true` to run the response
  handler inline so busted can assert synchronously, and
  `vim.g.hexwitch_test_plenary_unavailable = true` to simulate a missing dependency.
- **`hexwitch.generate`/`refine` accept an optional callback** `fun(data, err)`
  — use it to assert results without the success UI.
- **Storage tests set `themes_dir`** explicitly (storage reads `cfg.themes_dir`,
  not `stdpath`). Note that `state.json` shares `themes_dir`; `storage.list`
  excludes it so it never appears as a theme.
- `$OPENAI_API_KEY` in your shell will be picked up as the default key — tests
  that need the "no key" path pass `api_key = ""` explicitly.

## CI

GitHub Actions runs the plenary suite across Neovim 0.9.5 / 0.10 / nightly on
Ubuntu and macOS, plus luacheck. All tests must pass on `main`.
