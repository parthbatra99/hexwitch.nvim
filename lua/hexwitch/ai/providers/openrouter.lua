local logger = require("hexwitch.utils.logger")

local M = {}

-- OpenRouter provider configuration and implementation

-- Check if required dependencies are available
function M.is_available()
  local has_plenary, curl = pcall(require, "plenary.curl")
  return has_plenary
end

-- Create new OpenRouter provider instance
---@param config table Configuration
---@return table provider Provider instance
function M.new(config)
  local self = setmetatable({}, { __index = M })
  self.config = config
  self.api_key = config.api_key or ""
  self.model = config.model or "anthropic/claude-3.5-sonnet"
  self.timeout = config.timeout or 30000
  self.temperature = config.temperature or 0.7

  logger.debug("ai.providers.openrouter", "new",
    string.format("Created OpenRouter provider with model: %s", self.model))

  return self
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
  return string.format([[Generate a complete Neovim colorscheme based on this description: "%s"

Requirements:
1. **Base Colors**: Provide bg (background) and fg (foreground) with excellent contrast
2. **UI Elements**: Define bg_sidebar, bg_float, and bg_statusline for consistent UI
3. **Semantic Colors**:
   - red: errors, deletions, warnings
   - green: success, additions, strings
   - yellow: types, warnings, constants
   - blue: functions, information
   - cyan: operators, special characters
   - purple: keywords, control flow
   - magenta: special identifiers
   - orange: numbers, constants
4. **Accents**: comment (muted), selection (highlight), cursor (standout)

Ensure the palette is cohesive, accessible, and captures the essence of: "%s"

Return ONLY valid JSON with this exact structure:
{
  "name": "descriptive_theme_name",
  "description": "brief theme description",
  "colors": {
    "bg": "#hexcode",
    "fg": "#hexcode",
    "bg_sidebar": "#hexcode",
    "bg_float": "#hexcode",
    "bg_statusline": "#hexcode",
    "red": "#hexcode",
    "orange": "#hexcode",
    "yellow": "#hexcode",
    "green": "#hexcode",
    "cyan": "#hexcode",
    "blue": "#hexcode",
    "purple": "#hexcode",
    "magenta": "#hexcode",
    "comment": "#hexcode",
    "selection": "#hexcode",
    "cursor": "#hexcode"
  }
}

No explanations, just JSON.]], user_input, user_input)
end

-- Generate theme using OpenRouter API
---@param user_input string User's theme description
---@param callback function Callback function
function M:generate(user_input, callback)
  if not self.api_key or self.api_key == "" then
    local error_msg = "OpenRouter API key not configured. Set OPENROUTER_API_KEY env var or api_key in config"
    logger.error("ai.providers.openrouter", "generate", error_msg)
    callback(nil, error_msg)
    return
  end

  logger.info("ai.providers.openrouter", "generate",
    string.format("Generating theme with OpenRouter: %s", user_input),
    { model = self.model, temperature = self.temperature })

  local curl = require("plenary.curl")
  local prompt = self:build_prompt(user_input)

  local body = vim.json.encode({
    model = self.model,
    messages = {
      {
        role = "system",
        content = "You are an expert Neovim colorscheme designer specializing in creating beautiful, accessible, and harmonious color palettes. Always ensure colors are in hex format (#RRGGBB) and provide proper contrast between background and foreground.",
      },
      {
        role = "user",
        content = prompt,
      },
    },
    temperature = self.temperature,
    max_tokens = 1024,
  })

  logger.start_timer("openrouter_api_request")

  curl.post("https://openrouter.ai/api/v1/chat/completions", {
    headers = {
      ["Content-Type"] = "application/json",
      ["Authorization"] = "Bearer " .. self.api_key,
      ["HTTP-Referer"] = "https://github.com/hexwitch/hexwitch.nvim",
      ["X-Title"] = "hexwitch.nvim",
    },
    body = body,
    timeout = self.timeout,
    callback = vim.schedule_wrap(function(response)
      local elapsed = logger.end_timer("openrouter_api_request", "ai.providers.openrouter", "generate")

      logger.debug("ai.providers.openrouter", "generate",
        string.format("Received response in %.2fms with status: %s", elapsed or 0, response.status))

      if response.status ~= 200 then
        local error_msg = "API request failed with status " .. response.status
        if response.body then
          local ok, parsed = pcall(vim.json.decode, response.body)
          if ok and parsed.error then
            error_msg = error_msg .. ": " .. (parsed.error.message or "Unknown error")
          end
        end
        logger.error("ai.providers.openrouter", "generate", error_msg, {
          status = response.status,
          body = response.body and response.body:sub(1, 200) or nil,
        })
        callback(nil, error_msg)
        return
      end

      local ok, parsed = pcall(vim.json.decode, response.body)
      if not ok then
        local error_msg = "Failed to parse API response: " .. tostring(parsed)
        logger.error("ai.providers.openrouter", "generate", error_msg, { response_body = response.body and response.body:sub(1, 200) or nil })
        callback(nil, error_msg)
        return
      end

      if not parsed.choices or #parsed.choices == 0 then
        local error_msg = "No choices in API response"
        logger.error("ai.providers.openrouter", "generate", error_msg, { response = parsed })
        callback(nil, error_msg)
        return
      end

      local content = parsed.choices[1].message.content
      local colorscheme_ok, colorscheme = pcall(vim.json.decode, content)

      if not colorscheme_ok then
        local error_msg = "Failed to parse colorscheme JSON: " .. tostring(colorscheme)
        logger.error("ai.providers.openrouter", "generate", error_msg, {
          content = content and content:sub(1, 200) or nil,
        })
        callback(nil, error_msg)
        return
      end

      logger.info("ai.providers.openrouter", "generate",
        string.format("Successfully generated theme: %s", colorscheme.name or "unnamed"),
        { theme_name = colorscheme.name, provider = "openrouter" })

      callback(colorscheme, nil)
    end),
  })
end

-- Get provider information
---@return table info Provider information
function M:get_info()
  return {
    name = "OpenRouter",
    description = "Multi-model AI provider with access to various models",
    website = "https://openrouter.ai",
    models = {
      "anthropic/claude-3.5-sonnet",
      "anthropic/claude-3-haiku",
      "openai/gpt-4o",
      "openai/gpt-4o-mini",
      "google/gemini-pro",
    },
    features = {
      "json_schema_validation",
      "multiple_models",
      "fallback_support",
    },
  }
end

return M