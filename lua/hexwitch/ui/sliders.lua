local M = {}
local api = vim.api

local SLIDER_CONFIG = {
  temperature = { min = -60, max = 60, default = 0, step = 5 },
  contrast = { min = -30, max = 30, default = 0, step = 5 },
  saturation = { min = -50, max = 50, default = 0, step = 10 },
  brightness = { min = -40, max = 40, default = 0, step = 5 }
}

-- Open floating window helper function
local function open_floating_window(buf, config)
  config = config or {}
  local width = config.width or 50
  local height = config.height or 10

  -- Check if we're in a headless environment
  if not vim.api.nvim_list_uis() or #vim.api.nvim_list_uis() == 0 then
    -- Headless mode, just return the buffer
    return buf
  end

  -- Calculate center position
  local ui = vim.api.nvim_list_uis()[1]
  local win_width = ui.width
  local win_height = ui.height

  local row = math.floor((win_height - height) / 2)
  local col = math.floor((win_width - width) / 2)

  local win_config = {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    border = config.border or "single",
    title = config.title,
    title_pos = config.title_pos or "center",
    style = "minimal",
    noautocmd = true,
  }

  local win = api.nvim_open_win(buf, true, win_config)

  -- Set window options
  api.nvim_win_set_option(win, "wrap", false)
  api.nvim_win_set_option(win, "cursorline", true)
  api.nvim_win_set_option(win, "number", false)
  api.nvim_win_set_option(win, "relativenumber", false)
  api.nvim_win_set_option(win, "signcolumn", "no")

  return win
end

function M.create_slider_window(theme)
  local buf = api.nvim_create_buf(false, true)

  local content = {}
  table.insert(content, "Theme Refinement Studio")
  table.insert(content, string.rep("─", 30))
  table.insert(content, "")

  for name, config in pairs(SLIDER_CONFIG) do
    table.insert(content, string.format("%-12s: │%s│ %d",
      name:gsub("^%l", string.upper),
      string.rep(" ", 20),
      config.default
    ))
  end

  table.insert(content, "")
  table.insert(content, "[<C-Enter>] Apply [q] Cancel [r] Reset")

  api.nvim_buf_set_lines(buf, 0, -1, false, content)
  api.nvim_buf_set_option(buf, "modifiable", false)
  api.nvim_buf_set_option(buf, "readonly", true)

  open_floating_window(buf, {
    title = " Smart Refinement ",
    title_pos = "center",
    width = 35,
    height = #content,
    border = "rounded"
  })

  return buf
end

function M.apply_slider_adjustments(theme, adjustments)
  local color = require("hexwitch.utils.color")
  local adjusted = vim.deepcopy(theme)

  -- Map slider values to color adjustment functions
  for key, value in pairs(adjustments) do
    if key == "temperature" and value ~= 0 then
      -- Adjust hue (temperature)
      adjusted = color.apply_theme_adjustment(adjusted, color.adjust_hue, value)
    elseif key == "contrast" and value ~= 0 then
      -- Adjust contrast by modifying lightness
      if value > 0 then
        adjusted = color.apply_theme_adjustment(adjusted, function(c, v)
          if c == adjusted.background then
            return color.adjust_lightness(c, -v)
          elseif c == adjusted.foreground then
            return color.adjust_lightness(c, v)
          end
          return c
        end, math.abs(value))
      else
        adjusted = color.apply_theme_adjustment(adjusted, function(c, v)
          if c == adjusted.background then
            return color.adjust_lightness(c, v)
          elseif c == adjusted.foreground then
            return color.adjust_lightness(c, -v)
          end
          return c
        end, math.abs(value))
      end
    elseif key == "saturation" and value ~= 0 then
      -- Adjust saturation
      adjusted = color.apply_theme_adjustment(adjusted, color.adjust_saturation, value)
    elseif key == "brightness" and value ~= 0 then
      -- Adjust brightness (lightness)
      adjusted = color.apply_theme_adjustment(adjusted, color.adjust_lightness, value)
    end
  end

  return adjusted
end

return M