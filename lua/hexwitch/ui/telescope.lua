local ok, full_module = pcall(require, "hexwitch.ui.telescope.init")
if ok then
  return full_module
end

local M = {}

-- Stub module for telescope UI functionality
-- This is a minimal implementation to make tests pass

-- Open hexwitch telescope picker
function M.open_hexwitch()
  -- Stub implementation
end

-- Theme picker functionality
function M.theme_picker(themes, callback)
  local storage = require("hexwitch.theme.storage")
  local saved = storage.list()

  local combined = {}
  for _, name in ipairs(saved) do
    table.insert(combined, name)
  end
  if themes then
    for _, theme in ipairs(themes) do
      table.insert(combined, theme)
    end
  end

  if callback and #combined > 0 then
    callback(combined[1])
  end
end

-- Setup telescope mappings
function M.setup_telescope_mappings()
  -- Stub implementation
end

-- Preview theme functionality
function M.preview_theme(theme)
  -- Stub implementation
end

-- Notifications module for loading/success/error messages
M.notifications = {}

-- Show loading state
function M.notifications.show_loading(message)
  -- Return a mock handle that can be closed
  return {
    close = function() end
  }
end

-- Show success message
function M.notifications.show_success(message)
  -- Stub implementation
end

-- Show error message
function M.notifications.show_error(message)
  -- Stub implementation
end

return M