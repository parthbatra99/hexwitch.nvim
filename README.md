# hexwitch.nvim

AI-powered colorscheme generator for Neovim.

<img src="./hexwitch-logo.png" alt="Hexwitch Logo" width="200" />

hexwitch.nvim leverages OpenAI or OpenRouter models to generate custom colorschemes for Neovim from natural language prompts. Describe your desired theme, and Hexwitch will brew and apply a cohesive palette with one command.

## Features

- AI-Powered Generation: Create custom colorschemes using natural language descriptions
- Flexible Configuration: Choose AI provider, model, creativity level, and more
- Persistent Storage: Save and load generated themes locally
- Telescope Integration: Browse history and saved themes, quick actions
- Comprehensive Coverage: Syntax highlighting, LSP diagnostics, and terminal colors
- Health Checks: `:checkhealth hexwitch` validates your setup

## Requirements

- Neovim ≥ 0.9.0
- `curl` (for HTTP requests)
- An API key for your chosen provider:
  - OpenAI: set `OPENAI_API_KEY`
  - OpenRouter: set `OPENROUTER_API_KEY`
- [`plenary.nvim`](https://github.com/nvim-lua/plenary.nvim)
- [`telescope.nvim`](https://github.com/nvim-telescope/telescope.nvim)

## Installation

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  "hexwitch/hexwitch.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim",
  },
  config = function()
    require("hexwitch").setup()
  end,
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  "hexwitch/hexwitch.nvim",
  requires = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim",
  },
  config = function()
    require("hexwitch").setup()
  end,
}
```

### Using [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'hexwitch/hexwitch.nvim'
Plug 'nvim-lua/plenary.nvim'
Plug 'nvim-telescope/telescope.nvim'

lua << EOF
require("hexwitch").setup()
EOF
```

## Configuration

hexwitch.nvim comes with sensible defaults; pass only the options you need:

```lua
require("hexwitch").setup({
  -- Provider & auth
  ai_provider = "openai",              -- "openai", "openrouter", or "custom"
  api_key = nil,                       -- defaults to env (OPENAI_API_KEY/OPENROUTER_API_KEY)
  fallback_provider = "openrouter",    -- optional fallback

  -- Model behavior
  model = "gpt-4o-mini",
  temperature = 0.7,                    -- 0.0–2.0
  timeout = 30000,                      -- ms

  -- UI & storage
  ui_mode = "input",                   -- "input" or "telescope"
  save_themes = true,                   -- persist generated themes
  themes_dir = vim.fn.stdpath("data") .. "/hexwitch",
  max_history = 50,
  auto_save_history = true,
  contrast_threshold = 4.5,

  debug = false,
})
```

### Configuration Options

| Option             | Type      | Default                             | Description                                     |
|--------------------|-----------|-------------------------------------|-------------------------------------------------|
| `ai_provider`      | `string`  | `"openai"`                          | Primary AI provider (`openai`, `openrouter`, `custom`) |
| `api_key`          | `string`  | Provider env var                    | Uses `OPENAI_API_KEY` or `OPENROUTER_API_KEY`  |
| `fallback_provider`| `string`  | `"openrouter"`                      | Secondary provider if primary creation fails   |
| `model`            | `string`  | `"gpt-4o-mini"`                     | Model identifier for the active provider       |
| `temperature`      | `number`  | `0.7`                                | Creativity (0.0–2.0; higher = wilder palettes) |
| `timeout`          | `number`  | `30000`                              | Request timeout in milliseconds                |
| `ui_mode`          | `string`  | `"input"`                           | Prompt style (`input` or `telescope`)          |
| `save_themes`      | `boolean` | `true`                               | Automatically write generated themes to disk   |
| `themes_dir`       | `string`  | `vim.fn.stdpath("data") .. "/hexwitch"` | Directory used for stored themes         |
| `max_history`      | `number`  | `50`                                 | Maximum stored generation entries              |
| `auto_save_history`| `boolean` | `true`                               | Persist generation history between sessions    |
| `contrast_threshold`| `number` | `4.5`                                | Minimum contrast when evaluating themes        |
| `debug`            | `boolean` | `false`                              | Emit verbose logging to `:messages`            |

Note: Hexwitch also respects `HEXWITCH_API_KEY` as a generic fallback if a provider-specific env var is not set.

## Usage

### Basic Usage

The simplest way to use Hexwitch is with the `:Hexwitch` command:

```vim
:Hexwitch a dark theme with purple accents and warm colors
```

### Local Test Profile

Use the provided example to try Hexwitch in an isolated Neovim profile without touching your main config:

1) Run Neovim with a temporary app name

```
NVIM_APPNAME=hexwitch-test nvim -u examples/nvim-test/init.lua
```

2) (Optional) Test against a local checkout without installing

```
HEXWITCH_PLUGIN_DIR=/absolute/path/to/hexwitch.nvim \
NVIM_APPNAME=hexwitch-test nvim -u examples/nvim-test/init.lua
```

Notes
- Set your provider key via env (`OPENAI_API_KEY` or `OPENROUTER_API_KEY`).
- The example enables the Telescope UI and adds handy test mappings:
  - `<leader>hw` prompt, `<leader>hs` status, `<leader>hl` logs
  - `<leader>hb` browse themes, `<leader>hh` history, `<leader>hq` quick actions
  - `<leader>hp` presets, `<leader>hu` undo, `<leader>hr` redo
 

### Commands

- `:Hexwitch [description]` — generate and apply a theme from text. With no args, opens the prompt UI.
- `:Hexwitch quick` — generate a quick variation of the last theme.
- `:Hexwitch random` — surprise theme with creative colors.
- `:Hexwitch refine [changes]` — refine the current theme; with no args, opens refinement UI.
- `:Hexwitch save <name>` / `:Hexwitch load <name>` / `:Hexwitch delete <name>` — manage saved themes.
- `:Hexwitch list` or `:Hexwitch browse` — browse saved themes.
- `:Hexwitch history` / `:Hexwitch clear-history` — view or clear generation history.
- `:Hexwitch undo` / `:Hexwitch redo` — theme undo/redo.
- `:Hexwitch export <name>` / `:Hexwitch import` — copy theme JSON to/from clipboard.
- `:Hexwitch status` / `:Hexwitch providers` — status and provider info.
- `:Hexwitch test [provider]` — test AI connectivity for current or specific provider.

### Programmatic API

```lua
local hexwitch = require("hexwitch")

-- Generate and apply a theme
hexwitch.prompt()

-- Generate a theme with specific description
hexwitch.generate("a calming blue theme with high contrast")

-- Save the current theme
hexwitch.save("my-custom-theme")

-- Load a saved theme
hexwitch.load("my-custom-theme")

-- List all saved themes
local themes = hexwitch.list_themes()

-- Open quick actions menu (Telescope)
require("hexwitch.ui").quick_actions()
```

### Keymaps (optional)

```lua
-- Map leader+h to open the Hexwitch prompt
vim.keymap.set("n", "<leader>h", "<Plug>(HexwitchPrompt)")
```

### Example Theme Descriptions

Here are some example descriptions to get you started:

```vim
"Dark theme with purple accents and good contrast for coding"
"Light theme inspired by autumn colors, easy on the eyes"
"High contrast theme optimized for programming in low light"
"Monochromatic theme with subtle blue highlights"
"Retro terminal theme with green phosphor colors"
"Theme inspired by the ocean at sunset with warm and cool tones"
"Minimalist theme with just enough color for syntax highlighting"
```

### Telescope Integration

Hexwitch uses Telescope for all UI. With `ui_mode = "telescope"`, you get an examples picker and quick actions:

- Generate new theme, browse presets, or pick random
- Browse saved themes and generation history
- Undo/redo, view status, and show recent logs

Open the quick actions menu via:

```lua
require("hexwitch.ui").quick_actions()
```

## Advanced Configuration

### Custom Model Configuration

```lua
require("hexwitch").setup({
  model = "gpt-4-turbo-preview",
  temperature = 0.9, -- More creative themes
  timeout = 60000,   -- Longer timeout for complex themes
})
```

### Theme Storage Management

```lua
-- Custom themes directory
require("hexwitch").setup({
  save_themes = true,
  themes_dir = vim.fn.expand("~/.config/nvim/themes"),
})

-- Manual theme management
local hexwitch = require("hexwitch")

-- Save current theme with custom name
hexwitch.save("my-awesome-theme")

-- Load a specific theme
hexwitch.load("my-awesome-theme")

-- Delete a theme
hexwitch.delete("my-awesome-theme")

-- List all saved themes
local themes = hexwitch.list_themes()
for _, theme in ipairs(themes) do
  print(theme.name, theme.description)
end
```

### Debug Mode

Enable debug mode to troubleshoot issues:

```lua
require("hexwitch").setup({
  debug = true
})
```

Debug information will be logged to `:messages` and can help identify API issues, network problems, or theme generation errors.

## Health Check

Verify your setup with the built-in health check:

```vim
:checkhealth hexwitch
```

This checks:
- API key configuration for current/fallback providers
- Required dependencies (plenary.nvim, telescope.nvim)
- Theme storage directory permissions
- Basic system utilities like `curl`

## Theme Structure

Generated themes include comprehensive color definitions for:

- Editor UI: Background, foreground, cursor, selection
- Syntax Highlighting: All major language constructs
- LSP Diagnostics: Error, warning, info, hint highlights
- TreeSitter: Enhanced syntax parsing highlights
- Git Signs: Added, modified, removed indicators
- Telescope: Customized interface colors
- WhichKey: Key binding hints (if installed)
- Terminal: ANSI color mapping for integrated terminals

## Troubleshooting

### Common Issues

API Key Issues
```vim
" Check if API key is set
:echo $OPENAI_API_KEY

" Set API key temporarily (for testing)
:let $OPENAI_API_KEY = "sk-your-key-here"
```

Network Issues
```vim
" Test network connectivity
:!curl -I https://api.openai.com/v1/models

" Check timeout settings
:lua print(require("hexwitch.config").get().timeout)
```

Theme Storage Issues
```vim
" Check themes directory permissions
:!ls -la ~/.local/share/nvim/hexwitch

" Create directory manually
:!mkdir -p ~/.local/share/nvim/hexwitch
```

### Debug Commands

```vim
" View debug logs
:HexwitchLogs

" Show provider status
:HexwitchProviders

" Test API connectivity (current or specific provider)
:Hexwitch test
:Hexwitch test openrouter
```

## Contributing

Contributions are welcome! Feel free to open issues and PRs with ideas, bug reports, or improvements. If you’re planning a larger change, please open an issue first to discuss the approach.

## License

MIT License.

## Acknowledgments

- [OpenAI](https://openai.com/) for the powerful GPT models
- [plenary.nvim](https://github.com/nvim-lua/plenary.nvim) for utility functions
- [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim) for the fuzzy finder
- The Neovim community for inspiration and feedback

## Additional Resources

- [Neovim Colorscheme Guide](https://neovim.io/doc/user/syntax.html)
- [OpenAI API Documentation](https://platform.openai.com/docs/api-reference)
- [Lua Development Patterns](https://github.com/nvim-lua/lua-dev.nvim)

---

<p align="center">
  Made with ❤️ for the Neovim community
</p>
