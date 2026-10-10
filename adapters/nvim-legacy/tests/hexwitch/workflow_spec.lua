local hexwitch = require("hexwitch")
local config = require("hexwitch.config")

describe("hexwitch workflow tests", function()
  local original_config
  local temp_dir
  local mock_openai_responses

  before_each(function()
    -- Store original config
    original_config = vim.deepcopy(config.get())

    -- Create temporary directory for testing
    temp_dir = vim.fn.tempname()
    vim.fn.mkdir(temp_dir, "p")

    -- Mock vim.fn.stdpath
    vim.fn.stdpath = function(type)
      if type == "data" then
        return temp_dir
      end
      return "/tmp/nvim"
    end

    -- Setup test configuration
    config.setup({
      openai_api_key = "test-key",
      model = "gpt-4o-2024-08-06",
      temperature = 0.7,
      timeout = 30000,
      save_themes = true,
      themes_dir = temp_dir .. "/themes",
      debug = true
    })

    -- Create mock responses
    mock_openai_responses = {
      success = {
        status = 200,
        body = vim.json.encode({
          choices = {
            {
              message = {
                content = vim.json.encode({
                  name = "workflow-test-theme",
                  description = "A theme generated in workflow tests",
                  colors = {
                    bg = "#1a1b26",
                    fg = "#c0caf5",
                    bg_sidebar = "#1a1b26",
                    bg_float = "#24283b",
                    bg_statusline = "#1f2335",
                    red = "#f7768e",
                    orange = "#ff9e64",
                    yellow = "#e0af68",
                    green = "#9ece6a",
                    cyan = "#73daca",
                    blue = "#7aa2f7",
                    purple = "#bb9af7",
                    magenta = "#d18616",
                    comment = "#565f89",
                    selection = "#33467c",
                    cursor = "#c0caf5"
                  }
                })
              }
            }
          }
        })
      },
      error_no_api_key = {
        status = 401,
        body = vim.json.encode({
          error = {
            message = "Invalid API key"
          }
        })
      },
      error_network = {
        status = 500,
        body = vim.json.encode({
          error = {
            message = "Internal server error"
          }
        })
      }
    }
  end)

  after_each(function()
    -- Clean up temp directory
    if vim.fn.isdirectory(temp_dir) == 1 then
      vim.fn.delete(temp_dir, "rf")
    end

    -- Restore original config
    config.setup(original_config)

    -- Clear global state
    vim.g.colors_name = nil
    for i = 0, 15 do
      vim.g["terminal_color_" .. i] = nil
    end
  end)

  describe("complete user workflow", function()
    it("should handle full theme generation and save workflow", function()
      local mock_curl = {
        post = function(url, options)
          -- Simulate successful API call
          options.callback(mock_openai_responses.success)
        end
      }

      -- Mock plenary.curl
      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      -- Capture notifications
      local notifications = {}
      local original_notify = {
        info = require("hexwitch.utils.notify").info,
        debug = require("hexwitch.utils.notify").debug,
        error = require("hexwitch.utils.notify").error
      }

      require("hexwitch.utils.notify").info = function(msg)
        table.insert(notifications, { type = "info", msg = msg })
      end
      require("hexwitch.utils.notify").debug = function(msg)
        table.insert(notifications, { type = "debug", msg = msg })
      end
      require("hexwitch.utils.notify").error = function(msg)
        table.insert(notifications, { type = "error", msg = msg })
      end

      -- Generate theme
      local generation_completed = false
      hexwitch.generate("a dark theme with purple accents", function(result, error)
        assert.is_nil(error)
        assert.is_not_nil(result)
        assert.equals("workflow-test-theme", result.name)

        -- Check theme was applied
        assert.equals("workflow-test-theme", vim.g.colors_name)

        -- Check basic highlights
        local normal_hl = vim.api.nvim_get_hl(0, { name = "Normal" })
        assert.equals("#c0caf5", string.format("#%06x", normal_hl.fg))
        assert.equals("#1a1b26", string.format("#%06x", normal_hl.bg))

        generation_completed = true
      end)

      assert.is_true(generation_completed)

      -- Save the theme
      hexwitch.save("saved-workflow-theme")

      -- Check if theme was saved
      local themes = hexwitch.list_themes()
      assert.is_true(vim.tbl_contains(themes, "saved-workflow-theme"))

      -- Load the saved theme
      hexwitch.load("saved-workflow-theme")

      -- Verify theme was loaded correctly
      assert.equals("workflow-test-theme", vim.g.colors_name)

      -- Restore original modules
      package.loaded["plenary.curl"] = original_curl
      require("hexwitch.utils.notify").info = original_notify.info
      require("hexwitch.utils.notify").debug = original_notify.debug
      require("hexwitch.utils.notify").error = original_notify.error
    end)

    it("should handle API key error workflow", function()
      config.setup({ openai_api_key = nil })

      local mock_curl = {
        post = function(url, options)
          options.callback(mock_openai_responses.error_no_api_key)
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        assert.matches("OpenAI API key not configured", error)
        error_received = true
      end)

      assert.is_true(error_received)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle network error workflow", function()
      local mock_curl = {
        post = function(url, options)
          options.callback(mock_openai_responses.error_network)
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        assert.matches("API request failed with status 500", error)
        error_received = true
      end)

      assert.is_true(error_received)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle theme management workflow", function()
      -- Create multiple themes
      local theme_names = { "theme1", "theme2", "theme3" }

      for i, name in ipairs(theme_names) do
        -- Mock successful API response
        local mock_curl = {
          post = function(url, options)
            local response = vim.deepcopy(mock_openai_responses.success)
            response.body = vim.json.encode({
              choices = {
                {
                  message = {
                    content = vim.json.encode({
                      name = name,
                      description = "Test theme " .. i,
                      colors = mock_openai_responses.success.body and
                        vim.json.decode(mock_openai_responses.success.body).choices[1].message.content and
                        vim.json.decode(vim.json.decode(mock_openai_responses.success.body).choices[1].message.content).colors or {}
                    })
                  }
                }
              }
            })
            options.callback(response)
          end
        }

        local original_curl = package.loaded["plenary.curl"]
        package.loaded["plenary.curl"] = mock_curl

        hexwitch.generate("test theme " .. i, function(result, error)
          assert.is_nil(error)
          hexwitch.save(name)
        end)

        package.loaded["plenary.curl"] = original_curl
      end

      -- List all themes
      local themes = hexwitch.list_themes()
      for _, name in ipairs(theme_names) do
        assert.is_true(vim.tbl_contains(themes, name))
      end

      -- Load and verify each theme
      for _, name in ipairs(theme_names) do
        hexwitch.load(name)
        assert.equals(name, vim.g.colors_name)
      end

      -- Delete themes one by one
      for _, name in ipairs(theme_names) do
        hexwitch.delete(name)
        local remaining_themes = hexwitch.list_themes()
        assert.is_false(vim.tbl_contains(remaining_themes, name))
      end

      -- Final check - should be empty
      local final_themes = hexwitch.list_themes()
      assert.equals(0, #final_themes)
    end)
  end)

  describe("configuration workflow", function()
    it("should handle configuration updates", function()
      -- Test with default configuration
      local default_config = config.get()
      assert.equals("gpt-4o-2024-08-06", default_config.model)
      assert.equals(0.7, default_config.temperature)
      assert.is_true(default_config.save_themes)

      -- Update configuration
      local new_config = {
        model = "gpt-4-turbo",
        temperature = 0.9,
        save_themes = false,
        debug = true
      }

      config.setup(new_config)
      local updated_config = config.get()

      assert.equals("gpt-4-turbo", updated_config.model)
      assert.equals(0.9, updated_config.temperature)
      assert.is_false(updated_config.save_themes)
      assert.is_true(updated_config.debug)
      assert.equals("test-key", updated_config.openai_api_key) -- Should preserve existing API key
    end)

    it("should handle partial configuration updates", function()
      -- Start with full config
      config.setup({
        openai_api_key = "initial-key",
        model = "gpt-4o",
        temperature = 0.5,
        timeout = 60000,
        save_themes = true,
        debug = false
      })

      -- Update only some parameters
      config.setup({
        model = "gpt-4-turbo",
        debug = true
      })

      local updated_config = config.get()

      assert.equals("initial-key", updated_config.openai_api_key) -- Preserved
      assert.equals("gpt-4-turbo", updated_config.model) -- Updated
      assert.equals(0.5, updated_config.temperature) -- Preserved
      assert.equals(60000, updated_config.timeout) -- Preserved
      assert.equals(true, updated_config.save_themes) -- Preserved
      assert.equals(true, updated_config.debug) -- Updated
    end)

    it("should handle invalid configuration gracefully", function()
      -- Test with invalid configuration values
      local invalid_configs = {
        { model = 123 }, -- invalid type
        { temperature = 1.5 }, -- out of range
        { timeout = -1000 }, -- negative
        { ai_provider = "invalid" } -- invalid option
      }

      for _, invalid_config in ipairs(invalid_configs) do
        -- This should not crash and should use default values for invalid params
        pcall(function()
          config.setup(invalid_config)
        end)

        -- Verify we still have a valid configuration
        local current_config = config.get()
        assert.is_not_nil(current_config)
        assert.is_not_nil(current_config.model)
        assert.is_not_nil(current_config.temperature)
      end
    end)
  end)

  describe("error handling workflow", function()
    it("should handle missing dependencies gracefully", function()
      -- Temporarily remove plenary
      local plenary = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = nil

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        assert.matches("plenary.nvim is required", error)
        error_received = true
      end)

      assert.is_true(error_received)

      -- Restore plenary
      package.loaded["plenary.curl"] = plenary
    end)

    it("should handle invalid theme data from API", function()
      local mock_curl = {
        post = function(url, options)
          options.callback({
            status = 200,
            body = vim.json.encode({
              choices = {
                {
                  message = {
                    content = vim.json.encode({
                      name = "invalid-theme",
                      description = "Theme with invalid colors",
                      colors = {
                        bg = "invalid-color", -- Invalid hex color
                        fg = nil, -- Missing required color
                      }
                    })
                  }
                }
              }
            })
          })
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        -- The API call might succeed, but theme application should fail
        if result then
          -- If we got a result, try to apply it and check for errors
          local applier = require("hexwitch.theme.applier")
          local notifications = {}
          local original_notify = require("hexwitch.utils.notify").error
          require("hexwitch.utils.notify").error = function(msg)
            table.insert(notifications, msg)
          end

          applier.apply(result)

          -- Should have received an error notification about invalid colors
          assert.is_true(#notifications > 0)

          require("hexwitch.utils.notify").error = original_notify
        else
          -- Or the API call failed
          assert.is_not_nil(error)
          error_received = true
        end
      end)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle file system errors in storage", function()
      -- Test with read-only directory
      local theme_dir = temp_dir .. "/readonly"
      vim.fn.mkdir(theme_dir, "p")

      -- Make directory read-only (Unix systems only)
      if vim.fn.has("unix") == 1 then
        vim.fn.system("chmod 444 " .. theme_dir)
      end

      local notifications = {}
      local original_notify = require("hexwitch.utils.notify").error
      require("hexwitch.utils.notify").error = function(msg)
        table.insert(notifications, msg)
      end

      -- Try to save theme to read-only directory
      config.setup({ themes_dir = theme_dir })
      hexwitch.save("readonly-test")

      -- Should have received an error notification
      assert.is_true(#notifications > 0)

      -- Restore permissions and cleanup
      if vim.fn.has("unix") == 1 then
        vim.fn.system("chmod 755 " .. theme_dir)
      end
      vim.fn.delete(theme_dir, "rf")

      require("hexwitch.utils.notify").error = original_notify
    end)
  end)

  describe("performance workflow", function()
    it("should handle rapid successive operations", function()
      local mock_curl = {
        post = function(url, options)
          -- Simulate very fast response
          options.callback(mock_openai_responses.success)
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      local operations_completed = 0
      local total_operations = 5

      -- Perform multiple operations rapidly
      for i = 1, total_operations do
        hexwitch.generate("rapid test " .. i, function(result, error)
          assert.is_nil(error)
          assert.is_not_nil(result)
          hexwitch.save("rapid-theme-" .. i)
          operations_completed = operations_completed + 1
        end)
      end

      -- Wait for all operations to complete
      local max_wait = 5000 -- 5 seconds max wait
      local wait_time = 0
      local wait_interval = 100

      while operations_completed < total_operations and wait_time < max_wait do
        vim.fn.wait(wait_interval, function() return false end)
        wait_time = wait_time + wait_interval
      end

      assert.equals(total_operations, operations_completed)

      -- Verify all themes were saved
      local themes = hexwitch.list_themes()
      for i = 1, total_operations do
        assert.is_true(vim.tbl_contains(themes, "rapid-theme-" .. i))
      end

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle timeout scenarios gracefully", function()
      config.setup({ timeout = 100 }) -- Very short timeout

      local mock_curl = {
        post = function(url, options)
          -- Don't call callback to simulate timeout
          -- In real scenario, plenary.curl would handle timeout
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      -- This should not hang indefinitely
      local start_time = vim.loop.hrtime()

      hexwitch.generate("timeout test", function(result, error)
        -- This callback may not be called due to timeout
      end)

      local end_time = vim.loop.hrtime()
      local elapsed = (end_time - start_time) / 1000000 -- Convert to milliseconds

      -- Should complete within reasonable time (plus some buffer)
      assert.is_true(elapsed < 1000) -- Should be less than 1 second

      package.loaded["plenary.curl"] = original_curl
    end)
  end)
end)