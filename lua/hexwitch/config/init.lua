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
    ai_provider = { cfg.ai_provider, "string" },
    api_key = { cfg.api_key, "string" },
    model = { cfg.model, "string" },
    temperature = { cfg.temperature, "number" },
    timeout = { cfg.timeout, "number" },
    save_themes = { cfg.save_themes, "boolean" },
    themes_dir = { cfg.themes_dir, "string" },
    max_history = { cfg.max_history, "number" },
    auto_save_history = { cfg.auto_save_history, "boolean" },
    contrast_threshold = { cfg.contrast_threshold, "number" },
    debug = { cfg.debug, "boolean" },
    ui = { cfg.ui, "table" },
  })

  if not ok then
    return false, err
  end

  -- Additional custom validations
  if cfg.temperature < 0 or cfg.temperature > 2 then
    return false, "temperature must be between 0 and 2"
  end

  if not vim.tbl_contains({ "openai", "openrouter", "custom" }, cfg.ai_provider) then
    return false, "ai_provider must be 'openai', 'openrouter', or 'custom'"
  end

  if cfg.max_history < 1 then
    return false, "max_history must be at least 1"
  end

  if cfg.contrast_threshold < 1 or cfg.contrast_threshold > 21 then
    return false, "contrast_threshold must be between 1 and 21"
  end

  if cfg.timeout < 1000 then
    return false, "timeout must be at least 1000ms"
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

  -- Handle backward compatibility: if deprecated openai_api_key is provided, use it
  if config_table.openai_api_key and not config_table.api_key then
    config_table.api_key = config_table.openai_api_key
    config_table.ai_provider = config_table.ai_provider or "openai"
  end

  -- Set provider-specific API key defaults
  if not config_table.api_key then
    if config_table.ai_provider == "openai" then
      config_table.api_key = vim.env.OPENAI_API_KEY or ""
    elseif config_table.ai_provider == "openrouter" then
      config_table.api_key = vim.env.OPENROUTER_API_KEY or ""
    end
  end

  -- Merge with defaults into a candidate config
  local new_config = vim.tbl_deep_extend("force", schema.defaults, config_table)

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

