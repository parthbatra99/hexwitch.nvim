# hexwitch.nvim

<p align="center">
  <img src="./hexwitch-logo.png" alt="Hexwitch Logo" width="200" />
</p>

<p align="center">
  AI-powered colorscheme generator for Neovim.
</p>

Ever wanted to create a Neovim colorscheme just by describing it? Now you can. `hexwitch.nvim` uses OpenAI's models to generate and apply a colorscheme from a simple text prompt.

## Requirements

- Neovim ≥ 0.9.0
- `curl`
- An [OpenAI API key](https://platform.openai.com/api-keys)
- `plenary.nvim`
- `telescope.nvim`

## Installation

Use your favorite plugin manager. Here is an example with `lazy.nvim`:

```lua
{
  "parthbatra99/hexwitch.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim",
  },
  config = function()
    require("hexwitch").setup()
  end,
}
```

## Getting Started

1.  **Set your API key:** The plugin needs an API key for your chosen AI provider. Set it as an environment variable.

    ```bash
    # For OpenAI
    export OPENAI_API_KEY="sk-..."

    # For OpenRouter
    export OPENROUTER_API_KEY="sk-or-..."
    ```
    *Note: Never commit your API key to a public repository.*

2.  **Generate a theme:** Use the `:Hexwitch` command with a description of the theme you want.

    ```vim
    :Hexwitch a dark, high-contrast theme with solarized accents
    ```

    And that's it! Hexwitch will generate the colors and apply them.

## Configuration

You can customize the plugin by passing a table to the `setup()` function. Here are the defaults:

```lua
require("hexwitch").setup({
  ai_provider = "openai",           -- "openai", "openrouter", or "custom"
  model = "gpt-4o-mini",           -- AI model to use
  temperature = 0.7,               -- 0.0–2.0. Higher means more creative/random palettes
  timeout = 30000,                 -- API timeout in milliseconds
  save_themes = true,              -- Save generated themes to disk
  themes_dir = vim.fn.stdpath("data") .. "/hexwitch",  -- Directory for saved themes
  max_history = 50,                -- Maximum history entries
  auto_save_history = true,        -- Save generation history
  contrast_threshold = 4.5,        -- Minimum WCAG contrast ratio
  debug = false,                   -- Enable debug logging
})
```

## Commands

- `:Hexwitch [prompt]` - Generate a theme from a text description.
- `:Hexwitch quick` - Generate a variation of the current theme.
- `:Hexwitch random` - Generate a random theme.
- `:Hexwitch refine [changes]` - Tweak the colors of the current theme.
- `:Hexwitch browse` - Browse saved themes using Telescope.
- `:Hexwitch history` - View your generation history.
- `:Hexwitch save <name>` - Save the current theme.
- `:Hexwitch load <name>` - Load a saved theme.
- `:Hexwitch undo` / `:Hexwitch redo` - Undo or redo theme changes.

## Debugging

If you encounter issues, try these steps:

1. **Enable debug mode**:
   ```lua
   require("hexwitch").setup({ debug = true })
   ```

2. **Common issues**:
   - **API key**: Ensure your `OPENAI_API_KEY` or `OPENROUTER_API_KEY` is set
   - **Network**: Check internet connection and API status
   - **Theme fails**: Verify `curl` is installed and API key is valid

3. **Basic checks**:
   ```vim
   :checkhealth hexwitch  " Check plugin health
   :lua print(vim.env.OPENAI_API_KEY and "API key found" or "API key missing")
   ```

4. **Reset storage**: Themes are saved locally at:
   ```bash
   # Default location
   ~/.local/share/nvim/hexwitch/

   # Delete to reset all saved themes and history
   rm -rf ~/.local/share/nvim/hexwitch/
   ```

5. **Get help**: Check debug output with `:messages` and report issues with debug logs.

## Telescope UI

Hexwitch provides a built-in telescope interface for browsing saved themes, presets, and your generation history. The plugin automatically uses telescope for all UI interactions.

## Test Drive

Want to try Hexwitch without affecting your main Neovim config? There's an example `init.lua` you can use for an isolated test environment.

1.  Make sure your `OPENAI_API_KEY` is set.
2.  Run Neovim with the following command:

    ```bash
    NVIM_APPNAME=hexwitch-test nvim -u examples/nvim-test/init.lua
    ```

This starts Neovim with a temporary configuration that loads only Hexwitch and its dependencies. The example config also sets up some handy keymaps for you:

-   `<leader>hw`: Open the Hexwitch prompt
-   `<leader>hb`: Browse saved themes
-   `<leader>hh`: View generation history
-   `<leader>hq`: Open the quick actions menu
-   `<leader>hu`: Undo last theme change
-   `<leader>hr`: Redo theme change

*Tip: If you're developing the plugin locally, you can use your local version by setting the `HEXWITCH_PLUGIN_DIR` environment variable.*

## Contributing

This is a personal side project, but feel free to open an issue or a pull request if you have ideas or find a bug.

## License

MIT
