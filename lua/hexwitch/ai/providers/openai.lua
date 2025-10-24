local logger = require("hexwitch.utils.logger")
local notify = require("hexwitch.utils.notify")
local prompts = require("hexwitch.ai.prompts")

local M = {}

-- Sanitize sensitive data for logging
local function sanitize_log_data(data)
  if not data then return nil end

  local sanitized = vim.deepcopy(data)

  -- Redact API keys
  if sanitized.api_key then
    sanitized.api_key = "***REDACTED***"
  end

  -- Redact authorization headers
  if sanitized.headers and sanitized.headers.Authorization then
    sanitized.headers.Authorization = "Bearer ***REDACTED***"
  end

  -- Redact any field that might contain API keys
  local sensitive_fields = {"key", "token", "secret", "password", "credential"}
  for _, field in ipairs(sensitive_fields) do
    if sanitized[field] then
      sanitized[field] = "***REDACTED***"
    end
  end

  return sanitized
end

-- Check if required dependencies are available
function M.is_available()
  local has_plenary, curl = pcall(require, "plenary.curl")
  return has_plenary
end

-- Create new OpenAI provider instance
---@param cfg table Configuration
---@return table provider Provider instance
function M.new(cfg)
  local self = setmetatable({}, { __index = M })
  self.config = cfg
  self.api_key = cfg.api_key or cfg.openai_api_key or ""
  self.model = cfg.model or "gpt-4o-mini"
  self.timeout = cfg.timeout or 30000
  self.temperature = cfg.temperature or 0.7

  logger.debug("ai.providers.openai", "new",
    string.format("Created OpenAI provider with model: %s", self.model))

  return self
end

-- Get provider information
---@return table info Provider information
function M:get_info()
  return {
    name = "OpenAI",
    description = "OpenAI GPT models for theme generation",
    website = "https://openai.com",
    models = {
      "gpt-4o",
      "gpt-4o-mini",
      "gpt-4",
      "gpt-3.5-turbo",
    },
    features = {
      "json_schema_validation",
      "structured_outputs",
      "multiple_models",
    },
  }
end

-- Get JSON schema for structured output
---@return table schema
function M:get_schema()
  return {
    type = "object",
    properties = {
      name = { type = "string", description = "Short theme name" },
      description = { type = "string", description = "Theme description" },
      colors = {
        type = "object",
        properties = {
          bg = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          fg = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          bg_sidebar = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          bg_float = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          bg_statusline = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          red = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          orange = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          yellow = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          green = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          cyan = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          blue = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          purple = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          magenta = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          comment = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          selection = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
          cursor = { type = "string", pattern = "^#[0-9A-Fa-f]{6}$" },
        },
        required = { "bg", "fg", "bg_sidebar", "bg_float", "bg_statusline", "red", "orange", "yellow", "green", "cyan", "blue", "purple", "magenta", "comment", "selection", "cursor" },
        additionalProperties = false,
      },
    },
    required = { "name", "description", "colors" },
    additionalProperties = false,
  }
end

-- Build theme generation prompt
---@param user_input string User's theme description
---@return string prompt
function M:build_prompt(user_input)
  return prompts.build_theme_prompt(user_input)
end

-- Generate theme using OpenAI API
---@param user_input string User's theme description
---@param callback function Callback function
function M:generate(user_input, callback)
  if not self.api_key or self.api_key == "" then
    local error_msg = "OpenAI API key not configured"
    logger.error("ai.providers.openai", "generate", error_msg)
    callback(nil, error_msg)
    return
  end

  local curl = require("plenary.curl")
  local system_prompt = prompts.SYSTEM_PROMPT
  local user_prompt = self:build_prompt(user_input)
  local schema = self:get_schema()

  logger.debug("ai.providers.openai", "generate",
    string.format("Sending request to OpenAI API: model=%s, temp=%.1f", self.model, self.temperature),
    sanitize_log_data({ provider = "openai", model = self.model }))

  curl.post("https://api.openai.com/v1/chat/completions", {
    headers = {
      ["Content-Type"] = "application/json",
      ["Authorization"] = "Bearer " .. self.api_key,
    },
    body = vim.json.encode({
      model = self.model,
      messages = {
        { role = "system", content = system_prompt },
        { role = "user", content = user_prompt },
      },
      temperature = self.temperature,
      response_format = {
        type = "json_schema",
        json_schema = {
          name = "HexwitchThemeSchema",
          schema = schema,
          strict = true,
        },
      },
    }),
    timeout = self.timeout,
    callback = vim.schedule_wrap(function(response)
      self:_handle_response(response, callback, user_input)
    end),
  })
end

-- Handle API response
---@param response table HTTP response
---@param callback function Callback function
---@param user_input string Original user input
function M:_handle_response(response, callback, user_input)
  if response.status ~= 200 then
    local error_msg = string.format("OpenAI API error: %d - %s", response.status, response.body or "Unknown error")
    logger.error("ai.providers.openai", "_handle_response", error_msg,
      sanitize_log_data({ status = response.status, body = response.body }))
    callback(nil, error_msg)
    return
  end

  local success, data = pcall(vim.json.decode, response.body)
  if not success then
    local error_msg = "Failed to parse OpenAI API response"
    logger.error("ai.providers.openai", "_handle_response", error_msg,
      sanitize_log_data({ parse_error = data, response_body = response.body }))
    callback(nil, error_msg)
    return
  end

  if not data.choices or #data.choices == 0 or not data.choices[1].message or not data.choices[1].message.content then
    local error_msg = "Invalid response format from OpenAI API"
    logger.error("ai.providers.openai", "_handle_response", error_msg,
      sanitize_log_data({ response_data = data }))
    callback(nil, error_msg)
    return
  end

  local message = data.choices[1].message

  local function parse_message_payload(msg)
    if msg.parsed and type(msg.parsed) == "table" then
      return msg.parsed
    end

    if type(msg.content) == "string" then
      local ok, decoded = pcall(vim.json.decode, msg.content)
      if ok then
        return decoded
      end
      return nil, decoded
    end

    if type(msg.content) == "table" then
      for _, item in ipairs(msg.content) do
        if type(item) == "table" then
          local text = item.text or item.value or item.content
          if type(text) == "string" then
            local ok, decoded = pcall(vim.json.decode, text)
            if ok then
              return decoded
            end
          end
        end
      end
    end

    return nil, "Unable to parse structured output"
  end

  local colorscheme_data, parse_err = parse_message_payload(message)
  if not colorscheme_data then
    local error_msg = "Failed to parse theme data from OpenAI response"
    logger.error("ai.providers.openai", "_handle_response", error_msg,
      sanitize_log_data({ parse_error = parse_err, message = message }))
    callback(nil, error_msg)
    return
  end

  -- Validate and process the colorscheme data
  local processed_data = self:_process_colorscheme_data(colorscheme_data)
  if not processed_data then
    callback(nil, "Invalid colorscheme data format")
    return
  end

  local valid, warnings = prompts.validate_accessibility(processed_data)
  if not valid then
    notify.warn("Generated theme has accessibility issues:")
    for _, warning in ipairs(warnings) do
      notify.warn("  • " .. warning)
    end
    notify.info("Theme applied anyway - consider regenerating")
  end

  logger.info("ai.providers.openai", "_handle_response",
    string.format("Successfully generated theme: %s", processed_data.name or "unnamed"),
    { theme_name = processed_data.name, user_input = user_input, accessibility_warnings = warnings })

  callback(processed_data, nil)
end

-- Process and validate colorscheme data
---@param colorscheme_data table Raw colorscheme data
---@return table|nil processed_data Processed colorscheme data
function M:_process_colorscheme_data(colorscheme_data)
  -- Basic validation
  if not colorscheme_data or type(colorscheme_data) ~= "table" then
    return nil
  end

  -- Ensure required fields exist
  if not colorscheme_data.name or not colorscheme_data.colors or type(colorscheme_data.colors) ~= "table" then
    return nil
  end

  -- Add metadata
  colorscheme_data.provider = "openai"
  colorscheme_data.model = self.model
  colorscheme_data.generated_at = os.date("%Y-%m-%d %H:%M:%S")

  return colorscheme_data
end

return M