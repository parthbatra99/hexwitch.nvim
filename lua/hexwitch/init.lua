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
  ui.prompt()
end

---Generate and apply theme from user input
---@param user_input string Theme description
---@param callback? fun(data: table|nil, err: string|nil) Optional completion callback
function M.generate(user_input, callback)
  if not user_input or user_input == "" then
    notify.warn("Theme description cannot be empty")
    return
  end

  local ui = get_ui()
  notify.info("Generating your theme...")
  local loading_handle
  if ui and ui.show_loading then
    local ok, handle = pcall(ui.show_loading, "Generating your theme...")
    if ok then loading_handle = handle end
  end

  local ai = get_ai()
  ai.generate(user_input, function(colorscheme_data, err)
    if loading_handle and loading_handle.close then
      loading_handle.close()
    end
    if err then
      if ui and ui.show_error then
        pcall(ui.show_error, "Failed to generate theme", err)
      else
        notify.error("Failed to generate theme: " .. err)
      end
      if callback then callback(nil, err) end
      return
    end

    local theme = get_theme()
    theme.apply(colorscheme_data)

    local theme_name = colorscheme_data.name or "unnamed"
    if ui and ui.show_success then
      pcall(ui.show_success, "[SUCCESS] Theme '" .. theme_name .. "' applied!", theme_name, colorscheme_data)
    else
      notify.info("[SUCCESS] Theme '" .. theme_name .. "' applied!")
    end
    if callback then callback(colorscheme_data, nil) end
  end)
end

---Refine the currently applied theme via AI
---@param change string Natural-language refinement request
---@param callback? fun(data: table|nil, err: string|nil) Optional completion callback
function M.refine(change, callback)
  if not change or change == "" then
    notify.warn("Refinement description cannot be empty")
    return
  end

  local ui = get_ui()
  notify.info("Refining your theme...")
  local loading_handle
  if ui and ui.show_loading then
    local ok, handle = pcall(ui.show_loading, "Refining your theme...")
    if ok then loading_handle = handle end
  end

  local ai = get_ai()
  ai.refine(change, function(colorscheme_data, err)
    if loading_handle and loading_handle.close then
      loading_handle.close()
    end
    if err then
      if ui and ui.show_error then
        pcall(ui.show_error, "Failed to refine theme", err)
      else
        notify.error("Failed to refine theme: " .. err)
      end
      if callback then callback(nil, err) end
      return
    end

    local theme = get_theme()
    theme.apply(colorscheme_data)

    local theme_name = colorscheme_data.name or "unnamed"
    if ui and ui.show_success then
      pcall(ui.show_success, "[SUCCESS] Theme '" .. theme_name .. "' refined!", theme_name, colorscheme_data)
    else
      notify.info("[SUCCESS] Theme '" .. theme_name .. "' refined!")
    end
    if callback then callback(colorscheme_data, nil) end
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

return M
