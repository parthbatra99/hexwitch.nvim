-- Shared helpers for the hexwitch test suite.
-- Clean, deterministic, no live network. AI calls are made synchronous via
-- the client's `vim.g.hexwitch_test_sync_mode` hook.
local M = {}

---Create a fresh temp directory
---@return string path
function M.temp_dir()
  local d = vim.fn.tempname()
  vim.fn.mkdir(d, "p")
  return d
end

-- A WCAG-friendly palette (deep-green bg, no pure black/white) so accessibility
-- validation stays quiet during tests.
M.valid_theme = {
  name = "forest-moon",
  description = "A forest under a full moon",
  colors = {
    bg = "#0f1a14",
    fg = "#d8e8dc",
    bg_sidebar = "#0f1a14",
    bg_float = "#16241c",
    bg_statusline = "#14201a",
    red = "#e06c75",
    orange = "#d19a66",
    yellow = "#e5c07b",
    green = "#98c379",
    cyan = "#56b6c2",
    blue = "#61afef",
    purple = "#c678dd",
    magenta = "#c678dd",
    comment = "#7d8a80",
    selection = "#1f3a2a",
    cursor = "#d8e8dc",
  },
}

---A second palette, distinct from valid_theme (used to prove refine replaces colors)
M.refined_theme = {
  name = "forest-moon-warm",
  description = "Warmer forest under a full moon",
  colors = {
    bg = "#1a1410",
    fg = "#ede0d2",
    bg_sidebar = "#1a1410",
    bg_float = "#241c16",
    bg_statusline = "#201a14",
    red = "#e06c75",
    orange = "#d19a66",
    yellow = "#e5c07b",
    green = "#98c379",
    cyan = "#56b6c2",
    blue = "#61afef",
    purple = "#c678dd",
    magenta = "#c678dd",
    comment = "#8a7d6f",
    selection = "#3a2a1f",
    cursor = "#ede0d2",
  },
}

---Build an OpenAI chat.completion response whose message content is the theme JSON
---@param theme table
---@return table response {status, body}
function M.completion(theme)
  return {
    status = 200,
    body = vim.json.encode({
      choices = { { message = { content = vim.json.encode(theme) } } },
    }),
  }
end

---Structured-output style response: message.parsed is already a table
---@param theme table
---@return table response
function M.parsed_completion(theme)
  return {
    status = 200,
    body = vim.json.encode({
      choices = { { message = { parsed = theme } } },
    }),
  }
end

---Build an HTTP error response
---@param status number
---@param msg string
---@return table response
function M.error_response(status, msg)
  return {
    status = status,
    body = vim.json.encode({ error = { message = msg or ("HTTP " .. status) } }),
  }
end

---Install a plenary.curl.post mock.
---`responder(url, body_table) -> response_table` controls the reply.
---Returns (captured_requests, restore_fn).
---@param responder? function
---@return table captured, function restore
function M.mock_curl(responder)
  local captured = {}
  local real = package.loaded["plenary.curl"]
  package.loaded["plenary.curl"] = {
    post = function(url, opts)
      local body = vim.json.decode(opts.body)
      table.insert(captured, { url = url, headers = opts.headers, body = body, timeout = opts.timeout })
      local resp = responder and responder(url, body) or {}
      if opts.callback then
        opts.callback(resp)
      end
    end,
  }
  return captured, function() package.loaded["plenary.curl"] = real end
end

---Set up config deterministically (api_key explicit, so no env leakage).
---@param opts? table overrides
---@return table config
function M.setup_config(opts)
  opts = opts or {}
  local config = require("hexwitch.config")
  config.setup(vim.tbl_extend("force", {
    api_key = "test-key",
    base_url = "https://api.openai.com/v1",
    model = "gpt-4o-mini",
    temperature = 0.7,
    themes_dir = opts.themes_dir or M.temp_dir(),
    save_themes = true,
  }, opts))
  return config
end

---Build a copy of the valid palette with one color removed
---@param key string color key to omit
---@return table colors
function M.colors_missing(key)
  local c = {}
  for k, v in pairs(M.valid_theme.colors) do
    if k ~= key then c[k] = v end
  end
  return c
end

-- Themes that are invalid at the COLOR level (what the applier validates).
-- Note: the applier validates colors, not `name`, so a theme missing only its
-- name is intentionally NOT included here.
M.invalid_color_themes = {
  { description = "no colors table at all" },
  { name = "empty", description = "x", colors = {} },
  { name = "missing-bg", description = "x", colors = M.colors_missing("bg") },
  { name = "bad-hex", description = "x", colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = "not-a-color" }) },
  { name = "short-hex", description = "x", colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = "#123" }) },
  { name = "long-hex", description = "x", colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = "#1234567" }) },
}

return M

