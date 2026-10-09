local config = require("hexwitch.config")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Get state file path
local function get_state_file()
  local cfg = config.get()
  local state_dir = cfg.themes_dir
  vim.fn.mkdir(state_dir, "p")
  return state_dir .. "/state.json"
end

-- Default state structure
local default_state = {
  version = "1.0.0",
  session = {
    started_at = "",
    current_theme_id = nil,
    last_command = nil,
  },
  autoprompt = {
    frequency = "manual",
    last_prompted_at = nil,
  },
  history = {
    entries = {},
    total_generated = 0,
    total_saved = 0,
    last_generated_id = nil,
  },
  stats = {
    commands_used = {},
    providers_used = {},
    themes_per_day = {},
    average_generation_time = 0,
  },
}

-- Current state in memory
local current_state = nil

-- Load state from disk
function M.load()
  local state_file = get_state_file()
  local file = io.open(state_file, "r")

  if not file then
    logger.info("storage.state", "load", "No existing state file, creating new state")
    current_state = vim.deepcopy(default_state)
    current_state.session.started_at = os.date("%Y-%m-%dT%H:%M:%SZ")
    M.save()
    return current_state
  end

  local content = file:read("*all")
  file:close()

  local ok, state = pcall(vim.json.decode, content)
  if not ok then
    logger.warn("storage.state", "load", "Failed to parse state file, creating new state",
      { error = tostring(state) })
    current_state = vim.deepcopy(default_state)
    current_state.session.started_at = os.date("%Y-%m-%dT%H:%M:%SZ")
    M.save()
    return current_state
  end

  -- Merge with defaults to handle new fields
  current_state = vim.tbl_deep_extend("force", default_state, state)
  current_state.session.started_at = os.date("%Y-%m-%dT%H:%M:%SZ")

  logger.info("storage.state", "load", "State loaded successfully",
    { history_entries = #current_state.history.entries })

  return current_state
end

-- Save state to disk
function M.save()
  if not current_state then
    logger.warn("storage.state", "save", "No state to save")
    return false
  end

  local state_file = get_state_file()
  local file = io.open(state_file, "w")

  if not file then
    logger.error("storage.state", "save", "Failed to open state file for writing",
      { state_file = state_file })
    return false
  end

  local ok, json = pcall(vim.json.encode, current_state)
  if not ok then
    logger.error("storage.state", "save", "Failed to encode state to JSON",
      { error = tostring(json) })
    file:close()
    return false
  end

  file:write(json)
  file:close()

  logger.debug("storage.state", "save", "State saved successfully")
  return true
end

-- Get current state
function M.get()
  if not current_state then
    return M.load()
  end
  return current_state
end

-- Currently applied theme holder (in-memory). Set by the applier on every
-- apply (generate/refine/load/import), read by save() and refine().
local current_theme = nil

---Set the currently applied theme
---@param theme_data table Colorscheme data
function M.set_current_theme(theme_data)
  if not current_state then
    M.load()
  end
  current_theme = theme_data
  if theme_data and theme_data.name then
    current_state.session.current_theme_id = theme_data.name
  end
end

---Get the currently applied theme
---@return table|nil theme_data
function M.get_current_theme()
  return current_theme
end

-- Add entry to generation history
---@param prompt string User prompt
---@param theme_data table Generated theme data
---@param provider string AI provider used
---@param model string Model used
---@param generation_time number Generation time in ms
---@param user_action string User action ("applied", "saved", "discarded")
function M.add_history_entry(prompt, theme_data, provider, model, generation_time, user_action)
  if not current_state then
    M.load()
  end

  local entry = {
    id = "gen_" .. os.time(),
    prompt = prompt,
    theme = theme_data,
    provider = provider,
    model = model,
    generated_at = os.date("%Y-%m-%dT%H:%M:%SZ"),
    generation_time_ms = generation_time,
    user_action = user_action,
  }

  table.insert(current_state.history.entries, entry)

  -- Limit history size
  local cfg = config.get()
  local max_history = cfg.max_history
  if #current_state.history.entries > max_history then
    table.remove(current_state.history.entries, 1)
  end

  -- Update statistics
  current_state.history.total_generated = current_state.history.total_generated + 1
  current_state.history.last_generated_id = entry.id

  if user_action == "saved" then
    current_state.history.total_saved = current_state.history.total_saved + 1
  end

  -- Update provider stats
  if not current_state.stats.providers_used[provider] then
    current_state.stats.providers_used[provider] = 0
  end
  current_state.stats.providers_used[provider] = current_state.stats.providers_used[provider] + 1

  -- Update average generation time
  local total_time = current_state.stats.average_generation_time * (current_state.history.total_generated - 1) + generation_time
  current_state.stats.average_generation_time = total_time / current_state.history.total_generated

  logger.debug("storage.state", "add_history_entry", "History entry added",
    { entry_id = entry.id, provider = provider, generation_time = generation_time })

  M.save()
  return entry.id
end

-- Get recent history entries
---@param limit number Maximum number of entries to return
---@return table entries Recent history entries
function M.get_recent_history(limit)
  if not current_state then
    M.load()
  end

  limit = limit or 10
  local entries = current_state.history.entries
  local recent = {}

  for i = #entries, math.max(1, #entries - limit + 1), -1 do
    table.insert(recent, entries[i])
  end

  return recent
end

-- Record command usage
---@param command string Command name
function M.record_command_usage(command)
  if not current_state then
    M.load()
  end

  if not current_state.stats.commands_used[command] then
    current_state.stats.commands_used[command] = 0
  end
  current_state.stats.commands_used[command] = current_state.stats.commands_used[command] + 1

  current_state.session.last_command = command

  logger.debug("storage.state", "record_command_usage", "Command usage recorded",
    { command = command, count = current_state.stats.commands_used[command] })

  M.save()
end

-- Get statistics
---@return table stats Current statistics
function M.get_stats()
  if not current_state then
    M.load()
  end

  return vim.deepcopy(current_state.stats)
end

-- Clear history
function M.clear_history()
  if not current_state then
    M.load()
  end

  current_state.history.entries = {}
  current_state.history.total_generated = 0
  current_state.history.total_saved = 0
  current_state.history.last_generated_id = nil

  logger.info("storage.state", "clear_history", "History cleared")

  M.save()
end

-- Update autoprompt configuration (used by status UI)
---@param frequency string one of: manual|daily|weekly|never
---@param last_date string|nil ISO date string (YYYY-MM-DD)
function M.update_autoprompt(frequency, last_date)
  if not current_state then
    M.load()
  end

  current_state.autoprompt.frequency = frequency or current_state.autoprompt.frequency
  if last_date then
    current_state.autoprompt.last_prompted_at = last_date
  end

  logger.info("storage.state", "update_autoprompt", "Autoprompt updated", {
    frequency = current_state.autoprompt.frequency,
    last_prompted_at = current_state.autoprompt.last_prompted_at,
  })

  M.save()
end

return M
