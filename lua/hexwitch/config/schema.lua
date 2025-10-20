---@class hexwitch.Config
---User-facing configuration (all fields optional)
local M = {}

---@class hexwitch.UserConfig
---@field openai_api_key? string OpenAI API key (defaults to OPENAI_API_KEY env var)
---@field model? string OpenAI model to use (default: "gpt-4o-2024-08-06")
---@field temperature? number Creativity level 0-2 (default: 0.7)
---@field ui_mode? "input"|"telescope" Input method (default: "input")
---@field save_themes? boolean Enable theme saving (default: true)
---@field themes_dir? string Directory for saved themes
---@field timeout? number API timeout in ms (default: 30000)
---@field debug? boolean Enable debug logging (default: false)

---@class hexwitch.InternalConfig
---@field openai_api_key string
---@field model string
---@field temperature number
---@field ui_mode "input"|"telescope"
---@field save_themes boolean
---@field themes_dir string
---@field timeout number
---@field debug boolean

---Default configuration values
---@type hexwitch.InternalConfig
M.defaults = {
  openai_api_key = vim.env.OPENAI_API_KEY or "",
  model = "gpt-4o-2024-08-06",
  temperature = 0.7,
  ui_mode = "input",
  save_themes = true,
  themes_dir = vim.fn.stdpath("data") .. "/hexwitch-themes",
  timeout = 30000,
  debug = false,
}

return M

