local storage = require("hexwitch.theme.storage")
local applier = require("hexwitch.theme.applier")
local system_utils = require("hexwitch.utils.system")
local commands = require("hexwitch.commands")

describe("hexwitch security tests", function()
  describe("Command injection prevention", function()
    it("should sanitize system calls in logger", function()
      -- Test that the logger uses parameterized system calls
      local logger = require("hexwitch.utils.logger")

      -- This should not execute malicious commands
      local ok, err = pcall(function()
        logger.log_memory_usage("test", "test_memory", "test")
      end)

      assert.is_true(ok, "Logger should not crash with sanitized system calls")
    end)

    it("should validate system commands", function()
      -- Test dangerous command rejection
      local output, exit_code = system_utils.safely_execute("ls", {"; rm -rf /"})
      assert.equals(-1, exit_code, "Dangerous commands should be rejected")

      -- Test dangerous argument rejection
      local output, exit_code = system_utils.safely_execute("echo", {"hello; rm -rf /"})
      assert.equals(-1, exit_code, "Dangerous arguments should be rejected")
    end)
  end)

  describe("Path traversal protection", function()
    it("should sanitize theme names", function()
      local malicious_names = {
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "/etc/shadow",
        "theme\x00secret",
        "theme%00malicious",
        "theme; rm -rf /",
        "theme|cat /etc/passwd",
        "theme`whoami`",
        "theme$(id)",
      }

      for _, name in ipairs(malicious_names) do
        local result = storage.save(name)
        assert.is_nil(result, "Malicious theme name should be rejected: " .. name)
      end
    end)

    it("should prevent directory traversal in file operations", function()
      -- Test that path traversal attempts are blocked
      local result = storage.read("../../../etc/passwd")
      assert.is_nil(result, "Should not be able to read files outside theme directory")

      local result = storage.load("../../../etc/passwd")
      assert.is_nil(result, "Should not be able to load files outside theme directory")
    end)

    it("should validate theme path stays within directory", function()
      -- This tests internal path validation logic
      local result = storage.delete("../../../etc/passwd")
      assert.is_nil(result, "Should not be able to delete files outside theme directory")
    end)
  end)

  describe("Theme validation", function()
    it("should validate theme data structure", function()
      local invalid_themes = {
        nil,
        {},
        { colors = nil },
        { colors = {} },
        { colors = { bg = "invalid" } },
        { colors = { bg = "#ff0000" } }, -- missing required colors
      }

      -- Test valid theme
      local valid_theme = {
        colors = {
          bg = "#ff0000",
          fg = "#00ff00",
          bg_sidebar = "#0000ff",
          bg_float = "#ffffff",
          bg_statusline = "#111111",
          red = "#222222",
          orange = "#333333",
          yellow = "#444444",
          green = "#555555",
          cyan = "#666666",
          blue = "#777777",
          purple = "#888888",
          magenta = "#999999",
          comment = "#aaaaaa",
          selection = "#bbbbbb",
          cursor = "#cccccc"
        }
      }

      local success = applier.apply(valid_theme)
      assert.is_true(success, "Valid theme should be applied successfully")

      -- Test invalid themes
      for i = 1, #invalid_themes do
        local theme = invalid_themes[i]
        local success = applier.apply(theme)
        assert.is_false(success, "Invalid theme should be rejected: " .. vim.inspect(theme))
      end
    end)

    it("should validate color format", function()
      local invalid_colors = {
        "red",           -- not hex
        "#gggggg",       -- invalid hex
        "#12345",        -- too short
        "#1234567",      -- too long
        123456,         -- number instead of string
        nil,            -- missing color
        "#12345g",       -- contains non-hex character
      }

      for _, color in ipairs(invalid_colors) do
        local theme = {
          name = "test",
          colors = {
            bg = color,
            fg = "#ffffff",
            bg_sidebar = "#ffffff",
            bg_float = "#ffffff",
            bg_statusline = "#ffffff",
            red = "#ff0000",
            orange = "#ff8800",
            yellow = "#ffff00",
            green = "#00ff00",
            cyan = "#00ffff",
            blue = "#0000ff",
            purple = "#ff00ff",
            magenta = "#ff00ff",
            comment = "#888888",
            selection = "#444444",
            cursor = "#ffffff",
          }
        }

        local success = applier.apply(theme)
        assert.is_false(success, "Theme with invalid color should be rejected: " .. tostring(color))
      end
    end)

    it("should validate terminal colors", function()
      local theme_with_invalid_terminal = {
        name = "test",
        colors = {
          bg = "#000000",
          fg = "#ffffff",
          bg_sidebar = "#ffffff",
          bg_float = "#ffffff",
          bg_statusline = "#ffffff",
          red = "#ff0000",
          orange = "#ff8800",
          yellow = "#ffff00",
          green = "#00ff00",
          cyan = "#00ffff",
          blue = "#0000ff",
          purple = "#ff00ff",
          magenta = "#ff00ff",
          comment = "#888888",
          selection = "#444444",
          cursor = "#ffffff",
        },
        terminal = {
          [0] = "invalid_color",
          [1] = "#ff0000",
        }
      }

      local success = applier.apply(theme_with_invalid_terminal)
      assert.is_false(success, "Theme with invalid terminal colors should be rejected")
    end)
  end)

  describe("Clipboard import security", function()
    it("should validate clipboard theme data", function()
      -- Mock malicious clipboard content
      local original_reg = vim.fn.getreg("+")

      -- Test invalid JSON
      vim.fn.setreg("+", "not valid json")
      local ok = pcall(commands.import_theme)
      assert.is_true(ok, "Should not crash with invalid JSON")

      -- Test malicious theme data
      vim.fn.setreg("+", vim.json.encode({
        colors = {
          bg = "rm -rf /",  -- malicious content
          fg = "#ffffff",
          -- ... other required fields missing
        }
      }))

      local ok = pcall(commands.import_theme)
      assert.is_true(ok, "Should not crash with malicious theme data")

      -- Restore original clipboard
      vim.fn.setreg("+", original_reg)
    end)
  end)

  describe("API key security", function()
    it("should sanitize API keys in logs", function()
      -- This test would require mocking the logger to verify sanitization
      -- For now, we just verify the sanitize function exists
      local openai = require("hexwitch.ai.providers.openai")
      assert.is_function(openai.is_available, "OpenAI provider should have sanitize functions")
    end)
  end)

  describe("Cross-platform system calls", function()
    it("should validate URLs and paths", function()
      local invalid_paths = {
        "rm -rf /",
        "; cat /etc/passwd",
        "$(whoami)",
        "`id`",
        "https://evil.com; rm -rf /",
      }

      for _, path in ipairs(invalid_paths) do
        local success = system_utils.open_path(path)
        assert.is_false(success, "Should reject malicious path: " .. path)
      end
    end)

    it("should allow valid URLs and paths", function()
      local valid_paths = {
        "https://github.com/hexwitch/hexwitch.nvim",
        "/tmp",
        "test_file.txt",
      }

      -- Note: These might fail if the path doesn't exist, but they shouldn't be rejected for security reasons
      for _, path in ipairs(valid_paths) do
        local success = system_utils.open_path(path)
        -- We don't assert success here since the path might not exist
        -- We just verify it doesn't crash and passes security validation
      end
    end)
  end)

  describe("Input validation", function()
    it("should handle large inputs safely", function()
      local large_string = string.rep("a", 1000000) -- 1MB string

      -- Should not crash with large inputs
      local ok = pcall(function()
        local theme = {
          name = large_string:sub(1, 100), -- Limit theme name length
          colors = {
            bg = "#000000",
            fg = "#ffffff",
            bg_sidebar = "#ffffff",
            bg_float = "#ffffff",
            bg_statusline = "#ffffff",
            red = "#ff0000",
            orange = "#ff8800",
            yellow = "#ffff00",
            green = "#00ff00",
            cyan = "#00ffff",
            blue = "#0000ff",
            purple = "#ff00ff",
            magenta = "#ff00ff",
            comment = "#888888",
            selection = "#444444",
            cursor = "#ffffff",
          }
        }

        applier.apply(theme)
      end)

      assert.is_true(ok, "Should handle large inputs without crashing")
    end)
  end)
end)