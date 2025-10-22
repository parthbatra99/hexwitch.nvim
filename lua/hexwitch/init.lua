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
    ui.prompt()
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

  local ui = get_ui()
  notify.info("🧙‍♀️ Hexwitch is brewing your theme...")
  local loading_handle
  if ui and ui.show_loading then
    loading_handle = ui.show_loading("🧙‍♀️ Hexwitch is brewing your theme...")
  end

  local ai = get_ai()
  ai.generate(user_input, function(colorscheme_data, err)
    if loading_handle and loading_handle.close then
      loading_handle.close()
    end
    if err then
      if ui and ui.show_error then
        ui.show_error("Failed to generate theme", err)
      else
        notify.error("Failed to generate theme: " .. err)
      end
      return
    end

    local theme = get_theme()
    theme.apply(colorscheme_data)

    local theme_name = colorscheme_data.name or "unnamed"
    local ok_refine, refinement = pcall(require, "hexwitch.ui.refinement")
    if ok_refine and refinement and refinement.set_current_theme then
      refinement.set_current_theme(colorscheme_data)
    end
    if ui and ui.show_success then
      ui.show_success("✨ Theme '" .. theme_name .. "' applied!", theme_name, colorscheme_data)
    else
      notify.info("✨ Theme '" .. theme_name .. "' applied!")
    end
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

---List saved themes
---@return table themes List of theme names
function M.list_themes()
  local storage = require("hexwitch.theme.storage")
  return storage.list()
end

---Delete a saved theme
---@param theme_name string Name of theme to delete
function M.delete(theme_name)
  local notify = require("hexwitch.utils.notify")
  if not theme_name or theme_name == "" then
    notify.warn("Please provide a theme name")
    return
  end
  local storage = require("hexwitch.theme.storage")
  storage.delete(theme_name)
end

---Undo last theme change
function M.undo()
  local undo_mod = require("hexwitch.undo")
  undo_mod.undo()
end

---Redo theme change
function M.redo()
  local undo_mod = require("hexwitch.undo")
  undo_mod.redo()
end

return M
