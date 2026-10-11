-- hexwitch config: defaults, merge, validation.
--
-- The plugin is a thin client over the `hexwitch` engine binary. All
-- color work, provider calls, and validation live in the engine; this
-- file only configures how the binary is found and how results surface.

local M = {}

M.defaults = {
  -- nil discovers PATH, then this checkout's release/debug build.
  -- An explicit path is respected and never silently replaced.
  cmd = nil,
  -- One OpenAI-compatible Chat Completions API root; appends /chat/completions.
  base_url = nil, -- defaults to OPENAI_BASE_URL or https://api.openai.com/v1
  api_key_env = nil, -- defaults to OPENAI_API_KEY; name only, never the key
  response_format = "json_schema", -- "json_object" | "none" for compatible APIs without schemas
  provider = nil, -- legacy convenience alias; prefer base_url
  model = nil, -- exact model identifier for your selected API
  -- Review candidates before applying by default. Use :HexwitchApply to
  -- accept or :HexwitchDiscard to reject. Opt into auto-apply if desired;
  -- refused or failed edits still require explicit review.
  auto_apply = false,
  interpretation = "legacy", -- "v1" for property-aware :Hexwitch requests
  -- Notification level: "short" (one line per step) or "full" (report
  -- summaries with every moved role).
  report = "short",
}

M.config = nil

function M.setup(user)
  user = user or {}
  if type(user) ~= "table" then
    error("hexwitch: setup() expects a table")
  end
  local cfg = vim.tbl_deep_extend("force", M.defaults, user)
  if cfg.cmd ~= nil and (type(cfg.cmd) ~= "string" or cfg.cmd == "") then
    error("hexwitch: cmd must be a non-empty executable path or name")
  end
  if type(cfg.auto_apply) ~= "boolean" then
    error("hexwitch: auto_apply must be a boolean")
  end
  if cfg.model ~= nil and (type(cfg.model) ~= "string" or cfg.model == "") then
    error("hexwitch: model must be a non-empty string")
  end
  if cfg.provider ~= nil and cfg.provider ~= "openai" and cfg.provider ~= "openrouter" then
    error("hexwitch: provider must be 'openai' or 'openrouter'")
  end
  if cfg.base_url ~= nil and (type(cfg.base_url) ~= "string" or not cfg.base_url:match("^https?://")
      or cfg.base_url:find("[%s?#]") or cfg.base_url:match("^https?://[^/]*@")) then
    error("hexwitch: base_url must be an HTTP(S) API root without credentials, query or fragment")
  end
  if cfg.api_key_env ~= nil and (type(cfg.api_key_env) ~= "string" or not cfg.api_key_env:match("^[%a_][%w_]*$")) then
    error("hexwitch: api_key_env must name an environment variable, not contain the API key")
  end
  if cfg.response_format ~= "json_schema" and cfg.response_format ~= "json_object" and cfg.response_format ~= "none" then
    error("hexwitch: response_format must be json_schema, json_object or none")
  end
  if cfg.base_url and cfg.base_url:gsub("/+$", ""):match("/chat/completions$") then
    error("hexwitch: base_url is the API root, not /chat/completions")
  end
  if cfg.interpretation ~= "legacy" and cfg.interpretation ~= "v1" then
    error("hexwitch: interpretation must be legacy or v1")
  end
  if cfg.report ~= "short" and cfg.report ~= "full" then
    error("hexwitch: report must be 'short' or 'full'")
  end
  M.config = cfg
  return cfg
end

function M.get()
  if not M.config then
    M.setup()
  end
  return M.config
end

return M
