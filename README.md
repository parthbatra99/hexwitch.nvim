# hexwitch.nvim

<p align="center">
  <img src="./hexwitch-logo.png" alt="Hexwitch Logo" width="200" />
</p>

<p align="center">
  AI-powered colorscheme generator for Neovim.
</p>

Describe a vibe in plain English → an AI returns a 16-color palette → hexwitch
applies it as a live colorscheme (highlights, Tree-sitter, LSP diagnostics,
terminal colors). Talk to the AI for both generation **and** iteration:

```vim
:Hexwitch a calm forest at full moon
:Hexwitch refine make it a touch warmer
```

It works with **any OpenAI-compatible endpoint** — OpenAI, OpenRouter, Ollama,
LM Studio, and more — via a single `base_url` setting.

## Requirements

- Neovim ≥ 0.9.0
- `curl`
- An API key for an OpenAI-compatible provider (OpenAI, OpenRouter, …) — or a
  local model (Ollama/LM Studio) that needs no key
- [`plenary.nvim`](https://github.com/nvim-lua/plenary.nvim)
- [`telescope.nvim`](https://github.com/nvim-telescope/telescope.nvim)

## Installation

With [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{
  "parthbatra99/hexwitch.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim",
  },
  config = function()
    require("hexwitch").setup({
      base_url = "https://api.openai.com/v1",
      api_key  = vim.env.OPENAI_API_KEY, -- or hardcode / op.nvim
      model    = "gpt-4o-mini",
    })
  end,
}
```

## Configuration

All fields optional; shown with their defaults:

```lua
require("hexwitch").setup({
  base_url = "https://api.openai.com/v1",  -- any OpenAI-compatible endpoint
  api_key  = nil,                          -- falls back to $OPENAI_API_KEY / $HEXWITCH_API_KEY
  model    = "gpt-4o-mini",
  -- max_tokens = 1024,                    -- optional
  save_themes = true,
  themes_dir  = vim.fn.stdpath("data") .. "/hexwitch",
  debug = false,
})
```

Generation settings are locked to opinionated defaults (not configurable):
`temperature = 1` (reasoning-model friendly), a 30 s timeout, and JSON-schema
structured output always on — so the endpoint must support structured outputs.

### Pointing at other providers

```lua
-- OpenRouter
require("hexwitch").setup({
  base_url = "https://openrouter.ai/api/v1",
  api_key  = vim.env.OPENROUTER_API_KEY,
  model    = "anthropic/claude-3.5-sonnet",
})

-- Local Ollama (no key; needs a model that supports JSON-schema structured output)
require("hexwitch").setup({
  base_url = "http://localhost:11434/v1",
  api_key  = "ollama",
  model    = "llama3",
})
```

## Commands

| Command | Description |
|---|---|
| `:Hexwitch [description]` | Generate + apply a theme from a description |
| `:Hexwitch refine [change]` | Re-prompt the AI to iterate on the current theme |
| `:Hexwitch random` | Generate a random theme |
| `:Hexwitch save [name]` / `:Hexwitch load [name]` | Save / load themes |
| `:Hexwitch browse` | Browse saved themes (Telescope) |
| `:Hexwitch history` | View generation history |
| `:Hexwitch export [name]` / `:Hexwitch import` | Clipboard JSON export/import |
| `:Hexwitch status` / `:Hexwitch help` | Diagnostics |

Standalone commands (`:HexwitchRefine`, `:HexwitchRandom`, `:HexwitchBrowse`, …)
and `<Plug>(Hexwitch*)` mappings are also provided.

## Test drive

Try hexwitch in an isolated Neovim without touching your real config — the
example at `examples/nvim-test/init.lua` bootstraps lazy.nvim + dependencies and
loads hexwitch from your local checkout:

```bash
# from the repo root, using your local checkout:
HEXWITCH_PLUGIN_DIR="$PWD" \
OPENROUTER_API_KEY="sk-or-..." \
NVIM_APPNAME=hexwitch-test nvim
```

Then `:Hexwitch a forest at full moon` (or `<leader>hw`). See the file for the
full set of test keymaps.

## Debugging

```vim
:checkhealth hexwitch                       " health + endpoint/key check
:lua require("hexwitch").setup({ debug = true })
:Hexwitch logs                              " view request/response logs
```

Saved themes live at `themes_dir` (default `~/.local/share/nvim/hexwitch/`).

## License

MIT
