local M = {}

---Validate config with path prefix for better error messages
---@param path string The path to the field being validated
---@param tbl table The table to validate
---@return boolean is_valid
---@return string|nil error_message
function M.validate_path(path, tbl)
  local ok, err = pcall(vim.validate, tbl)
  if not ok then
    return false, path .. "." .. err
  end
  return true, nil
end

---Check if a command exists
---@param cmd string Command name
---@return boolean
function M.command_exists(cmd)
  return vim.fn.executable(cmd) == 1
end

---Check if a module can be required
---@param module string Module name
---@return boolean
function M.can_require(module)
  local ok = pcall(require, module)
  return ok
end

return M

