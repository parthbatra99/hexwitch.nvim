local notify = require("hexwitch.utils.notify")

local M = {}

-- Get the directory for storing themes
local function get_theme_dir()
  local config_dir = vim.fn.stdpath("data") .. "/hexwitch"
  vim.fn.mkdir(config_dir, "p")
  return config_dir
end

-- Get the full path for a theme file
local function get_theme_path(theme_name)
  return get_theme_dir() .. "/" .. theme_name .. ".json"
end

---Save the current theme
---@param theme_name string Name to save theme as
function M.save(theme_name)
  if not theme_name or theme_name == "" then
    notify.error("Theme name cannot be empty")
    return
  end

  -- Get current colorscheme data
  local colorscheme_data = {}

  -- Try to get the colorscheme name
  colorscheme_data.name = vim.g.colors_name or "unknown"
  colorscheme_data.description = "Saved theme: " .. theme_name
  colorscheme_data.colors = {}

  -- Extract current highlight colors
  local highlight_groups = {
    "Normal", "NormalFloat", "NormalSB", "StatusLine", "StatusLineNC",
    "Cursor", "CursorLine", "CursorLineNr", "Visual", "VisualNOS",
    "Comment", "Constant", "String", "Character", "Number", "Boolean", "Float",
    "Identifier", "Function", "Keyword", "Conditional", "Repeat", "Label",
    "Operator", "Exception", "PreProc", "Include", "Define", "Macro", "PreCondit",
    "Type", "StorageClass", "Structure", "Typedef", "Special", "SpecialChar",
    "Tag", "Delimiter", "SpecialComment", "Error", "Todo", "Underlined", "Ignore"
  }

  for _, group in ipairs(highlight_groups) do
    local hl = vim.api.nvim_get_hl(0, { name = group })
    if hl and (hl.fg or hl.bg) then
      colorscheme_data.colors[group:lower()] = {
        fg = hl.fg and string.format("#%06x", hl.fg) or nil,
        bg = hl.bg and string.format("#%06x", hl.bg) or nil,
      }
    end
  end

  -- Add terminal colors if available
  if vim.g.terminal_color_0 then
    colorscheme_data.terminal = {
      [0] = vim.g.terminal_color_0,
      [1] = vim.g.terminal_color_1,
      [2] = vim.g.terminal_color_2,
      [3] = vim.g.terminal_color_3,
      [4] = vim.g.terminal_color_4,
      [5] = vim.g.terminal_color_5,
      [6] = vim.g.terminal_color_6,
      [7] = vim.g.terminal_color_7,
      [8] = vim.g.terminal_color_8,
      [9] = vim.g.terminal_color_9,
      [10] = vim.g.terminal_color_10,
      [11] = vim.g.terminal_color_11,
      [12] = vim.g.terminal_color_12,
      [13] = vim.g.terminal_color_13,
      [14] = vim.g.terminal_color_14,
      [15] = vim.g.terminal_color_15,
    }
  end

  local theme_path = get_theme_path(theme_name)
  local file = io.open(theme_path, "w")
  if not file then
    notify.error("Failed to create theme file: " .. theme_path)
    return
  end

  local ok, json = pcall(vim.json.encode, colorscheme_data)
  if not ok then
    notify.error("Failed to encode theme data: " .. tostring(json))
    file:close()
    return
  end

  file:write(json)
  file:close()

  notify.info("Theme '" .. theme_name .. "' saved successfully")
end

---Load a saved theme
---@param theme_name string Name of saved theme to load
function M.load(theme_name)
  if not theme_name or theme_name == "" then
    notify.error("Theme name cannot be empty")
    return
  end

  local theme_path = get_theme_path(theme_name)
  local file = io.open(theme_path, "r")
  if not file then
    notify.error("Theme file not found: " .. theme_path)
    return
  end

  local content = file:read("*all")
  file:close()

  local ok, colorscheme_data = pcall(vim.json.decode, content)
  if not ok then
    notify.error("Failed to parse theme file: " .. tostring(colorscheme_data))
    return
  end

  -- Apply the loaded theme using the applier
  local applier = require("hexwitch.theme.applier")
  applier.apply(colorscheme_data)

  notify.info("Theme '" .. theme_name .. "' loaded successfully")
end

---List all saved themes
---@return table List of theme names
function M.list()
  local theme_dir = get_theme_dir()
  local handle = vim.fn.glob(theme_dir .. "/*.json", false, true)
  local themes = {}

  for _, file in ipairs(handle) do
    local theme_name = vim.fn.fnamemodify(file, ":t:r")
    table.insert(themes, theme_name)
  end

  return themes
end

---Delete a saved theme
---@param theme_name string Name of theme to delete
function M.delete(theme_name)
  if not theme_name or theme_name == "" then
    notify.error("Theme name cannot be empty")
    return
  end

  local theme_path = get_theme_path(theme_name)
  local ok, err = os.remove(theme_path)
  if not ok then
    notify.error("Failed to delete theme: " .. err)
    return
  end

  notify.info("Theme '" .. theme_name .. "' deleted successfully")
end

return M