local config = require("hexwitch.config")
local notify = require("hexwitch.utils.notify")
local prompts = require("hexwitch.ai.prompts")

local M = {}

---@class hexwitch.ColorschemeData
---@field name string Theme name
---@field description string Theme description
---@field colors hexwitch.ColorPalette

---@class hexwitch.ColorPalette
---@field bg string Background color
---@field fg string Foreground color
---@field bg_sidebar string
---@field bg_float string
---@field bg_statusline string
---@field red string
---@field orange string
---@field yellow string
---@field green string
---@field cyan string
---@field blue string
---@field purple string
---@field magenta string
---@field comment string
---@field selection string
---@field cursor string

---Get JSON schema for structured output
---@return table
local function get_schema()
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

---Generate colorscheme using OpenAI API
---@param user_input string User's theme description
---@param callback fun(result: hexwitch.ColorschemeData|nil, error: string|nil)
function M.generate(user_input, callback)
  local cfg = config.get()

  if not cfg.openai_api_key or cfg.openai_api_key == "" then
    callback(nil, "OpenAI API key not configured. Set via config or OPENAI_API_KEY env var")
    return
  end

  notify.debug("Generating theme for: " .. user_input)

  local prompt = prompts.build_theme_prompt(user_input)

  local body = vim.json.encode({
    model = cfg.model,
    messages = {
      {
        role = "system",
        content = prompts.SYSTEM_PROMPT,
      },
      {
        role = "user",
        content = prompt,
      },
    },
    response_format = {
      type = "json_schema",
      json_schema = {
        name = "neovim_colorscheme",
        strict = true,
        schema = get_schema(),
      },
    },
    temperature = cfg.temperature,
  })

  -- Check if plenary is available (respect test override)
  if vim.g.hexwitch_test_plenary_unavailable == true then
    callback(nil, "plenary.nvim is required")
    return
  end

  local has_plenary, curl = pcall(require, "plenary.curl")
  if not has_plenary then
    callback(nil, "plenary.nvim is required but not installed")
    return
  end

  notify.debug("Sending request to OpenAI API")

  -- Define response handler function
  local function handle_response(response)
    notify.debug("Received response with status: " .. tostring(response.status))

    if response.status ~= 200 then
      local error_msg = "API request failed with status " .. response.status
      if response.body then
        local ok, parsed = pcall(vim.json.decode, response.body)
        if ok and parsed.error then
          error_msg = error_msg .. ": " .. (parsed.error.message or "Unknown error")
        end
      end
      callback(nil, error_msg)
      return
    end

    local ok, parsed = pcall(vim.json.decode, response.body)
    if not ok then
      callback(nil, "Failed to parse API response: " .. tostring(parsed))
      return
    end

    if not parsed.choices or #parsed.choices == 0 then
      callback(nil, "No choices in API response")
      return
    end

    local content = parsed.choices[1].message.content
    local colorscheme_ok, colorscheme = pcall(vim.json.decode, content)

    if not colorscheme_ok then
      callback(nil, "Failed to parse colorscheme JSON: " .. tostring(colorscheme))
      return
    end

    notify.debug("Successfully generated theme: " .. (colorscheme.name or "unnamed"))
    callback(colorscheme, nil)
  end

  curl.post("https://api.openai.com/v1/chat/completions", {
    headers = {
      ["Content-Type"] = "application/json",
      ["Authorization"] = "Bearer " .. cfg.openai_api_key,
    },
    body = body,
    timeout = cfg.timeout,
    callback = vim.g.hexwitch_test_sync_mode and handle_response or vim.schedule_wrap(handle_response),
  })
end

return M

