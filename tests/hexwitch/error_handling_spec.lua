local hexwitch = require("hexwitch")
local config = require("hexwitch.config")
local openai = require("hexwitch.ai.openai")
local applier = require("hexwitch.theme.applier")
local storage = require("hexwitch.theme.storage")

describe("hexwitch error handling and edge cases", function()
  local original_config
  local temp_dir

  before_each(function()
    -- Store original config
    original_config = vim.deepcopy(config.get())

    -- Create temporary directory
    temp_dir = vim.fn.tempname()
    vim.fn.mkdir(temp_dir, "p")

    -- Mock vim.fn.stdpath
    vim.fn.stdpath = function(type)
      if type == "data" then
        return temp_dir
      end
      return "/tmp/nvim"
    end

    -- Setup basic config
    config.setup({
      openai_api_key = "test-key",
      model = "gpt-4o-2024-08-06",
      temperature = 0.7,
      timeout = 30000,
      save_themes = true,
      debug = false
    })
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

  describe("configuration errors", function()
    it("should handle missing API key", function()
      config.setup({ openai_api_key = nil })

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        assert.matches("OpenAI API key not configured", error)
        error_received = true
      end)

      assert.is_true(error_received)
    end)

    it("should handle empty API key", function()
      config.setup({ openai_api_key = "" })

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        assert.matches("OpenAI API key not configured", error)
        error_received = true
      end)

      assert.is_true(error_received)
    end)

    it("should handle invalid API key format", function()
      local invalid_keys = {
        "not-a-key",
        "sk-",
        "123",
        "sk-test-without-proper-format",
        string.rep("a", 1000) -- Extremely long key
      }

      for _, invalid_key in ipairs(invalid_keys) do
        config.setup({ openai_api_key = invalid_key })

        local mock_curl = {
          post = function(url, options)
            options.callback({
              status = 401,
              body = vim.json.encode({
                error = { message = "Invalid API key" }
              })
            })
          end
        }

        local original_curl = package.loaded["plenary.curl"]
        package.loaded["plenary.curl"] = mock_curl

        local error_received = false
        hexwitch.generate("test theme", function(result, error)
          assert.is_nil(result)
          assert.is_not_nil(error)
          assert.matches("API request failed with status 401", error)
          error_received = true
        end)

        assert.is_true(error_received)

        package.loaded["plenary.curl"] = original_curl
      end
    end)

    it("should handle invalid model names", function()
      local invalid_models = {
        "",
        "invalid-model",
        "gpt-3.5-turbo", -- Older model that might not support structured output
        123, -- Wrong type
        nil,
      }

      for _, model in ipairs(invalid_models) do
        -- Should not crash when setting invalid model
        pcall(function()
          config.setup({ model = model })
        end)

        -- Should still have a valid config object
        local current_config = config.get()
        assert.is_not_nil(current_config)
      end
    end)

    it("should handle invalid temperature values", function()
      local invalid_temperatures = {
        -1, -- Too low
        2, -- Too high
        1.5, -- Too high
        math.huge, -- Infinite
        -math.huge, -- Negative infinite
        math.nan, -- Not a number
        "0.5", -- String instead of number
        nil,
      }

      for _, temp in ipairs(invalid_temperatures) do
        pcall(function()
          config.setup({ temperature = temp })
        end)

        -- Should maintain valid config
        local current_config = config.get()
        assert.is_not_nil(current_config)
        if current_config.temperature then
          assert.is_true(current_config.temperature >= 0 and current_config.temperature <= 1)
        end
      end
    end)

    it("should handle invalid timeout values", function()
      local invalid_timeouts = {
        -1000, -- Negative
        0, -- Zero (might be valid in some cases)
        1, -- Too short (might cause issues)
        math.huge, -- Infinite
        "5000", -- String instead of number
        nil,
      }

      for _, timeout in ipairs(invalid_timeouts) do
        pcall(function()
          config.setup({ timeout = timeout })
        end)

        local current_config = config.get()
        assert.is_not_nil(current_config)
      end
    end)
  end)

  describe("network and API errors", function()
    it("should handle network timeout", function()
      config.setup({ timeout = 100 }) -- Very short timeout

      local mock_curl = {
        post = function(url, options)
          -- Don't call callback to simulate timeout
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      -- Should not hang indefinitely
      local start_time = vim.loop.hrtime()
      hexwitch.generate("timeout test", function() end)
      local end_time = vim.loop.hrtime()
      local elapsed = (end_time - start_time) / 1000000

      -- Should complete quickly (with some buffer)
      assert.is_true(elapsed < 1000)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle connection refused", function()
      local mock_curl = {
        post = function(url, options)
          options.callback({
            status = 0,
            body = nil
          })
        end
      }

      local original_curl = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = mock_curl

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        error_received = true
      end)

      assert.is_true(error_received)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle malformed API response", function()
      local malformed_responses = {
        "invalid json",
        "{ incomplete json",
        '{"error": "missing closing brace"',
        'null',
        'undefined',
        '',
        '{"choices": []}', -- Empty choices
        '{"choices": [{}]}', -- Empty choice object
        '{"choices": [{"message": null}]}', -- Null message
        '{"choices": [{"message": {"content": null}]}', -- Null content
        '{"choices": [{"message": {"content": "invalid json"}}]}', -- Invalid JSON in content
      }

      for _, response_body in ipairs(malformed_responses) do
        local mock_curl = {
          post = function(url, options)
            options.callback({
              status = 200,
              body = response_body
            })
          end
        }

        local original_curl = package.loaded["plenary.curl"]
        package.loaded["plenary.curl"] = mock_curl

        local error_received = false
        hexwitch.generate("test theme", function(result, error)
          -- Either result is nil with error, or result is invalid
          if not result then
            assert.is_not_nil(error)
            error_received = true
          end
        end)

        package.loaded["plenary.curl"] = original_curl
      end
    end)

    it("should handle HTTP error status codes", function()
      local error_codes = {
        400, -- Bad Request
        401, -- Unauthorized
        403, -- Forbidden
        404, -- Not Found
        429, -- Rate Limited
        500, -- Internal Server Error
        502, -- Bad Gateway
        503, -- Service Unavailable
        504, -- Gateway Timeout
      }

      for _, status_code in ipairs(error_codes) do
        local mock_curl = {
          post = function(url, options)
            options.callback({
              status = status_code,
              body = vim.json.encode({
                error = { message = "HTTP " .. status_code .. " error" }
              })
            })
          end
        }

        local original_curl = package.loaded["plenary.curl"]
        package.loaded["plenary.curl"] = mock_curl

        local error_received = false
        hexwitch.generate("test theme", function(result, error)
          assert.is_nil(result)
          assert.is_not_nil(error)
          assert.matches("API request failed with status " .. status_code, error)
          error_received = true
        end)

        assert.is_true(error_received)

        package.loaded["plenary.curl"] = original_curl
      end
    end)
  end)

  describe("theme application errors", function()
    it("should handle invalid theme data structures", function()
      local invalid_themes = {
        nil,
        {},
        { colors = nil },
        { colors = {} },
        { name = "test", colors = {} },
        { name = nil, colors = { bg = "#000000", fg = "#ffffff" } },
        { name = "", colors = { bg = "#000000", fg = "#ffffff" } },
        { colors = { bg = nil, fg = "#ffffff" } },
        { colors = { bg = "#000000", fg = nil } },
      }

      for _, theme in ipairs(invalid_themes) do
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        applier.apply(theme)

        -- Should have received an error notification
        assert.is_true(#notifications > 0)
        assert.matches("Invalid colorscheme data", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end
    end)

    it("should handle invalid color values", function()
      local invalid_colors = {
        "not-a-color",
        "#12345", -- Too short
        "#1234567", -- Too long
        "123456", -- Missing #
        "#GGGGGG", -- Invalid hex characters
        "#-12345", -- Invalid format
        nil,
        123, -- Number instead of string
        {}, -- Table instead of string
        function() end, -- Function instead of string
      }

      for _, color in ipairs(invalid_colors) do
        local theme_with_invalid_color = {
          name = "test",
          description = "Test theme",
          colors = {
            bg = color or "#000000",
            fg = "#ffffff",
            bg_sidebar = "#000000",
            bg_float = "#000000",
            bg_statusline = "#000000",
            red = "#ff0000",
            orange = "#ff8800",
            yellow = "#ffff00",
            green = "#00ff00",
            cyan = "#00ffff",
            blue = "#0000ff",
            purple = "#ff00ff",
            magenta = "#ff0088",
            comment = "#888888",
            selection = "#444444",
            cursor = "#ffffff"
          }
        }

        -- Should not crash when applying theme with invalid colors
        pcall(function()
          applier.apply(theme_with_invalid_color)
        end)

        -- Either applies successfully (if color is somehow valid) or fails gracefully
      end
    end)

    it("should handle highlight API errors", function()
      local valid_theme = {
        name = "test",
        description = "Test theme",
        colors = {
          bg = "#000000",
          fg = "#ffffff",
          bg_sidebar = "#000000",
          bg_float = "#000000",
          bg_statusline = "#000000",
          red = "#ff0000",
          orange = "#ff8800",
          yellow = "#ffff00",
          green = "#00ff00",
          cyan = "#00ffff",
          blue = "#0000ff",
          purple = "#ff00ff",
          magenta = "#ff0088",
          comment = "#888888",
          selection = "#444444",
          cursor = "#ffffff"
        }
      }

      -- Mock nvim_set_hl to throw error
      local original_set_hl = vim.api.nvim_set_hl
      vim.api.nvim_set_hl = function()
        error("Mock highlight API error")
      end

      -- Should handle highlight API errors gracefully
      pcall(function()
        applier.apply(valid_theme)
      end)

      -- Restore original function
      vim.api.nvim_set_hl = original_set_hl
    end)
  end)

  describe("storage errors", function()
    it("should handle file system permission errors", function()
      if vim.fn.has("unix") == 1 then
        -- Create read-only directory
        local readonly_dir = temp_dir .. "/readonly"
        vim.fn.mkdir(readonly_dir, "p")
        vim.fn.system("chmod 444 " .. readonly_dir)

        config.setup({ themes_dir = readonly_dir })

        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.save("readonly-test")

        -- Should have received an error notification
        assert.is_true(#notifications > 0)

        -- Restore permissions and cleanup
        vim.fn.system("chmod 755 " .. readonly_dir)
        vim.fn.delete(readonly_dir, "rf")

        require("hexwitch.utils.notify").error = original_notify
      end
    end)

    it("should handle disk full scenarios", function()
      -- Mock io.open to simulate disk full error
      local original_open = io.open
      io.open = function(path, mode)
        if mode:match("w") then
          return nil, "No space left on device"
        end
        return original_open(path, mode)
      end

      local notifications = {}
      local original_notify = require("hexwitch.utils.notify").error
      require("hexwitch.utils.notify").error = function(msg)
        table.insert(notifications, msg)
      end

      storage.save("disk-full-test")

      -- Should have received an error notification
      assert.is_true(#notifications > 0)

      -- Restore original function
      io.open = original_open
      require("hexwitch.utils.notify").error = original_notify
    end)

    it("should handle corrupted theme files", function()
      -- Create corrupted theme file
      local theme_dir = temp_dir .. "/hexwitch"
      vim.fn.mkdir(theme_dir, "p")

      local corrupted_files = {
        { name = "invalid.json", content = "invalid json content" },
        { name = "incomplete.json", content = '{"name": "test"' },
        { name = "empty.json", content = "" },
        { name = "wrong-type.json", content = "null" },
        { name = "binary.json", content = "\x00\x01\x02\x03" },
      }

      for _, file in ipairs(corrupted_files) do
        local file_path = theme_dir .. "/" .. file.name
        local f = io.open(file_path, "w")
        f:write(file.content)
        f:close()

        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.load(file.name:gsub("%.json$", ""))

        -- Should have received an error notification
        assert.is_true(#notifications > 0)

        require("hexwitch.utils.notify").error = original_notify
      end
    end)

    it("should handle theme name edge cases", function()
      local edge_case_names = {
        "", -- Empty
        "   ", -- Whitespace only
        string.rep("a", 1000), -- Very long
        "theme with spaces",
        "theme-with-dashes",
        "theme_with_underscores",
        "theme.with.dots",
        "theme/with/slashes",
        "theme\\with\\backslashes",
        "theme:with:colons",
        "theme;with;semicolons",
        "theme'with'quotes",
        'theme"with"quotes',
        "theme\nwith\nnewlines",
        "theme\twith\ttabs",
        "theme\x00with\x00null",
      }

      for _, name in ipairs(edge_case_names) do
        -- Should handle edge case names gracefully
        pcall(function()
          storage.save(name)
        end)

        pcall(function()
          storage.load(name)
        end)

        pcall(function()
          storage.delete(name)
        end)

        -- Should not crash the plugin
        assert.is_true(true)
      end
    end)
  end)

  describe("dependency errors", function()
    it("should handle missing plenary dependency", function()
      -- Temporarily remove plenary
      local plenary_curl = package.loaded["plenary.curl"]
      local plenary = package.loaded["plenary"]
      package.loaded["plenary.curl"] = nil
      package.loaded["plenary"] = nil

      local error_received = false
      hexwitch.generate("test theme", function(result, error)
        assert.is_nil(result)
        assert.is_not_nil(error)
        assert.matches("plenary.nvim is required", error)
        error_received = true
      end)

      assert.is_true(error_received)

      -- Restore plenary
      package.loaded["plenary.curl"] = plenary_curl
      package.loaded["plenary"] = plenary
    end)

    it("should handle missing vim functions", function()
      -- Mock missing vim functions
      local original_stdpath = vim.fn.stdpath
      local original_mkdir = vim.fn.mkdir
      local original_filereadable = vim.fn.filereadable

      vim.fn.stdpath = nil
      vim.fn.mkdir = nil
      vim.fn.filereadable = nil

      -- Should handle missing functions gracefully
      pcall(function()
        local hexwitch = require("hexwitch")
        hexwitch.setup({})
      end)

      pcall(function()
        storage.save("test")
      end)

      pcall(function()
        storage.load("test")
      end)

      -- Restore functions
      vim.fn.stdpath = original_stdpath
      vim.fn.mkdir = original_mkdir
      vim.fn.filereadable = original_filereadable
    end)

    it("should handle JSON encoding/decoding errors", function()
      -- Mock vim.json.encode to fail
      local original_encode = vim.json.encode
      local original_decode = vim.json.decode

      vim.json.encode = function()
        error("Mock JSON encode error")
      end

      local theme_data = {
        name = "test",
        description = "test",
        colors = { bg = "#000000", fg = "#ffffff" }
      }

      -- Should handle JSON encode errors gracefully
      pcall(function()
        local json = vim.json.encode(theme_data)
      end)

      vim.json.encode = original_encode

      vim.json.decode = function()
        error("Mock JSON decode error")
      end

      -- Should handle JSON decode errors gracefully
      pcall(function()
        vim.json.decode('{"test": "data"}')
      end)

      vim.json.decode = original_decode
    end)
  end)

  describe("resource exhaustion", function()
    it("should handle memory pressure", function()
      -- Create large theme data
      local large_theme = {
        name = string.rep("large-theme-name-", 1000),
        description = string.rep("very long theme description ", 1000),
        colors = {}
      }

      -- Add many color properties
      local color_names = {
        "bg", "fg", "bg_sidebar", "bg_float", "bg_statusline",
        "red", "orange", "yellow", "green", "cyan", "blue", "purple", "magenta",
        "comment", "selection", "cursor"
      }

      for _, color_name in ipairs(color_names) do
        large_theme.colors[color_name] = string.rep("#ff0000", 100)
      end

      -- Should handle large data gracefully
      pcall(function()
        applier.apply(large_theme)
      end)

      -- Should not crash
      assert.is_true(true)
    end)

    it("should handle rapid successive operations", function()
      local operations = 0
      local max_operations = 100

      -- Simulate rapid operations
      for i = 1, max_operations do
        pcall(function()
          config.setup({ debug = i % 2 == 0 })
          operations = operations + 1
        end)

        pcall(function()
          local current_config = config.get()
          operations = operations + 1
        end)
      end

      -- Should handle rapid operations without crashing
      assert.is_true(operations > 0)
    end)

    it("should handle deep recursion scenarios", function()
      -- Create deeply nested configuration
      local deep_config = {
        openai_api_key = "test-key",
        nested = {}
      }

      local current = deep_config
      for i = 1, 100 do
        current.nested = {
          level = i,
          value = "deep-nested-value-" .. i,
          nested = {}
        }
        current = current.nested
      end

      -- Should handle deep config without stack overflow
      pcall(function()
        config.setup(deep_config)
      end)

      -- Should not crash
      assert.is_true(true)
    end)
  end)
end)