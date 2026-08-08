local config = require("hexwitch.config")
local state = require("hexwitch.storage.state")
local logger = require("hexwitch.utils.logger")
local client = require("hexwitch.ai.client")

local M = {}

-- Derive the provider label recorded in history from the configured base_url
local function provider_label()
  local base_url = config.get().base_url
  return (base_url:match("^%w+://([^/]+)")) or "openai-compat"
end

-- Generate a theme from a description and hand the result to `callback`.
---@param user_input string Theme description
---@param callback function Callback(data, err)
function M.generate(user_input, callback)
  if not user_input or user_input == "" then
    local error_msg = "Theme description cannot be empty"
    logger.error("ai.init", "generate", error_msg)
    callback(nil, error_msg)
    return
  end

  logger.info("ai.init", "generate", string.format("Starting theme generation: %s", user_input))

  state.record_command_usage("generate")
  logger.start_timer("theme_generation")

  client.generate(user_input, function(colorscheme_data, err)
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

    local cfg = config.get()
    state.add_history_entry(
      user_input,
      colorscheme_data,
      provider_label(),
      cfg.model,
      elapsed or 0,
      "applied"
    )

    logger.info("ai.init", "generate",
      string.format("Theme generated successfully: %s", colorscheme_data.name or "unnamed"),
      { theme_name = colorscheme_data.name, generation_time_ms = elapsed, model = cfg.model })

    callback(colorscheme_data, nil)
  end)
end

-- Refine the currently applied theme per a natural-language change request.
---@param change string Refinement request
---@param callback function Callback(data, err)
function M.refine(change, callback)
  if not change or change == "" then
    callback(nil, "Refinement description cannot be empty")
    return
  end

  local current_theme = state.get_current_theme()
  if not current_theme then
    callback(nil, "No theme to refine. Generate one first.")
    return
  end

  logger.info("ai.init", "refine", string.format("Refining theme: %s", change))

  state.record_command_usage("refine")
  logger.start_timer("theme_refinement")

  client.refine(current_theme, change, function(colorscheme_data, err)
    local elapsed = logger.end_timer("theme_refinement", "ai.init", "refine")

    if err then
      logger.error("ai.init", "refine", string.format("Theme refinement failed: %s", err),
        { change = change, elapsed_ms = elapsed })
      callback(nil, err)
      return
    end

    if not colorscheme_data then
      callback(nil, "Provider returned nil colorscheme data")
      return
    end

    local cfg = config.get()
    state.add_history_entry(
      "refine: " .. change,
      colorscheme_data,
      provider_label(),
      cfg.model,
      elapsed or 0,
      "applied"
    )

    logger.info("ai.init", "refine",
      string.format("Theme refined successfully: %s", colorscheme_data.name or "unnamed"),
      { change = change, refinement_time_ms = elapsed, model = cfg.model })

    callback(colorscheme_data, nil)
  end)
end

-- Flat, honest info about the configured AI client (no registry, no fallback)
---@return table info
function M.get_provider_info()
  local cfg = config.get()
  return {
    base_url = cfg.base_url,
    model = cfg.model,
    temperature = cfg.temperature,
    timeout = cfg.timeout,
    available = client.is_available() and cfg.api_key ~= "",
  }
end

return M
