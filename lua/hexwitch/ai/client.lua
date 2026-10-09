local logger = require("hexwitch.utils.logger")
local notify = require("hexwitch.utils.notify")
local prompts = require("hexwitch.ai.prompts")
local config = require("hexwitch.config")

local M = {}

-- Locked request defaults (not user-configurable — kept simple/opinionated).
local REQUEST_TEMPERATURE = 1.0 -- reasoning models want temperature = 1
local REQUEST_TIMEOUT = 30000 -- ms

-- Sanitize sensitive data for logging
local function sanitize_log_data(data)
  if not data then return nil end

  local sanitized = vim.deepcopy(data)

  if sanitized.api_key then
    sanitized.api_key = "***REDACTED***"
  end

  if sanitized.headers and sanitized.headers.Authorization then
    sanitized.headers.Authorization = "Bearer ***REDACTED***"
  end

  local sensitive_fields = { "key", "token", "secret", "password", "credential" }
  for _, field in ipairs(sensitive_fields) do
    if sanitized[field] then
      sanitized[field] = "***REDACTED***"
    end
  end

  return sanitized
end

-- Check if the HTTP dependency (plenary.curl) is available
function M.is_available()
  if vim.g.hexwitch_test_plenary_unavailable then
    return false
  end
  return pcall(require, "plenary.curl")
end

-- Derive a short provider label from the configured base_url host
local function provider_label(base_url)
  return (base_url:match("^%w+://([^/]+)")) or "openai-compat"
end

-- Best-effort JSON extraction from a model response that did not return a
-- parsed object (e.g. endpoints without structured-output support).
local function extract_json(text)
  if type(text) ~= "string" then
    return nil
  end

  local ok, decoded = pcall(vim.json.decode, text)
  if ok then
    return decoded
  end

  local block = text:match("```json%s*(.-)```") or text:match("```%s*(.-)```")
  if block then
    ok, decoded = pcall(vim.json.decode, block)
    if ok then
      return decoded
    end
  end

  local sub = text:match("(%b{})")
  if sub then
    ok, decoded = pcall(vim.json.decode, sub)
    if ok then
      return decoded
    end
  end

  return nil
end

-- Parse the chat completion message into a colorscheme table
local function parse_message(message)
  if type(message) ~= "table" then
    return nil, "empty or invalid message"
  end

  if message.parsed and type(message.parsed) == "table" then
    return message.parsed
  end

  if type(message.content) == "string" then
    local ok, decoded = pcall(vim.json.decode, message.content)
    if ok then
      return decoded
    end
    local extracted = extract_json(message.content)
    if extracted then
      return extracted
    end
    return nil, "unable to parse JSON from content"
  end

  if type(message.content) == "table" then
    for _, item in ipairs(message.content) do
      if type(item) == "table" then
        local text = item.text or item.value or item.content
        if type(text) == "string" then
          local extracted = extract_json(text)
          if extracted then
            return extracted
          end
        end
      end
    end
  end

  return nil, "unable to parse structured output"
end

-- Validate and stamp metadata onto raw colorscheme data
local function process_colorscheme_data(colorscheme_data)
  if not colorscheme_data or type(colorscheme_data) ~= "table" then
    return nil
  end

  if not colorscheme_data.name or not colorscheme_data.colors or type(colorscheme_data.colors) ~= "table" then
    return nil
  end

  local cfg = config.get()
  colorscheme_data.provider = provider_label(cfg.base_url)
  colorscheme_data.model = cfg.model
  colorscheme_data.generated_at = os.date("%Y-%m-%d %H:%M:%S")

  return colorscheme_data
end

-- Handle the HTTP response
local function handle_response(response, callback)
  if response.status ~= 200 then
    local error_msg = string.format("API error: %d - %s", response.status, response.body or "Unknown error")
    logger.error("ai.client", "response", error_msg,
      sanitize_log_data({ status = response.status, body = response.body }))
    callback(nil, error_msg)
    return
  end

  local success, data = pcall(vim.json.decode, response.body)
  if not success then
    local error_msg = "Failed to parse API response"
    logger.error("ai.client", "response", error_msg,
      sanitize_log_data({ parse_error = data, response_body = response.body }))
    callback(nil, error_msg)
    return
  end

  if type(data) ~= "table" then
    local error_msg = "Invalid response format from API"
    logger.error("ai.client", "response", error_msg,
      sanitize_log_data({ response_body = response.body }))
    callback(nil, error_msg)
    return
  end

  local choice = data.choices and data.choices[1]
  if type(choice) ~= "table" or type(choice.message) ~= "table" then
    local error_msg = "Invalid response format from API"
    logger.error("ai.client", "response", error_msg, sanitize_log_data({ response_data = data }))
    callback(nil, error_msg)
    return
  end

  local colorscheme_data, parse_err = parse_message(choice.message)
  if not colorscheme_data then
    local error_msg = "Failed to parse theme data from response: " .. tostring(parse_err)
    logger.error("ai.client", "response", error_msg,
      sanitize_log_data({ parse_error = parse_err, message = data.choices[1].message }))
    callback(nil, error_msg)
    return
  end

  local processed = process_colorscheme_data(colorscheme_data)
  if not processed then
    callback(nil, "Invalid colorscheme data format")
    return
  end

  local valid, warnings = prompts.validate_accessibility(processed)
  if not valid then
    notify.warn("Generated theme has accessibility issues:")
    for _, warning in ipairs(warnings) do
      notify.warn("  - " .. warning)
    end
    notify.info("Theme applied anyway - consider refining")
  end

  logger.info("ai.client", "response",
    string.format("Successfully generated theme: %s", processed.name or "unnamed"),
    { theme_name = processed.name, accessibility_warnings = warnings })

  callback(processed, nil)
end

-- Shared POST for any messages payload
---@param messages table Chat messages
---@param callback function Callback(data, err)
local function request(messages, callback)
  local cfg = config.get()

  if not cfg.api_key or cfg.api_key == "" then
    local error_msg = "API key not configured (set `api_key` or $OPENAI_API_KEY)"
    logger.error("ai.client", "request", error_msg)
    callback(nil, error_msg)
    return
  end

  if not M.is_available() then
    local error_msg = "plenary.curl not available (install plenary.nvim)"
    logger.error("ai.client", "request", error_msg)
    callback(nil, error_msg)
    return
  end

  local curl = require("plenary.curl")
  local url = cfg.base_url:gsub("/+$", "") .. "/chat/completions"

  local body = {
    model = cfg.model,
    messages = messages,
    temperature = REQUEST_TEMPERATURE,
    response_format = {
      type = "json_schema",
      json_schema = {
        name = prompts.SCHEMA_NAME,
        schema = prompts.SCHEMA,
        strict = true,
      },
    },
  }
  if cfg.max_tokens then
    body.max_tokens = cfg.max_tokens
  end

  logger.debug("ai.client", "request",
    string.format("POST %s model=%s", url, cfg.model),
    sanitize_log_data({ base_url = cfg.base_url, model = cfg.model }))

  curl.post(url, {
    headers = {
      ["Content-Type"] = "application/json",
      ["Authorization"] = "Bearer " .. cfg.api_key,
    },
    body = vim.json.encode(body),
    timeout = REQUEST_TIMEOUT,
    callback = function(response)
      -- In test sync mode run the handler inline so busted can assert synchronously
      if vim.g.hexwitch_test_sync_mode then
        handle_response(response, callback)
      else
        vim.schedule(function()
          handle_response(response, callback)
        end)
      end
    end,
  })
end

-- Generate a brand-new theme from a description
---@param user_input string Theme description
---@param callback function Callback(data, err)
function M.generate(user_input, callback)
  request({
    { role = "system", content = prompts.SYSTEM_PROMPT },
    { role = "user", content = prompts.build_theme_prompt(user_input) },
  }, callback)
end

-- Refine the current theme per a natural-language change request
---@param current_theme table Current colorscheme data
---@param change string Refinement request
---@param callback function Callback(data, err)
function M.refine(current_theme, change, callback)
  if not current_theme then
    callback(nil, "No current theme to refine")
    return
  end
  request({
    { role = "system", content = prompts.SYSTEM_PROMPT },
    { role = "user", content = prompts.build_refinement_prompt(current_theme, change) },
  }, callback)
end

return M
