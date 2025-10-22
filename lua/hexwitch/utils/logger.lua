local config = require("hexwitch.config")

local M = {}

-- Log levels
M.LOG_LEVELS = {
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
}

-- Session ID for tracking
M.session_id = vim.fn.strftime("%Y%m%d_%H%M%S") .. "_" .. math.random(1000, 9999)

-- In-memory buffer for current session
local session_buffer = {}

-- Get the log file path
local function get_log_file()
  local cfg = config.get()
  local log_dir = cfg.themes_dir .. "/logs"
  vim.fn.mkdir(log_dir, "p")
  return log_dir .. "/hexwitch_" .. vim.fn.strftime("%Y%m%d") .. ".log"
end

-- Format log entry
---@param level string Log level
---@param module string Module name
---@param fn_name string Function name
---@param message string Log message
---@param data? table Additional data
---@return string formatted_entry
local function format_entry(level, module, fn_name, message, data)
  local timestamp = os.date("%Y-%m-%dT%H:%M:%S.3f", os.time())
  local entry = {
    timestamp = timestamp,
    level = level,
    module = module,
    ["function"] = fn_name,
    message = message,
    session_id = M.session_id,
  }

  if data then
    entry.data = data
  end

  return vim.json.encode(entry)
end

-- Core logging function
---@param level string Log level
---@param module string Module name
---@param fn_name string Function name
---@param message string Log message
---@param data? table Additional data
local function log(level, module, fn_name, message, data)
  local cfg = config.get()
  local level_num = M.LOG_LEVELS[level] or M.LOG_LEVELS.INFO
  local debug_level = M.LOG_LEVELS.DEBUG

  -- Skip if debug is off and this is a debug message
  if not cfg.debug and level_num <= debug_level then
    return
  end

  local formatted = format_entry(level, module, fn_name, message, data)

  -- Add to session buffer
  table.insert(session_buffer, {
    level = level,
    module = module,
    message = message,
    timestamp = os.date("%H:%M:%S"),
  })

  -- Keep session buffer manageable
  if #session_buffer > 1000 then
    table.remove(session_buffer, 1)
  end

  -- Write to file
  local log_file = get_log_file()
  local file = io.open(log_file, "a")
  if file then
    file:write(formatted .. "\n")
    file:close()
  end

  -- Console output for errors and important messages
  if level_num >= M.LOG_LEVELS.WARN or (cfg.debug and level_num >= M.LOG_LEVELS.INFO) then
    local prefix = string.format("[hexwitch:%s] %s", level:lower(), message)
    if level == "ERROR" then
      vim.api.nvim_err_writeln(prefix)
    else
      vim.notify(prefix, vim.log.levels[level])
    end
  end
end

-- Public logging functions
function M.debug(module, fn_name, message, data)
  log("DEBUG", module, fn_name, message, data)
end

function M.info(module, fn_name, message, data)
  log("INFO", module, fn_name, message, data)
end

function M.warn(module, fn_name, message, data)
  log("WARN", module, fn_name, message, data)
end

function M.error(module, fn_name, message, data)
  log("ERROR", module, fn_name, message, data)
end

-- Get session buffer entries
---@param level_filter? string Filter by level
---@return table entries
function M.get_session_logs(level_filter)
  if not level_filter then
    return vim.deepcopy(session_buffer)
  end

  local filtered = {}
  for _, entry in ipairs(session_buffer) do
    if entry.level == level_filter then
      table.insert(filtered, entry)
    end
  end
  return filtered
end

-- Clear session buffer
function M.clear_session_buffer()
  session_buffer = {}
end

-- Show recent logs in a floating window
function M.show_recent_logs()
  local logs = M.get_session_logs()
  if #logs == 0 then
    vim.notify("No logs in current session", vim.log.levels.INFO)
    return
  end

  -- Format logs for display
  local lines = {}
  for _, entry in ipairs(logs) do
    local line = string.format("[%s] %s:%s - %s",
      entry.timestamp, entry.level, entry.module, entry.message)
    table.insert(lines, line)
  end

  -- Create floating window
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, "modifiable", false)
  vim.api.nvim_buf_set_option(buf, "filetype", "log")

  local width = math.min(120, vim.fn.winwidth(0) - 10)
  local height = math.min(30, #logs + 2)

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    col = math.floor((vim.fn.winwidth(0) - width) / 2),
    row = math.floor((vim.fn.winheight(0) - height) / 2),
    border = "rounded",
    title = " Recent hexwitch Logs ",
    title_pos = "center",
  })

  -- Keymaps
  local opts = { buffer = buf, silent = true }
  vim.keymap.set("n", "q", function() vim.api.nvim_win_close(win, true) end, opts)
  vim.keymap.set("n", "<Esc>", function() vim.api.nvim_win_close(win, true) end, opts)
end

-- Performance timing helper
local timers = {}

function M.start_timer(name)
  timers[name] = vim.loop.hrtime()
end

function M.end_timer(name, module, fn_name)
  if not timers[name] then
    return
  end

  local elapsed_ms = (vim.loop.hrtime() - timers[name]) / 1e6
  M.debug(module, fn_name or "timer", string.format("Timer '%s': %.2fms", name, elapsed_ms))
  timers[name] = nil
  return elapsed_ms
end

-- Memory usage helper
function M.log_memory_usage(module, fn_name, context)
  local kb = vim.fn.system("ps -o rss= -p " .. vim.fn.getpid()):gsub("%s+", "")
  M.debug(module, fn_name or "memory", string.format("Memory usage %s: %s KB", context or "", kb))
end

return M