# hexwitch.nvim

AI-powered colorscheme generator for Neovim.

![Hexwitch Logo](./hexwitch-logo.png)

hexwitch.nvim leverages OpenAI's GPT models to generate beautiful, custom colorschemes for Neovim using natural language descriptions. Describe your desired theme, and hexwitch will create and apply a cohesive color palette tailored to your preferences.

## Features

- AI-Powered Generation: Create custom colorschemes using natural language descriptions
- Extensive Configuration: Customize AI models, temperature, and generation parameters
- Persistent Storage: Save and load your favorite generated themes
- Telescope Integration: Browse and manage themes with an intuitive interface
- Comprehensive Coverage: Supports syntax highlighting, LSP diagnostics, and terminal colors
- Fast Performance: Asynchronous generation with proper error handling
- Health Checks: Built-in diagnostics to ensure proper setup

## Requirements

- Neovim ≥ 0.9.0
- `curl` (for HTTP requests)
- OpenAI API key
- [`plenary.nvim`](https://github.com/nvim-lua/plenary.nvim)
- [`telescope.nvim`](https://github.com/nvim-telescope/telescope.nvim) (optional, for enhanced UI)

## Installation

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  "yourusername/hexwitch.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim", -- optional
  },
  config = function()
    require("hexwitch").setup({
      -- your configuration here
    })
  end,
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  "yourusername/hexwitch.nvim",
  requires = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim", -- optional
  },
  config = function()
    require("hexwitch").setup({
      -- your configuration here
    })
  end
}
```

### Using [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'yourusername/hexwitch.nvim'
Plug 'nvim-lua/plenary.nvim'
Plug 'nvim-telescope/telescope.nvim' " optional

lua << EOF
require("hexwitch").setup({
  -- your configuration here
})
EOF
```

## Configuration

hexwitch.nvim comes with sensible defaults, but you can customize it extensively:

```lua
require("hexwitch").setup({
  -- OpenAI API configuration
  openai_api_key = "sk-your-api-key-here", -- or set OPENAI_API_KEY environment variable
  model = "gpt-4o-2024-08-06",             -- AI model to use
  temperature = 0.7,                       -- Creativity level (0.0-1.0)
  timeout = 30000,                         -- Request timeout in milliseconds

  -- UI configuration
  ui_mode = "input",                       -- "input" or "telescope"

  -- Theme storage
  save_themes = true,                      -- Automatically save generated themes
  themes_dir = vim.fn.expand("~/.local/share/nvim/hexwitch-themes"),

  -- Debugging
  debug = false,                           -- Enable debug logging
})
```

### Configuration Options

| Option            | Type      | Default                          | Description                                                    |
|-------------------|-----------|----------------------------------|----------------------------------------------------------------|
| `openai_api_key`  | `string`  | `nil`                            | OpenAI API key (can also be set via `OPENAI_API_KEY` env var) |
| `model`           | `string`  | `"gpt-4o-2024-08-06"`            | OpenAI model to use for generation                            |
| `temperature`     | `number`  | `0.7`                            | AI creativity level (0.0-1.0, higher = more creative)         |
| `timeout`         | `number`  | `30000`                          | HTTP request timeout in milliseconds                          |
| `ui_mode`         | `string`  | `"input"`                        | UI mode: `"input"` or `"telescope"`                           |
| `save_themes`     | `boolean` | `true`                           | Automatically save generated themes                           |
| `themes_dir`      | `string`  | `"~/.local/share/nvim/hexwitch-themes"` | Directory to save themes                                |
| `debug`           | `boolean` | `false`                          | Enable debug logging                                           |

## Usage

### Basic Usage

The simplest way to use hexwitch is with the `:Hexwitch` command:

```vim
:Hexwitch a dark theme with purple accents and warm colors
```

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

### Telescope Integration (Optional)

If you have `telescope.nvim` installed and set `ui_mode = "telescope"`, you can use the enhanced UI:

```lua
-- Open telescope interface
require("hexwitch.ui.telescope").open_hexwitch()
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

This will check:
- OpenAI API key configuration
- Required dependencies (plenary.nvim)
- Optional dependencies (telescope.nvim)
- Theme storage directory permissions
- Network connectivity

## Theme Structure

Generated themes include comprehensive color definitions for:

- Editor UI: Background, foreground, cursor, selection
- Syntax Highlighting: All major language constructs
- LSP Diagnostics: Error, warning, info, hint highlights
- TreeSitter: Enhanced syntax parsing highlights
- Git Signs: Added, modified, removed indicators
- Telescope: Customized interface colors (if installed)
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
:lua print(require("hexwitch").config.timeout)
```

Theme Storage Issues
```vim
" Check themes directory permissions
:!ls -la ~/.local/share/nvim/hexwitch-themes

" Create directory manually
:!mkdir -p ~/.local/share/nvim/hexwitch-themes
```

### Debug Commands

```vim
" View debug logs
:messages

" Check current configuration
:lua print(vim.inspect(require("hexwitch").config))

" Test API connectivity
:lua require("hexwitch.ai.openai").test_connection()
```

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Code style and conventions
- Testing requirements
- Submitting pull requests
- Reporting issues

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

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