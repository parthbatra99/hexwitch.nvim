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
  undo_stack = {},
  redo_stack = {},
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
    { history_entries = #current_state.history.entries, undo_stack_size = #current_state.undo_stack })

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

-- Add theme to undo stack
---@param theme_data table Theme data
---@param source string Source of theme ("ai_generation", "saved_load", "refinement")
function M.add_to_undo_stack(theme_data, source)
  if not current_state then
    M.load()
  end

  local theme_entry = {
    id = "theme_" .. os.time(),
    theme = theme_data,
    applied_at = os.date("%Y-%m-%dT%H:%M:%SZ"),
    source = source,
  }

  table.insert(current_state.undo_stack, theme_entry)

  -- Limit undo stack size
  local max_undo = 20
  if #current_state.undo_stack > max_undo then
    table.remove(current_state.undo_stack, 1)
  end

  -- Clear redo stack when new action is performed
  current_state.redo_stack = {}

  -- Update current theme
  current_state.session.current_theme_id = theme_entry.id

  logger.debug("storage.state", "add_to_undo_stack", "Theme added to undo stack",
    { theme_id = theme_entry.id, source = source, stack_size = #current_state.undo_stack })

  M.save()
  return theme_entry.id
end

-- Pop from undo stack
---@return table|nil theme_data Theme data if available
function M.undo()
  if not current_state then
    M.load()
  end

  if #current_state.undo_stack <= 1 then
    logger.info("storage.state", "undo", "Nothing to undo")
    return nil
  end

  -- Remove current theme from undo stack and add to redo stack
  local current_theme = table.remove(current_state.undo_stack)
  table.insert(current_state.redo_stack, current_theme)

  -- Get previous theme
  local previous_theme = current_state.undo_stack[#current_state.undo_stack]
  current_state.session.current_theme_id = previous_theme.id

  logger.info("storage.state", "undo", "Undo performed",
    { theme_id = previous_theme.id, undo_stack_size = #current_state.undo_stack })

  M.save()
  return previous_theme.theme
end

-- Redo action
---@return table|nil theme_data Theme data if available
function M.redo()
  if not current_state then
    M.load()
  end

  if #current_state.redo_stack == 0 then
    logger.info("storage.state", "redo", "Nothing to redo")
    return nil
  end

  -- Get theme from redo stack
  local theme_to_redo = table.remove(current_state.redo_stack)
  table.insert(current_state.undo_stack, theme_to_redo)

  current_state.session.current_theme_id = theme_to_redo.id

  logger.info("storage.state", "redo", "Redo performed",
    { theme_id = theme_to_redo.id, redo_stack_size = #current_state.redo_stack })

  M.save()
  return theme_to_redo.theme
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

-- Get undo/redo stack sizes
---@return table sizes Stack sizes
function M.get_stack_sizes()
  if not current_state then
    M.load()
  end

  return {
    undo = #current_state.undo_stack,
    redo = #current_state.redo_stack,
  }
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
