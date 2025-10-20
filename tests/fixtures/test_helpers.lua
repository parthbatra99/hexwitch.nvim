-- Test helper functions for hexwitch testing
-- Provides utilities for setting up test environments and common test patterns

local M = {}

---Setup a temporary directory for testing
---@return string Path to temporary directory
M.setup_temp_dir = function()
  local temp_dir = vim.fn.tempname()
  vim.fn.mkdir(temp_dir, "p")
  return temp_dir
end

---Clean up a temporary directory
---@param temp_dir string Path to directory to clean up
M.cleanup_temp_dir = function(temp_dir)
  if vim.fn.isdirectory(temp_dir) == 1 then
    vim.fn.delete(temp_dir, "rf")
  end
end

---Mock vim functions for testing
---@return table Original function references for restoration
M.mock_vim_functions = function()
  local originals = {
    stdpath = vim.fn.stdpath,
    mkdir = vim.fn.mkdir,
    filereadable = vim.fn.filereadable,
    delete = vim.fn.delete,
    glob = vim.fn.glob,
    fnamemodify = vim.fn.fnamemodify,
  }

  vim.fn.stdpath = function(type)
    if type == "data" then
      return "/tmp/nvim-test-data"
    end
    return "/tmp/nvim"
  end

  return originals
end

---Restore original vim functions
---@param originals table Original function references
M.restore_vim_functions = function(originals)
  for func_name, original_func in pairs(originals) do
    vim.fn[func_name] = original_func
  end
end

---Setup mock configuration for testing
---@param overrides table Configuration values to override
---@return table Original configuration for restoration
M.setup_test_config = function(overrides)
  local config = require("hexwitch.config")
  local original_config = vim.deepcopy(config.get())

  local test_config = vim.tbl_extend("force", {
    openai_api_key = "test-api-key",
    model = "gpt-4o-2024-08-06",
    temperature = 0.7,
    timeout = 30000,
    ui_mode = "input",
    save_themes = true,
    themes_dir = "/tmp/nvim-test-themes",
    debug = false
  }, overrides or {})

  config.setup(test_config)
  return original_config
end

---Create a mock curl function for testing API calls
---@param response table Response to return from mock
---@return function Mock curl function
M.create_mock_curl = function(response)
  return function(url, options)
    local delay = response.delay or 0

    if delay > 0 then
      vim.defer_fn(function()
        options.callback(response)
      end, delay)
    else
      options.callback(response)
    end
  end
end

---Setup plenary curl mock
---@param response table Response to return
---@return function Cleanup function to restore original
M.setup_plenary_mock = function(response)
  local original_curl = package.loaded["plenary.curl"]
  package.loaded["plenary.curl"] = M.create_mock_curl(response)

  return function()
    package.loaded["plenary.curl"] = original_curl
  end
end

---Capture notifications for testing
---@return table Functions to control notification capture
M.capture_notifications = function()
  local notifications = {}
  local original_notify = {}

  local capture_types = { "info", "warn", "error", "debug", "success" }

  for _, notify_type in ipairs(capture_types) do
    original_notify[notify_type] = require("hexwitch.utils.notify")[notify_type]
    require("hexwitch.utils.notify")[notify_type] = function(msg)
      table.insert(notifications, {
        type = notify_type,
        msg = msg,
        timestamp = vim.loop.hrtime()
      })
    end
  end

  return {
    get = function() return vim.deepcopy(notifications) end,
    get_by_type = function(type)
      local filtered = {}
      for _, notif in ipairs(notifications) do
        if notif.type == type then
          table.insert(filtered, notif)
        end
      end
      return filtered
    end,
    clear = function() notifications = {} end,
    restore = function()
      for notify_type, original_func in pairs(original_notify) do
        require("hexwitch.utils.notify")[notify_type] = original_func
      end
    end,
    count = function() return #notifications end
  }
end

---Wait for a condition to be true with timeout
---@param condition_func Function that returns true when condition is met
---@param timeout number Maximum time to wait in milliseconds
---@param interval number Check interval in milliseconds
---@return boolean True if condition was met, false if timeout
M.wait_for_condition = function(condition_func, timeout, interval)
  timeout = timeout or 5000
  interval = interval or 100

  local start_time = vim.loop.hrtime()
  local timeout_ns = timeout * 1000000 -- Convert to nanoseconds

  while true do
    if condition_func() then
      return true
    end

    local elapsed = vim.loop.hrtime() - start_time
    if elapsed > timeout_ns then
      return false
    end

    vim.fn.wait(interval, function() return false end)
  end
end

---Create a spy function to track calls
---@param original_func function Original function to spy on (optional)
---@return table Spy object with call tracking
M.create_spy = function(original_func)
  local spy = {
    calls = {},
    args = {},
    returns = {},
    call_count = 0
  }

  spy.function = function(...)
    local args = {...}
    spy.call_count = spy.call_count + 1
    table.insert(spy.calls, {
      args = vim.deepcopy(args),
      timestamp = vim.loop.hrtime()
    })
    table.insert(spy.args, vim.deepcopy(args))

    if original_func then
      local results = {original_func(...)}
      table.insert(spy.returns, vim.deepcopy(results))
      return unpack(results)
    end
  end

  spy.reset = function()
    spy.calls = {}
    spy.args = {}
    spy.returns = {}
    spy.call_count = 0
  end

  spy.was_called = function()
    return spy.call_count > 0
  end

  spy.was_called_with = function(expected_args)
    for _, call in ipairs(spy.calls) do
      local match = true
      for i, expected_arg in ipairs(expected_args) do
        if call.args[i] ~= expected_arg then
          match = false
          break
        end
      end
      if match then
        return true
      end
    end
    return false
  end

  return spy
end

---Assert that a table has specific keys
---@param tbl table Table to check
---@param expected_keys table List of keys that should be present
M.assert_has_keys = function(tbl, expected_keys)
  assert.is_not_nil(tbl, "Table should not be nil")
  for _, key in ipairs(expected_keys) do
    assert.is_not_nil(tbl[key], "Table should have key: " .. tostring(key))
  end
end

---Assert that a theme has all required colors
---@param theme table Theme data to validate
M.assert_valid_theme = function(theme)
  assert.is_not_nil(theme, "Theme should not be nil")
  assert.is_not_nil(theme.name, "Theme should have a name")
  assert.is_not_nil(theme.colors, "Theme should have colors")

  local required_colors = {
    "bg", "fg", "bg_sidebar", "bg_float", "bg_statusline",
    "red", "orange", "yellow", "green", "cyan", "blue",
    "purple", "magenta", "comment", "selection", "cursor"
  }

  M.assert_has_keys(theme.colors, required_colors)

  -- Validate hex color format
  for color_name, color_value in pairs(theme.colors) do
    if color_value then
      assert.matches("^#[0-9A-Fa-f]{6}$", color_value,
        string.format("Color %s should be valid hex format", color_name))
    end
  end
end

---Compare two themes for equality
---@param theme1 table First theme
---@param theme2 table Second theme
---@return boolean True if themes are equal
M.themes_equal = function(theme1, theme2)
  if theme1.name ~= theme2.name then
    return false
  end

  if theme1.description ~= theme2.description then
    return false
  end

  for color_name, color_value in pairs(theme1.colors) do
    if theme2.colors[color_name] ~= color_value then
      return false
    end
  end

  return true
end

---Create a test context with common setup
---@param options table Test options
---@return table Test context object
M.create_test_context = function(options)
  options = options or {}

  local context = {
    temp_dir = M.setup_temp_dir(),
    original_config = M.setup_test_config(options.config),
    notifications = M.capture_notifications(),
    cleanup_functions = {}
  }

  -- Add cleanup function for temp directory
  table.insert(context.cleanup_functions, function()
    M.cleanup_temp_dir(context.temp_dir)
  end)

  -- Add cleanup function for config
  table.insert(context.cleanup_functions, function()
    local config = require("hexwitch.config")
    config.setup(context.original_config)
  end)

  -- Add cleanup function for notifications
  table.insert(context.cleanup_functions, function()
    context.notifications.restore()
  end)

  context.add_cleanup = function(func)
    table.insert(context.cleanup_functions, func)
  end

  context.cleanup = function()
    for _, cleanup_func in ipairs(context.cleanup_functions) do
      pcall(cleanup_func)
    end
  end

  return context
end

---Run a test with automatic cleanup
---@param test_func function Test function to run
---@param options table Test options
M.with_test_context = function(test_func, options)
  local context = M.create_test_context(options)

  local success, error_msg = pcall(test_func, context)

  -- Always cleanup, even if test fails
  pcall(context.cleanup)

  if not success then
    error(error_msg)
  end
end

---Generate a random hex color
---@return string Random hex color
M.random_hex_color = function()
  return string.format("#%02x%02x%02x",
    math.random(0, 255),
    math.random(0, 255),
    math.random(0, 255)
  )
end

---Generate random theme data
---@param name string Theme name (optional)
---@return table Random theme data
M.generate_random_theme = function(name)
  local colors = {}
  local color_names = {
    "bg", "fg", "bg_sidebar", "bg_float", "bg_statusline",
    "red", "orange", "yellow", "green", "cyan", "blue",
    "purple", "magenta", "comment", "selection", "cursor"
  }

  for _, color_name in ipairs(color_names) do
    colors[color_name] = M.random_hex_color()
  end

  return {
    name = name or "random-theme-" .. math.random(1000, 9999),
    description = "Randomly generated test theme",
    colors = colors
  }
end

return M