local logger = require("hexwitch.utils.logger")
local config = require("hexwitch.config")

local M = {}

-- Lazy-load UI components
local function get_telescope()
  return require("hexwitch.ui.telescope")
end

local function get_refinement()
  return require("hexwitch.ui.refinement")
end

local function get_sliders()
  return require("hexwitch.ui.sliders")
end

-- Show theme generation prompt
function M.prompt()
  logger.info("ui.init", "prompt", "Opening theme generation prompt")
  local telescope = get_telescope()
  if not telescope then
    logger.error("ui.init", "prompt", "Failed to load telescope module")
    vim.notify("Failed to load telescope UI module", vim.log.levels.ERROR)
    return
  end

  if not telescope.input then
    logger.error("ui.init", "prompt", "Telescope input module is nil")
    vim.notify("Telescope input module is not available", vim.log.levels.ERROR)
    return
  end

  if not telescope.input.show_examples then
    logger.error("ui.init", "prompt", "show_examples function not available")
    vim.notify("show_examples function not available", vim.log.levels.ERROR)
    return
  end

  telescope.input.show_examples()
end


-- Show quick actions menu
function M.quick_actions()
  logger.info("ui.init", "quick_actions", "Opening quick actions menu")
  get_telescope().quick_actions()
end

-- Browse saved themes
function M.browse_themes()
  logger.info("ui.init", "browse_themes", "Opening theme browser")
  get_telescope().browse_themes()
end

-- Show generation history
function M.show_history()
  logger.info("ui.init", "show_history", "Opening generation history")
  get_telescope().show_history()
end

-- Open refinement interface
function M.refine_theme()
  logger.info("ui.init", "refine_theme", "Opening theme refinement")
  get_refinement().open()
end

-- Open sliders interface
function M.open_sliders(theme)
  logger.info("ui.init", "open_sliders", "Opening theme sliders")
  return get_sliders().create_slider_window(theme)
end

-- Apply slider adjustments
function M.apply_slider_adjustments(theme, adjustments)
  logger.info("ui.init", "apply_slider_adjustments", "Applying slider adjustments")
  return get_sliders().apply_slider_adjustments(theme, adjustments)
end

-- Show loading state
---@param message string Loading message
---@return table handle Handle for closing the window
function M.show_loading(message)
  logger.info("ui.init", "show_loading", "Showing loading state", { message = message })
  return get_telescope().notifications.show_loading(message)
end

-- Show success message
---@param message string Success message
---@param theme_name string Theme name
---@param theme_data table Theme data for preview
function M.show_success(message, theme_name, theme_data)
  logger.info("ui.init", "show_success", "Showing success message",
    { message = message, theme_name = theme_name })
  get_telescope().notifications.show_success(message, theme_name, theme_data)
end

-- Show error message
---@param message string Error message
---@param error_details string Detailed error information
function M.show_error(message, error_details)
  logger.warn("ui.init", "show_error", "Showing error message",
    { message = message, details = error_details })
  get_telescope().notifications.show_error(message, error_details)
end

-- Show warning message
---@param message string Warning message
function M.show_warning(message)
  logger.warn("ui.init", "show_warning", "Showing warning message", { message = message })
  get_telescope().notifications.show_warning(message)
end

-- Show info message
---@param message string Info message
function M.show_info(message)
  logger.info("ui.init", "show_info", "Showing info message", { message = message })
  get_telescope().notifications.show_info(message)
end

-- Close all UI windows
function M.close_all()
  logger.debug("ui.init", "close_all", "Closing all UI windows")
  get_telescope().notifications.close_all()
end

-- Check if UI dependencies are available
---@return boolean available True if dependencies are available
function M.is_available()
  local ok, telescope = pcall(require, "telescope")
  if not ok then
    logger.warn("ui.init", "is_available", "Telescope not available")
    return false
  end
  return true
end

-- Validate UI configuration
---@return boolean valid True if configuration is valid
---@return string|nil error Error message if invalid
function M.validate_config()
  local cfg = config.get()

  -- Check if telescope is available
  local ok, telescope = pcall(require, "telescope")
  if not ok then
    return false, "telescope.nvim is required"
  end

  -- Validate window dimensions
  if cfg.ui.width_ratio <= 0 or cfg.ui.width_ratio > 1 then
    return false, "ui.width_ratio must be between 0 and 1"
  end

  if cfg.ui.height_ratio <= 0 or cfg.ui.height_ratio > 1 then
    return false, "ui.height_ratio must be between 0 and 1"
  end

  -- Validate border style
  local valid_borders = { "none", "single", "double", "rounded" }
  if not vim.tbl_contains(valid_borders, cfg.ui.border) then
    return false, "ui.border must be one of: " .. table.concat(valid_borders, ", ")
  end

  return true, nil
end

return M
