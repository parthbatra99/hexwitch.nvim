---@class hexwitch.Config
---User-facing configuration (all fields optional)
local M = {}

---@class hexwitch.UserConfig
---@field ai_provider? "openai"|"openrouter"|"custom" AI provider to use (default: "openai")
---@field api_key? string API key (defaults to provider-specific env var)
---@field openai_api_key? string OpenAI API key (defaults to OPENAI_API_KEY env var) [DEPRECATED use api_key]
---@field model? string AI model to use (default: "gpt-4o-2024-08-06")
---@field temperature? number Creativity level 0-2 (default: 0.7)
---@field timeout? number API timeout in ms (default: 30000)
---@field save_themes? boolean Enable theme saving (default: true)
---@field themes_dir? string Directory for saved themes
---@field max_history? number Maximum history entries (default: 50)
---@field auto_save_history? boolean Save generation history (default: true)
---@field contrast_threshold? number Minimum WCAG contrast ratio (default: 4.5)
---@field debug? boolean Enable debug logging (default: false)
---@field ui? hexwitch.UIConfig UI appearance settings
---@field keymaps? hexwitch.KeymapConfig Keybinding settings
---@field on_theme_applied? function Callback when theme applied
---@field on_theme_saved? function Callback when theme saved
---@field on_error? function Callback on errors

---@class hexwitch.InternalConfig
---@field ai_provider "openai"|"openrouter"|"custom"
---@field api_key string
---@field model string
---@field temperature number
---@field timeout number
---@field save_themes boolean
---@field themes_dir string
---@field max_history number
---@field auto_save_history boolean
---@field contrast_threshold number
---@field debug boolean
---@field ui hexwitch.UIConfig
---@field keymaps hexwitch.KeymapConfig

---@class hexwitch.UIConfig
---@field border "none"|"single"|"double"|"rounded" Window border style (default: "rounded")
---@field width_ratio number Floating window width ratio (default: 0.6)
---@field height_ratio number Floating window height ratio (default: 0.4)
---@field icons boolean Show emoji/icons (default: true)
---@field auto_preview boolean Show theme previews in telescope (default: true)
---@field compact_mode boolean Use more compact telescope layout (default: false)

---@class hexwitch.KeymapConfig
---@field close string Close key (default: "<Esc>")
---@field confirm string Confirm key (default: "<CR>")
---@field cancel string Cancel key (default: "<C-c>")
---@field next string Next key (default: "<Tab>")
---@field prev string Previous key (default: "<S-Tab>")

---Default configuration values
---@type hexwitch.InternalConfig
M.defaults = {
  ai_provider = "openai",
  api_key = vim.env.OPENAI_API_KEY or vim.env.HEXWITCH_API_KEY or "",
  model = "gpt-4o-mini",
  temperature = 0.7,
  timeout = 30000,
  save_themes = true,
  themes_dir = vim.fn.stdpath("data") .. "/hexwitch",
  max_history = 50,
  auto_save_history = true,
  contrast_threshold = 4.5,
  debug = false,
  ui = {
    border = "rounded",
    width_ratio = 0.6,
    height_ratio = 0.4,
    icons = true,
    auto_preview = true,
    compact_mode = false,
  },
  keymaps = {
    close = "<Esc>",
    confirm = "<CR>",
    cancel = "<C-c>",
    next = "<Tab>",
    prev = "<S-Tab>",
  },
}

return M

