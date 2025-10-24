local notify = require("hexwitch.utils.notify")

local M = {}

-- Sanitize theme name to prevent path traversal attacks
local function sanitize_theme_name(name)
  if not name or type(name) ~= "string" then
    return nil
  end

  -- Remove path separators and dangerous characters including command injection patterns
  local sanitized = name:gsub("[/\\:%*%?\"<>|%c`$(){};&]", "_")

  -- Limit length and ensure it starts with alphanumeric
  sanitized = sanitized:sub(1, 50)
  if not sanitized:match("^[a-zA-Z0-9]") then
    sanitized = "_" .. sanitized
  end

  -- Ensure no null bytes or other injection patterns
  sanitized = sanitized:gsub("%z", "_")

  return sanitized
end

-- Get the directory for storing themes
local function get_theme_dir()
  local config_dir = vim.fn.stdpath("data") .. "/hexwitch"
  vim.fn.mkdir(config_dir, "p")
  return config_dir
end

-- Get the full path for a theme file with security validation
local function get_theme_path(theme_name)
  local safe_name = sanitize_theme_name(theme_name)
  if not safe_name or safe_name ~= theme_name then
    notify.error("Invalid theme name: theme names can only contain letters, numbers, underscores, and hyphens")
    return nil
  end

  local theme_path = get_theme_dir() .. "/" .. safe_name .. ".json"

  -- Ensure the path stays within the theme directory
  local resolved_path = vim.fn.resolve(theme_path)
  if not resolved_path:match("^" .. vim.pesc(get_theme_dir())) then
    notify.error("Security error: theme path attempted to escape theme directory")
    return nil
  end

  return resolved_path
end

---Read theme data without applying it
---@param theme_name string
---@return table|nil colorscheme_data
function M.read(theme_name)
  if not theme_name or theme_name == "" then
    return nil
  end

  local theme_path = get_theme_path(theme_name)
  if not theme_path then
    return nil
  end

  local file = io.open(theme_path, "r")
  if not file then
    return nil
  end

  local content = file:read("*all")
  file:close()

  local ok, colorscheme_data = pcall(vim.json.decode, content)
  if not ok then
    return nil
  end

  return colorscheme_data
end

---Save the current theme
---@param theme_name string Name to save theme as
function M.save(theme_name)
  if not theme_name or theme_name == "" then
    notify.error("Theme name cannot be empty")
    return
  end

  -- Security validation of theme name
  if sanitize_theme_name(theme_name) ~= theme_name then
    notify.error("Invalid theme name: theme names can only contain letters, numbers, underscores, and hyphens")
    return
  end

  -- Get current colorscheme data
  local colorscheme_data = {}

  -- Try to get the colorscheme name
  colorscheme_data.name = vim.g.colors_name or "unknown"
  colorscheme_data.description = "Saved theme: " .. theme_name
  colorscheme_data.colors = {}

  -- Extract current highlight colors in the expected format
  local normal_hl = vim.api.nvim_get_hl(0, { name = "Normal" })
  local comment_hl = vim.api.nvim_get_hl(0, { name = "Comment" })
  local string_hl = vim.api.nvim_get_hl(0, { name = "String" })
  local keyword_hl = vim.api.nvim_get_hl(0, { name = "Keyword" })
  local function_hl = vim.api.nvim_get_hl(0, { name = "Function" })
  local constant_hl = vim.api.nvim_get_hl(0, { name = "Constant" })
  local identifier_hl = vim.api.nvim_get_hl(0, { name = "Identifier" })
  local type_hl = vim.api.nvim_get_hl(0, { name = "Type" })
  local diagnostic_error_hl = vim.api.nvim_get_hl(0, { name = "DiagnosticError" })
  local diagnostic_warn_hl = vim.api.nvim_get_hl(0, { name = "DiagnosticWarn" })
  local diagnostic_info_hl = vim.api.nvim_get_hl(0, { name = "DiagnosticInfo" })
  local diagnostic_hint_hl = vim.api.nvim_get_hl(0, { name = "DiagnosticHint" })
  local visual_hl = vim.api.nvim_get_hl(0, { name = "Visual" })
  local cursor_hl = vim.api.nvim_get_hl(0, { name = "Cursor" })

  colorscheme_data.colors = {
    fg = normal_hl.fg and string.format("#%06x", normal_hl.fg) or "#ffffff",
    bg = normal_hl.bg and string.format("#%06x", normal_hl.bg) or "#000000",
    bg_sidebar = normal_hl.bg and string.format("#%06x", normal_hl.bg) or "#000000",
    bg_float = normal_hl.bg and string.format("#%06x", normal_hl.bg) or "#000000",
    bg_statusline = normal_hl.bg and string.format("#%06x", normal_hl.bg) or "#000000",
    red = diagnostic_error_hl.fg and string.format("#%06x", diagnostic_error_hl.fg) or "#ff0000",
    orange = constant_hl.fg and string.format("#%06x", constant_hl.fg) or "#ff8800",
    yellow = diagnostic_warn_hl.fg and string.format("#%06x", diagnostic_warn_hl.fg) or "#ffff00",
    green = string_hl.fg and string.format("#%06x", string_hl.fg) or "#00ff00",
    cyan = diagnostic_info_hl.fg and string.format("#%06x", diagnostic_info_hl.fg) or "#00ffff",
    blue = function_hl.fg and string.format("#%06x", function_hl.fg) or "#0000ff",
    purple = keyword_hl.fg and string.format("#%06x", keyword_hl.fg) or "#ff00ff",
    magenta = constant_hl.fg and string.format("#%06x", constant_hl.fg) or "#ff00ff",
    comment = comment_hl.fg and string.format("#%06x", comment_hl.fg) or "#888888",
    selection = visual_hl.bg and string.format("#%06x", visual_hl.bg) or "#444444",
    cursor = cursor_hl.bg and string.format("#%06x", cursor_hl.bg) or "#ffffff",
  }

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
  if not theme_path then
    notify.error("Failed to create secure theme path")
    return
  end

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
  if not theme_path then
    notify.error("Security error: invalid theme path")
    return
  end

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

  -- Apply the loaded theme using the applier with validation
  local applier = require("hexwitch.theme.applier")
  local success = applier.apply(colorscheme_data)
  if not success then
    notify.error("Failed to apply theme: invalid theme data")
    return
  end

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
  if not theme_path then
    notify.error("Security error: invalid theme path")
    return
  end

  local ok, err = os.remove(theme_path)
  if not ok then
    notify.error("Failed to delete theme: " .. err)
    return
  end

  notify.info("Theme '" .. theme_name .. "' deleted successfully")
end

return M