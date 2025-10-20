local config = require("hexwitch.config")
local notify = require("hexwitch.utils.notify")

local M = {}

-- Lazy-load heavy modules
local function get_ui()
  return require("hexwitch.ui")
end

local function get_theme()
  return require("hexwitch.theme")
end

local function get_ai()
  return require("hexwitch.ai")
end

---Setup hexwitch with user configuration
---@param opts? hexwitch.UserConfig
function M.setup(opts)
  local success, err = config.setup(opts)
  if not success then
    notify.error(err or "Configuration failed")
    return
  end

  notify.debug("hexwitch.nvim initialized successfully")
end

---Prompt user for theme description
function M.prompt()
  local ui = get_ui()
  local cfg = config.get()

  if cfg.ui_mode == "telescope" then
    ui.telescope_prompt()
  else
    ui.simple_prompt()
  end
end

---Generate and apply theme from user input
---@param user_input string Theme description
function M.generate(user_input)
  if not user_input or user_input == "" then
    notify.warn("Theme description cannot be empty")
    return
  end

  notify.info("🧙‍♀️ Hexwitch is brewing your theme...")

  local ai = get_ai()
  ai.generate(user_input, function(colorscheme_data, err)
    if err then
      notify.error("Failed to generate theme: " .. err)
      return
    end

    local theme = get_theme()
    theme.apply(colorscheme_data)

    local theme_name = colorscheme_data.name or "unnamed"
    notify.info("✨ Theme '" .. theme_name .. "' applied!")
  end)
end

---Save currently applied theme
---@param theme_name string Name to save theme as
function M.save(theme_name)
  if not theme_name or theme_name == "" then
    notify.warn("Please provide a theme name")
    return
  end

  local theme = get_theme()
  theme.save(theme_name)
end

---Load a saved theme
---@param theme_name string Name of saved theme
function M.load(theme_name)
  if not theme_name or theme_name == "" then
    notify.warn("Please provide a theme name")
    return
  end

  local theme = get_theme()
  theme.load(theme_name)
end

return M

