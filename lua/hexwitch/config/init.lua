local schema = require("hexwitch.config.schema")
local validate_utils = require("hexwitch.utils.validate")

local M = {}

---@type hexwitch.InternalConfig
M.current = vim.deepcopy(schema.defaults)

---Validate configuration
---@param cfg hexwitch.InternalConfig
---@return boolean is_valid
---@return string|nil error_message
local function validate(cfg)
  local ok, err = validate_utils.validate_path("vim.g.hexwitch", {
    openai_api_key = { cfg.openai_api_key, "string" },
    model = { cfg.model, "string" },
    temperature = { cfg.temperature, "number" },
    ui_mode = { cfg.ui_mode, "string" },
    save_themes = { cfg.save_themes, "boolean" },
    themes_dir = { cfg.themes_dir, "string" },
    timeout = { cfg.timeout, "number" },
    debug = { cfg.debug, "boolean" },
  })

  if not ok then
    return false, err
  end

  -- Additional custom validations
  if cfg.temperature < 0 or cfg.temperature > 2 then
    return false, "temperature must be between 0 and 2"
  end

  if cfg.ui_mode ~= "input" and cfg.ui_mode ~= "telescope" then
    return false, "ui_mode must be 'input' or 'telescope'"
  end

  return true, nil
end

---Apply user configuration
---@param user_config? hexwitch.UserConfig|fun():hexwitch.UserConfig
---@return boolean success
---@return string|nil error_message
function M.setup(user_config)
  -- Support both table and function
  local config_table = type(user_config) == "function" and user_config() or user_config or {}

  -- Merge with defaults
  M.current = vim.tbl_deep_extend("force", schema.defaults, config_table)

  -- Validate
  local valid, err = validate(M.current)
  if not valid then
    return false, "Configuration error: " .. (err or "unknown error")
  end

  -- Create themes directory if saving is enabled
  if M.current.save_themes then
    vim.fn.mkdir(M.current.themes_dir, "p")
  end

  return true, nil
end

---Get current configuration
---@return hexwitch.InternalConfig
function M.get()
  return M.current
end

return M

