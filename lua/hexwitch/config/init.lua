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
    base_url = { cfg.base_url, "string" },
    api_key = { cfg.api_key, "string" },
    model = { cfg.model, "string" },
    save_themes = { cfg.save_themes, "boolean" },
    themes_dir = { cfg.themes_dir, "string" },
    max_history = { cfg.max_history, "number" },
    auto_save_history = { cfg.auto_save_history, "boolean" },
    debug = { cfg.debug, "boolean" },
    ui = { cfg.ui, "table" },
  })

  if not ok then
    return false, err
  end

  if cfg.max_tokens ~= nil and type(cfg.max_tokens) ~= "number" then
    return false, "max_tokens must be a number"
  end

  if not cfg.base_url:match("^https?://") then
    return false, "base_url must be a valid http(s) URL"
  end

  if cfg.max_history < 1 then
    return false, "max_history must be at least 1"
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

  -- Resolve API key from environment only when neither the caller nor the
  -- current config already provides one.
  if config_table.api_key == nil and (M.current.api_key == nil or M.current.api_key == "") then
    config_table.api_key = vim.env.OPENAI_API_KEY or vim.env.HEXWITCH_API_KEY or ""
  end

  -- Merge over the current config so repeated setup() calls are incremental
  local new_config = vim.tbl_deep_extend("force", M.current, config_table)

  -- Validate candidate config; only persist if valid
  local valid, err = validate(new_config)
  if not valid then
    return false, "Configuration error: " .. (err or "unknown error")
  end

  -- Persist validated config
  M.current = new_config

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
