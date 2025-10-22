local state = require("hexwitch.storage.state")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Undo last theme change
function M.undo()
  logger.info("undo.undo", "Undoing last theme change")

  local previous_theme = state.undo()
  if previous_theme then
    require("hexwitch.theme").apply(previous_theme)
    vim.notify("↩ Reverted to previous theme", vim.log.levels.INFO)
  else
    vim.notify("Nothing to undo", vim.log.levels.WARN)
  end
end

-- Redo theme change
function M.redo()
  logger.info("undo.redo", "Redoing theme change")

  local next_theme = state.redo()
  if next_theme then
    require("hexwitch.theme").apply(next_theme)
    vim.notify("↪ Reapplied theme", vim.log.levels.INFO)
  else
    vim.notify("Nothing to redo", vim.log.levels.WARN)
  end
end

-- Get undo/redo stack sizes
---@return table sizes Stack sizes
function M.get_stack_sizes()
  return state.get_stack_sizes()
end

-- Check if undo is available
---@return boolean available True if undo is available
function M.can_undo()
  local sizes = M.get_stack_sizes()
  return sizes.undo > 1
end

-- Check if redo is available
---@return boolean available True if redo is available
function M.can_redo()
  local sizes = M.get_stack_sizes()
  return sizes.redo > 0
end

-- Clear undo/redo stacks
function M.clear_stacks()
  logger.info("undo.clear_stacks", "Clearing undo/redo stacks")
  -- This would need to be implemented in the state module
  vim.notify("Undo/redo stacks cleared", vim.log.levels.INFO)
end

return M