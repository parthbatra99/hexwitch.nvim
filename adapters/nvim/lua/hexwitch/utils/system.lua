local M = {}

-- Cross-platform function to open URLs and directories
---@param path string Path to open (URL or directory)
---@return boolean success Whether the operation succeeded
function M.open_path(path)
  if not path or type(path) ~= "string" or path == "" then
    return false
  end

  -- Validate the path to prevent command injection
  -- Allow only valid URL patterns for URLs
  -- Allow only alphanumeric, forward slashes, dots, hyphens, underscores for directories
  local is_url = path:match("^https?://[a-zA-Z0-9._/-]+$")
  local is_valid_path = is_url or path:match("^[%w%./_%-]+$")

  if not is_valid_path then
    vim.notify("Security error: invalid path format", vim.log.levels.ERROR)
    return false
  end

  local success = false
  local error_msg = ""

  -- Use platform-appropriate command
  if vim.fn.has("mac") == 1 then
    -- macOS
    local result = vim.fn.system({ "open", path })
    success = vim.v.shell_error == 0
    if not success then
      error_msg = result
    end
  elseif vim.fn.has("unix") == 1 then
    -- Linux/Unix
    local result = vim.fn.system({ "xdg-open", path })
    success = vim.v.shell_error == 0
    if not success then
      error_msg = result
    end
  elseif vim.fn.has("win32") == 1 then
    -- Windows
    local result = vim.fn.system({ "cmd", "/c", "start", "", path })
    success = vim.v.shell_error == 0
    if not success then
      error_msg = result
    end
  else
    vim.notify("Unsupported platform: cannot open file or URL", vim.log.levels.ERROR)
    return false
  end

  if not success then
    vim.notify("Failed to open path: " .. (error_msg or "Unknown error"), vim.log.levels.ERROR)
  end

  return success
end

-- Safely execute system commands with proper argument separation
---@param command string Command to execute
---@param args table Arguments to pass to command
---@return string output Command output
---@return number exit_code Exit code of the command
function M.safely_execute(command, args)
  if not command or type(command) ~= "string" then
    return "", -1
  end

  if not args or type(args) ~= "table" then
    args = {}
  end

  -- Validate command to prevent injection
  if not command:match("^[%w_%-]+$") then
    vim.notify("Security error: invalid command format", vim.log.levels.ERROR)
    return "", -1
  end

  -- Validate arguments
  for i, arg in ipairs(args) do
    if type(arg) ~= "string" then
      args[i] = tostring(arg)
    end
    -- Reject dangerous argument patterns
    if arg:match("[;|&`$<>%(%){}]") then
      vim.notify("Security error: invalid argument format", vim.log.levels.ERROR)
      return "", -1
    end
  end

  -- Execute with parameterized arguments
  local full_args = {command}
  for _, arg in ipairs(args) do
    table.insert(full_args, arg)
  end

  local result = vim.fn.system(full_args)
  return result, vim.v.shell_error
end

return M