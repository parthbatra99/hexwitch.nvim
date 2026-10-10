local config = require("hexwitch.config")

local M = {}

---@enum hexwitch.LogLevel
M.levels = {
  DEBUG = vim.log.levels.DEBUG,
  INFO = vim.log.levels.INFO,
  WARN = vim.log.levels.WARN,
  ERROR = vim.log.levels.ERROR,
}

---Send notification to user
---@param message string
---@param level? number Log level (default: INFO)
function M.notify(message, level)
  level = level or M.levels.INFO
  vim.notify("[hexwitch] " .. message, level)
end

---Debug logging (only when debug mode enabled)
---@param message string
function M.debug(message)
  if config.get().debug then
    M.notify("[DEBUG] " .. message, M.levels.DEBUG)
  end
end

---Info notification
---@param message string
function M.info(message)
  M.notify(message, M.levels.INFO)
end

---Warning notification
---@param message string
function M.warn(message)
  M.notify(message, M.levels.WARN)
end

---Error notification
---@param message string
function M.error(message)
  M.notify(message, M.levels.ERROR)
end

return M

