-- hexwitch config: defaults, merge, validation.
--
-- The plugin is a thin client over the `hexwitch` engine binary. All
-- color work, provider calls, and validation live in the engine; this
-- file only configures how the binary is found and how results surface.

local M = {}

M.defaults = {
  -- Engine binary. Must be on PATH unless an absolute path is given.
  cmd = "hexwitch",
  -- Optional provider overrides passed to `hexwitch interpret`.
  provider = nil, -- "openai" | "openrouter"
  model = nil, -- e.g. "gpt-4o-mini", "anthropic/claude-3.5-sonnet"
  -- Review candidates before applying by default. Use :HexwitchApply to
  -- accept or :HexwitchDiscard to reject. Opt into auto-apply if desired;
  -- refused or failed edits still require explicit review.
  auto_apply = false,
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
  if type(cfg.cmd) ~= "string" or cfg.cmd == "" then
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
