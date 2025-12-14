local config = require("hexwitch.config")
local state = require("hexwitch.storage.state")
local logger = require("hexwitch.utils.logger")
local color_utils = require("hexwitch.utils.color")

local M = {}

-- Current theme being refined
local current_theme = nil

-- Original theme before any refinements (for restoration)
local original_theme = nil

-- Track which adjustments have been applied to prevent repeated adjustments
local applied_adjustments = {
  contrast = nil,
  temperature = nil,
  saturation = nil,
  brightness = nil
}

local function apply_original_theme()
  if not original_theme or not original_theme.colors then
    vim.notify("No original theme available to restore", vim.log.levels.WARN)
    return
  end

  logger.info("ui.refinement", "apply_original_theme", "Restoring original theme")

  local restored_theme = vim.deepcopy(original_theme)

  require("hexwitch.theme").apply(restored_theme)

  state.add_to_undo_stack(restored_theme, "refinement")

  -- Reset adjustment tracking
  applied_adjustments = {
    contrast = nil,
    temperature = nil,
    saturation = nil,
    brightness = nil
  }

  vim.notify("Restored original theme", vim.log.levels.INFO)
end

-- Quick adjustment functions
local adjustments = {
  contrast = {
    increase = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.increase_contrast, colors.bg, config.get().contrast_threshold) end,
    decrease = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.decrease_contrast, colors.bg, 10) end,
  },
  temperature = {
    warmer = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.adjust_hue, 15) end,
    cooler = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.adjust_hue, -15) end,
  },
  saturation = {
    more_vibrant = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.adjust_saturation, 20) end,
    more_muted = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.adjust_saturation, -20) end,
  },
  brightness = {
    lighter = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.adjust_lightness, 15) end,
    darker = function(colors) return color_utils.apply_theme_adjustment(colors, color_utils.adjust_lightness, -15) end,
  },
}

-- Get current theme colors
local function get_current_theme_colors()
  if not current_theme then
    -- Try to get current colors from Neovim highlights
    local colors = {}
    local normal_hl = vim.api.nvim_get_hl(0, { name = "Normal" })

    if normal_hl.bg and normal_hl.fg then
      colors.bg = string.format("#%06x", normal_hl.bg)
      colors.fg = string.format("#%06x", normal_hl.fg)
    else
      -- Fallback colors
      colors.bg = "#1a1a1a"
      colors.fg = "#e0e0e0"
    end

    -- Get other colors from highlight groups
    local color_map = {
      red = "Error",
      green = "String",
      yellow = "WarningMsg",
      blue = "Function",
      purple = "Keyword",
      cyan = "Operator",
      orange = "Number",
      magenta = "Constant",
      comment = "Comment",
      selection = "Visual",
      cursor = "Cursor",
    }

    for color_name, hl_group in pairs(color_map) do
      local hl = vim.api.nvim_get_hl(0, { name = hl_group })
      if hl.fg then
        colors[color_name] = string.format("#%06x", hl.fg)
      end
    end

    -- Set sensible defaults for missing colors
    local defaults = {
      bg_sidebar = colors.bg,
      bg_float = colors.bg,
      bg_statusline = colors.bg,
      red = "#ff6b6b",
      green = "#51cf66",
      yellow = "#ffd43b",
      blue = "#339af0",
      purple = "#9775fa",
      cyan = "#22b8cf",
      orange = "#ff922b",
      magenta = "#e64980",
      comment = "#868e96",
      selection = "#343a40",
      cursor = "#f8f9fa",
    }

    for key, value in pairs(defaults) do
      colors[key] = colors[key] or value
    end

    current_theme = { colors = colors, name = "current_theme" }
  end

  return current_theme.colors
end

-- Apply refinement adjustment
---@param adjustment_type string Type of adjustment
---@param direction string Direction (increase/decrease, warmer/cooler, etc.)
local function apply_adjustment(adjustment_type, direction)
  -- Check if this adjustment type was already applied
  if applied_adjustments[adjustment_type] then
    local applied_direction = applied_adjustments[adjustment_type]
    vim.notify(
      adjustment_type:gsub("^%l", string.upper) .. " already adjusted (" .. applied_direction ..
      "). Press [Ctrl+O] to restore original and try again",
      vim.log.levels.WARN
    )
    return
  end

  local colors = get_current_theme_colors()
  local adjustment = adjustments[adjustment_type]

  if not adjustment or not adjustment[direction] then
    vim.notify("Invalid adjustment: " .. adjustment_type .. " " .. direction, vim.log.levels.ERROR)
    return
  end

  logger.info("ui.refinement", "apply_adjustment",
    string.format("Applying %s %s adjustment", adjustment_type, direction),
    { adjustment_type = adjustment_type, direction = direction })

  local new_colors = adjustment[direction](colors)

  -- Apply the refined theme
  local refined_theme = {
    name = (current_theme.name or "refined") .. "_refined",
    description = "Refined theme with " .. adjustment_type .. " " .. direction,
    colors = new_colors,
  }

  require("hexwitch.theme").apply(refined_theme)

  -- Update current theme to the refined version
  current_theme = refined_theme

  -- Track that this adjustment was applied
  applied_adjustments[adjustment_type] = direction

  -- Add to undo stack
  state.add_to_undo_stack(refined_theme, "refinement")

  vim.notify("Applied " .. adjustment_type .. " " .. direction .. " adjustment", vim.log.levels.INFO)

  -- Show save options after refinement
  vim.defer_fn(function()
    vim.notify("Press [S] in refinement window to save this refined theme", vim.log.levels.INFO)
  end, 1000)
end

-- Show save options for refined theme
local function show_save_options()
  local theme_name = current_theme and current_theme.name or "refined_theme"

  logger.info("ui.refinement", "show_save_options", "Showing save options for refined theme",
    { theme_name = theme_name })

  require("hexwitch.ui.telescope.input").show_save_dialog(theme_name, function(name)
    if name and name ~= "" then
      -- Create a copy of current theme with the new name
      local theme_to_save = vim.deepcopy(current_theme)
      theme_to_save.name = name

      -- Save the theme
      require("hexwitch").save(name)
      vim.notify("Theme '" .. name .. "' saved successfully!", vim.log.levels.INFO)
    end
  end)
end

-- Open refinement UI
function M.open(opts)
  opts = opts or {}
  local cfg = config.get()

  logger.info("ui.refinement", "open", "Opening theme refinement UI")

  local content = {
    "[EDIT] Refine Theme",
    "================",
    "",
    "Quick Adjustments:",
    "",
  }

  table.insert(content, "[CONTRAST] Contrast:    [I]ncrease  [D]ecrease")
  table.insert(content, "[TEMP] Temperature: [W]armer   [C]ooler")
  table.insert(content, "[COLOR] Saturation:  [M]ore      [L]ess")
  table.insert(content, "[BRIGHT] Brightness:  [B]righter  [J]darker")

  table.insert(content, "")
  table.insert(content, "[Ctrl+O] Restore completely original theme")
  table.insert(content, "")
  table.insert(content, "Or describe specific changes:")
  table.insert(content, "")
  table.insert(content, "[Enter] to apply custom changes")
  table.insert(content, "[S]ave current theme")
  table.insert(content, "[R]eset to original")
  table.insert(content, "[Esc] to close")

  -- Create floating window
  local width = math.max(50, math.min(80, vim.fn.winwidth(0) * 0.6))
  local height = #content + 2
  local col = math.floor((vim.fn.winwidth(0) - width) / 2)
  local row = math.floor((vim.fn.winheight(0) - height) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, content)
  vim.api.nvim_buf_set_option(buf, "modifiable", false)
  vim.api.nvim_buf_set_option(buf, "readonly", true)
  vim.api.nvim_buf_set_option(buf, "filetype", "text")

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    col = col,
    row = row,
    border = cfg.ui.border,
    title = " Theme Refinement ",
    title_pos = "center",
    style = "minimal",
  })

  -- Set up keymaps
  local key_opts = { buffer = buf, silent = true, nowait = true }

  -- Contrast adjustments
  vim.keymap.set("n", "i", function()
    apply_adjustment("contrast", "increase")
  end, key_opts)

  vim.keymap.set("n", "I", function()
    apply_adjustment("contrast", "increase")
  end, key_opts)

  vim.keymap.set("n", "d", function()
    apply_adjustment("contrast", "decrease")
  end, key_opts)

  vim.keymap.set("n", "D", function()
    apply_adjustment("contrast", "decrease")
  end, key_opts)

  -- Temperature adjustments
  vim.keymap.set("n", "w", function()
    apply_adjustment("temperature", "warmer")
  end, key_opts)

  vim.keymap.set("n", "W", function()
    apply_adjustment("temperature", "warmer")
  end, key_opts)

  vim.keymap.set("n", "c", function()
    apply_adjustment("temperature", "cooler")
  end, key_opts)

  vim.keymap.set("n", "C", function()
    apply_adjustment("temperature", "cooler")
  end, key_opts)

  -- Saturation adjustments
  vim.keymap.set("n", "m", function()
    apply_adjustment("saturation", "more_vibrant")
  end, key_opts)

  vim.keymap.set("n", "M", function()
    apply_adjustment("saturation", "more_vibrant")
  end, key_opts)

  vim.keymap.set("n", "l", function()
    apply_adjustment("saturation", "more_muted")
  end, key_opts)

  vim.keymap.set("n", "L", function()
    apply_adjustment("saturation", "more_muted")
  end, key_opts)

  -- Brightness adjustments
  vim.keymap.set("n", "b", function()
    apply_adjustment("brightness", "lighter")
  end, key_opts)

  vim.keymap.set("n", "B", function()
    apply_adjustment("brightness", "lighter")
  end, key_opts)

  vim.keymap.set("n", "j", function()
    apply_adjustment("brightness", "darker")
  end, key_opts)

  vim.keymap.set("n", "J", function()
    apply_adjustment("brightness", "darker")
  end, key_opts)

  vim.keymap.set("n", "<C-o>", function()
    apply_original_theme()
  end, key_opts)

  -- Apply custom refinements
  vim.keymap.set("n", cfg.keymaps.confirm, function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })

    -- Show refinement input dialog
    require("hexwitch.ui.telescope.input").show_refinement_input(function(description)
      M.apply_custom_refinement(description)
    end)
  end, key_opts)

  -- Reset to original
  vim.keymap.set("n", "r", function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })
    require("hexwitch").undo()
  end, key_opts)

  vim.keymap.set("n", "R", function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })
    require("hexwitch").undo()
  end, key_opts)

  -- Save theme
  vim.keymap.set("n", "s", function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })
    show_save_options()
  end, key_opts)

  vim.keymap.set("n", "S", function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })
    show_save_options()
  end, key_opts)

  -- Close
  vim.keymap.set("n", cfg.keymaps.close, function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })
  end, key_opts)

  vim.keymap.set("n", cfg.keymaps.cancel, function()
    vim.api.nvim_win_close(win, true)
    vim.api.nvim_buf_delete(buf, { force = true })
  end, key_opts)
end

-- Apply custom refinement description
---@param description string Custom refinement description
function M.apply_custom_refinement(description)
  logger.info("ui.refinement", "apply_custom_refinement",
    "Applying custom refinement", { description = description })

  local colors = get_current_theme_colors()

  -- Parse simple descriptions and apply algorithmic changes
  local description_lower = description:lower()
  local new_colors = vim.deepcopy(colors)

  -- Simple pattern matching for common requests
  if description_lower:match("darker") or description_lower:match("dim") then
    new_colors = color_utils.apply_theme_adjustment(colors, color_utils.adjust_lightness, -20)
  elseif description_lower:match("lighter") or description_lower:match("bright") then
    new_colors = color_utils.apply_theme_adjustment(colors, color_utils.adjust_lightness, 20)
  elseif description_lower:match("warmer") or description_lower:match("warm") then
    new_colors = color_utils.apply_theme_adjustment(colors, color_utils.adjust_hue, 20)
  elseif description_lower:match("cooler") or description_lower:match("cool") then
    new_colors = color_utils.apply_theme_adjustment(colors, color_utils.adjust_hue, -20)
  elseif description_lower:match("more contrast") or description_lower:match("higher contrast") then
    new_colors = color_utils.apply_theme_adjustment(colors, color_utils.increase_contrast, colors.bg, 6.0)
  elseif description_lower:match("less contrast") or description_lower:match("lower contrast") then
    new_colors = color_utils.apply_theme_adjustment(colors, color_utils.decrease_contrast, colors.bg, 10)
  end

  -- Apply the refined theme
  local refined_theme = {
    name = (current_theme.name or "refined") .. "_custom",
    description = "Custom refinement: " .. description,
    colors = new_colors,
  }

  require("hexwitch.theme").apply(refined_theme)

  -- Update current theme to the refined version
  current_theme = refined_theme

  -- Reset adjustment tracking after custom refinement
  applied_adjustments = {
    contrast = nil,
    temperature = nil,
    saturation = nil,
    brightness = nil
  }

  -- Add to undo stack
  state.add_to_undo_stack(refined_theme, "refinement")

  vim.notify("Applied custom refinement: " .. description, vim.log.levels.INFO)

  -- Show save options after refinement
  vim.defer_fn(function()
    vim.notify("Press [S] in refinement window to save this refined theme", vim.log.levels.INFO)
  end, 1000)
end


-- Set current theme for refinement
---@param theme_data table Theme data
function M.set_current_theme(theme_data)
  current_theme = theme_data
  original_theme = vim.deepcopy(theme_data)

  -- Reset adjustment tracking for new theme
  applied_adjustments = {
    contrast = nil,
    temperature = nil,
    saturation = nil,
    brightness = nil
  }

  logger.debug("ui.refinement", "set_current_theme", "Set current theme for refinement",
    { theme_name = theme_data.name })
end

return M