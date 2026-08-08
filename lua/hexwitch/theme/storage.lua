local notify = require("hexwitch.utils.notify")
local config = require("hexwitch.config")

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

-- Get the directory for storing themes (respects the configured themes_dir)
local function get_theme_dir()
  local config_dir = config.get().themes_dir
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

  -- Persist the actually-applied theme (the real palette), not a lossy
  -- re-derivation from highlight groups.
  local state = require("hexwitch.storage.state")
  local current = state.get_current_theme()
  if not current then
    notify.error("No theme is currently applied. Generate or load a theme first.")
    return
  end

  local colorscheme_data = vim.deepcopy(current)
  colorscheme_data.saved_at = os.date("%Y-%m-%dT%H:%M:%SZ")

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
    -- Skip the plugin's own state file, which shares this directory
    if theme_name ~= "state" then
      table.insert(themes, theme_name)
    end
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