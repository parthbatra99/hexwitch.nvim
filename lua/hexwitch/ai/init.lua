local config = require("hexwitch.config")
local state = require("hexwitch.storage.state")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Lazy-load provider registry
local function get_providers()
  return require("hexwitch.ai.providers")
end

-- Create and configure AI provider
---@return table|nil provider Configured provider instance
---@return string|nil error Error message
local function create_provider()
  local cfg = config.get()
  local providers = get_providers()

  logger.debug("ai.init", "create_provider",
    string.format("Creating provider: %s", cfg.ai_provider))

  local provider, error = providers.create(cfg)
  if provider then
    return provider, nil
  end

  return nil, error or "Failed to create AI provider"
end

-- Generate theme using configured AI provider
---@param user_input string User's theme description
---@param callback function Callback function
function M.generate(user_input, callback)
  if not user_input or user_input == "" then
    local error_msg = "Theme description cannot be empty"
    logger.error("ai.init", "generate", error_msg)
    callback(nil, error_msg)
    return
  end

  logger.info("ai.init", "generate", string.format("Starting theme generation: %s", user_input))

  local provider, error = create_provider()
  if not provider then
    logger.error("ai.init", "generate", string.format("Failed to create AI provider: %s", error))
    callback(nil, error or "Failed to create AI provider")
    return
  end

  -- Record command usage
  state.record_command_usage("generate")

  -- Start timing
  logger.start_timer("theme_generation")

  -- Call provider's generate method
  provider:generate(user_input, function(colorscheme_data, err)
    local elapsed = logger.end_timer("theme_generation", "ai.init", "generate")

    if err then
      logger.error("ai.init", "generate", string.format("Theme generation failed: %s", err),
        { user_input = user_input, elapsed_ms = elapsed })
      callback(nil, err)
      return
    end

    if not colorscheme_data then
      local error_msg = "Provider returned nil colorscheme data"
      logger.error("ai.init", "generate", error_msg, { user_input = user_input })
      callback(nil, error_msg)
      return
    end

    -- Add to history
    local cfg = config.get()
    state.add_history_entry(
      user_input,
      colorscheme_data,
      cfg.ai_provider,
      cfg.model,
      elapsed or 0,
      "applied" -- Assume applied by default
    )

    logger.info("ai.init", "generate",
      string.format("Theme generated successfully: %s", colorscheme_data.name or "unnamed"),
      {
        theme_name = colorscheme_data.name,
        generation_time_ms = elapsed,
        provider = cfg.ai_provider,
        model = cfg.model,
      })

    callback(colorscheme_data, nil)
  end)
end

-- Get available AI providers
---@return table providers List of available providers
function M.get_available_providers()
  local providers = get_providers()
  return providers.list_available()
end

-- Get information about current provider configuration
---@return table info Provider configuration info
function M.get_provider_info()
  local cfg = config.get()
  local providers = get_providers()

  local provider = providers.get(cfg.ai_provider)

  return {
    provider = {
      name = cfg.ai_provider,
      available = provider ~= nil,
      info = provider and provider:get_info() or nil,
    },
    model = cfg.model,
    temperature = cfg.temperature,
    timeout = cfg.timeout,
  }
end

-- Test provider connectivity
---@param provider_name? string Specific provider to test (optional)
---@param callback function Callback with result
function M.test_connectivity(provider_name, callback)
  local cfg = config.get()
  local test_config = provider_name and vim.tbl_deep_extend("force", cfg, {
    ai_provider = provider_name,
  }) or cfg

  logger.info("ai.init", "test_connectivity",
    string.format("Testing connectivity for provider: %s", test_config.ai_provider))

  local provider, error = create_provider()
  if not provider then
    callback(false, error or "Failed to create provider")
    return
  end

  -- Use a simple test prompt
  local test_prompt = "test theme generation"

  provider:generate(test_prompt, function(colorscheme_data, err)
    if err then
      logger.warn("ai.init", "test_connectivity",
        string.format("Connectivity test failed: %s", err))
      callback(false, err)
    else
      logger.info("ai.init", "test_connectivity", "Connectivity test successful")
      callback(true, "Connectivity test successful")
    end
  end)
end

return M