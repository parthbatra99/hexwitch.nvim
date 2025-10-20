local applier = require("hexwitch.theme.applier")
local storage = require("hexwitch.theme.storage")

describe("hexwitch.theme", function()
  local test_theme_data
  local temp_dir

  before_each(function()
    -- Create test theme data
    test_theme_data = {
      name = "test-theme",
      description = "A test theme for unit testing",
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
    }

    -- Create temporary directory for theme storage tests
    temp_dir = vim.fn.tempname()
    vim.fn.mkdir(temp_dir, "p")

    -- Mock vim.fn.stdpath to use our temp directory
    vim.fn.stdpath = function(type)
      if type == "data" then
        return temp_dir
      end
      return "/tmp/nvim"
    end
  end)

  after_each(function()
    -- Clean up temp directory
    if vim.fn.isdirectory(temp_dir) == 1 then
      vim.fn.delete(temp_dir, "rf")
    end

    -- Restore vim.g.colors_name
    vim.g.colors_name = nil

    -- Clear terminal colors
    for i = 0, 15 do
      vim.g["terminal_color_" .. i] = nil
    end
  end)

  describe("applier", function()
    it("should apply theme with valid data", function()
      applier.apply(test_theme_data)

      assert.equals("test-theme", vim.g.colors_name)

      -- Check some basic highlights were set
      local normal_hl = vim.api.nvim_get_hl(0, { name = "Normal" })
      assert.equals(test_theme_data.colors.fg, normal_hl.fg)
      assert.equals(test_theme_data.colors.bg, normal_hl.bg)

      local comment_hl = vim.api.nvim_get_hl(0, { name = "Comment" })
      assert.equals(test_theme_data.colors.comment, comment_hl.fg)
      assert.is_true(comment_hl.italic)

      -- Check terminal colors
      assert.equals(test_theme_data.colors.bg, vim.g.terminal_color_0)
      assert.equals(test_theme_data.colors.red, vim.g.terminal_color_1)
      assert.equals(test_theme_data.colors.green, vim.g.terminal_color_2)
      assert.equals(test_theme_data.colors.fg, vim.g.terminal_color_7)
    end)

    it("should handle theme without name", function()
      local theme_without_name = vim.deepcopy(test_theme_data)
      theme_without_name.name = nil

      applier.apply(theme_without_name)

      assert.equals("hexwitch", vim.g.colors_name)
    end)

    it("should handle empty theme name", function()
      local theme_with_empty_name = vim.deepcopy(test_theme_data)
      theme_with_empty_name.name = ""

      applier.apply(theme_with_empty_name)

      assert.equals("hexwitch", vim.g.colors_name)
    end)

    it("should handle invalid theme data", function()
      local invalid_themes = {
        nil,
        {},
        { colors = nil },
        { colors = {} },
        { name = "test", colors = {} }
      }

      for _, theme in ipairs(invalid_themes) do
        -- Capture notifications
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        applier.apply(theme)

        -- Should have received an error notification
        assert.is_true(#notifications > 0)
        assert.matches("Invalid colorscheme data", notifications[1])

        -- Restore original notify
        require("hexwitch.utils.notify").error = original_notify
      end
    end)

    it("should clear existing highlights before applying new ones", function()
      -- Set an existing colorscheme name
      vim.g.colors_name = "existing-theme"

      applier.apply(test_theme_data)

      -- Should have overridden the colorscheme name
      assert.equals("test-theme", vim.g.colors_name)
    end)

    it("should apply all expected highlight groups", function()
      applier.apply(test_theme_data)

      -- Test key highlight groups
      local expected_groups = {
        "Normal", "NormalFloat", "NormalSB", "StatusLine", "StatusLineNC",
        "Cursor", "CursorLine", "CursorLineNr", "Visual", "VisualNOS",
        "Comment", "Constant", "String", "Character", "Number", "Boolean", "Float",
        "Identifier", "Function", "Keyword", "Conditional", "Repeat", "Label",
        "Operator", "Exception", "PreProc", "Include", "Define", "Macro", "PreCondit",
        "Type", "StorageClass", "Structure", "Typedef", "Special", "SpecialChar",
        "Tag", "Delimiter", "SpecialComment", "Error", "Todo", "Underlined", "Ignore",
        "DiagnosticError", "DiagnosticWarn", "DiagnosticInfo", "DiagnosticHint",
        "DiagnosticUnderlineError", "DiagnosticUnderlineWarn", "DiagnosticUnderlineInfo", "DiagnosticUnderlineHint"
      }

      -- Test TreeSitter groups
      local treesitter_groups = {
        "@variable", "@variable.builtin", "@variable.parameter", "@variable.member",
        "@constant", "@constant.builtin", "@constant.macro", "@string", "@string.regex",
        "@string.escape", "@character", "@number", "@boolean", "@float", "@function",
        "@function.builtin", "@function.macro", "@operator", "@keyword", "@keyword.return",
        "@conditional", "@repeat", "@label", "@exception", "@type", "@type.builtin",
        "@type.definition", "@namespace", "@include", "@preproc", "@debug", "@tag",
        "@tag.attribute", "@tag.delimiter"
      }

      for _, group in ipairs(expected_groups) do
        local hl = vim.api.nvim_get_hl(0, { name = group })
        assert.is_not_nil(hl, "Highlight group " .. group .. " should be defined")
        assert.is_true(hl.fg ~= nil or hl.bg ~= nil or hl.italic ~= nil or hl.underline ~= nil,
          "Highlight group " .. group .. " should have some properties")
      end

      for _, group in ipairs(treesitter_groups) do
        local hl = vim.api.nvim_get_hl(0, { name = group })
        assert.is_not_nil(hl, "TreeSitter group " .. group .. " should be defined")
        assert.is_true(hl.fg ~= nil or hl.bg ~= nil or hl.italic ~= nil or hl.underline ~= nil,
          "TreeSitter group " .. group .. " should have some properties")
      end
    end)

    it("should set correct colors for specific groups", function()
      applier.apply(test_theme_data)

      -- Test specific color mappings
      local comment_hl = vim.api.nvim_get_hl(0, { name = "Comment" })
      assert.equals(test_theme_data.colors.comment, comment_hl.fg)
      assert.is_true(comment_hl.italic)

      local string_hl = vim.api.nvim_get_hl(0, { name = "String" })
      assert.equals(test_theme_data.colors.green, string_hl.fg)

      local keyword_hl = vim.api.nvim_get_hl(0, { name = "Keyword" })
      assert.equals(test_theme_data.colors.purple, keyword_hl.fg)

      local function_hl = vim.api.nvim_get_hl(0, { name = "Function" })
      assert.equals(test_theme_data.colors.blue, function_hl.fg)

      local diagnostic_error_hl = vim.api.nvim_get_hl(0, { name = "DiagnosticError" })
      assert.equals(test_theme_data.colors.red, diagnostic_error_hl.fg)

      local diagnostic_underline_hl = vim.api.nvim_get_hl(0, { name = "DiagnosticUnderlineError" })
      assert.equals(test_theme_data.colors.red, diagnostic_underline_hl.sp)
      assert.is_true(diagnostic_underline_hl.undercurl)
    end)

    it("should apply all terminal colors correctly", function()
      applier.apply(test_theme_data)

      local expected_terminal_colors = {
        [0] = test_theme_data.colors.bg,     -- black
        [1] = test_theme_data.colors.red,     -- red
        [2] = test_theme_data.colors.green,   -- green
        [3] = test_theme_data.colors.yellow,  -- yellow
        [4] = test_theme_data.colors.blue,    -- blue
        [5] = test_theme_data.colors.purple,  -- magenta
        [6] = test_theme_data.colors.cyan,    -- cyan
        [7] = test_theme_data.colors.fg,      -- white
        [8] = test_theme_data.colors.comment, -- bright black
        [9] = test_theme_data.colors.red,     -- bright red
        [10] = test_theme_data.colors.green,  -- bright green
        [11] = test_theme_data.colors.yellow, -- bright yellow
        [12] = test_theme_data.colors.blue,   -- bright blue
        [13] = test_theme_data.colors.purple, -- bright magenta
        [14] = test_theme_data.colors.cyan,   -- bright cyan
        [15] = test_theme_data.colors.fg      -- bright white
      }

      for i, expected_color in pairs(expected_terminal_colors) do
        assert.equals(expected_color, vim.g["terminal_color_" .. i],
          "Terminal color " .. i .. " should be set correctly")
      end
    end)
  end)

  describe("storage", function()
    describe("save", function()
      it("should save theme to file", function()
        -- First apply a theme so we have something to save
        applier.apply(test_theme_data)

        storage.save("saved-test-theme")

        -- Check file was created
        local theme_path = temp_dir .. "/hexwitch/saved-test-theme.json"
        assert.equals(1, vim.fn.filereadable(theme_path))

        -- Check file content
        local content = vim.fn.readfile(theme_path)
        local saved_data = vim.json.decode(table.concat(content, "\n"))

        assert.equals("test-theme", saved_data.name)
        assert.equals("Saved theme: saved-test-theme", saved_data.description)
        assert.is_not_nil(saved_data.colors)
      end)

      it("should handle empty theme name", function()
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.save("")

        assert.is_true(#notifications > 0)
        assert.matches("Theme name cannot be empty", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)

      it("should handle nil theme name", function()
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.save(nil)

        assert.is_true(#notifications > 0)
        assert.matches("Theme name cannot be empty", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)

      it("should handle file write errors", function()
        -- Create a directory where we want to write a file (causing error)
        local conflict_path = temp_dir .. "/hexwitch/saved-test-theme.json"
        vim.fn.mkdir(conflict_path, "p")

        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.save("saved-test-theme")

        assert.is_true(#notifications > 0)
        assert.matches("Failed to create theme file", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)
    end)

    describe("load", function()
      before_each(function()
        -- Create a test theme file
        applier.apply(test_theme_data)
        storage.save("test-load-theme")
      end)

      it("should load theme from file", function()
        -- Clear current theme first
        vim.g.colors_name = nil

        storage.load("test-load-theme")

        assert.equals("test-theme", vim.g.colors_name)
      end)

      it("should handle non-existent theme", function()
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.load("non-existent-theme")

        assert.is_true(#notifications > 0)
        assert.matches("Theme file not found", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)

      it("should handle empty theme name", function()
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.load("")

        assert.is_true(#notifications > 0)
        assert.matches("Theme name cannot be empty", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)

      it("should handle corrupted theme file", function()
        -- Create corrupted file
        local theme_path = temp_dir .. "/hexwitch/corrupted-theme.json"
        local file = io.open(theme_path, "w")
        file:write("invalid json content")
        file:close()

        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.load("corrupted-theme")

        assert.is_true(#notifications > 0)
        assert.matches("Failed to parse theme file", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)
    end)

    describe("list", function()
      before_each(function()
        -- Create some test theme files
        applier.apply(test_theme_data)
        storage.save("theme1")
        storage.save("theme2")
        storage.save("theme3")

        -- Create a non-json file to ensure it's ignored
        local readme_path = temp_dir .. "/hexwitch/README.txt"
        local file = io.open(readme_path, "w")
        file:write("This is not a theme file")
        file:close()
      end)

      it("should list all saved themes", function()
        local themes = storage.list()

        assert.is_true(#themes >= 3)
        assert.is_true(vim.tbl_contains(themes, "theme1"))
        assert.is_true(vim.tbl_contains(themes, "theme2"))
        assert.is_true(vim.tbl_contains(themes, "theme3"))
        assert.is_false(vim.tbl_contains(themes, "README")) -- Should ignore non-json files
      end)

      it("should return empty list when no themes exist", function()
        -- Remove all themes
        local theme_files = vim.fn.glob(temp_dir .. "/hexwitch/*.json", false, true)
        for _, file in ipairs(theme_files) do
          os.remove(file)
        end

        local themes = storage.list()
        assert.equals(0, #themes)
      end)

      it("should handle empty themes directory", function()
        -- Remove the themes directory
        vim.fn.delete(temp_dir .. "/hexwitch", "rf")

        local themes = storage.list()
        assert.equals(0, #themes)
      end)
    end)

    describe("delete", function()
      before_each(function()
        applier.apply(test_theme_data)
        storage.save("theme-to-delete")
      end)

      it("should delete existing theme", function()
        local theme_path = temp_dir .. "/hexwitch/theme-to-delete.json"
        assert.equals(1, vim.fn.filereadable(theme_path))

        storage.delete("theme-to-delete")

        assert.equals(0, vim.fn.filereadable(theme_path))
      end)

      it("should handle non-existent theme", function()
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.delete("non-existent-theme")

        assert.is_true(#notifications > 0)
        assert.matches("Failed to delete theme", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)

      it("should handle empty theme name", function()
        local notifications = {}
        local original_notify = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(msg)
          table.insert(notifications, msg)
        end

        storage.delete("")

        assert.is_true(#notifications > 0)
        assert.matches("Theme name cannot be empty", notifications[1])

        require("hexwitch.utils.notify").error = original_notify
      end)
    end)

    describe("integration", function()
      it("should save and load theme with same colors", function()
        -- Apply original theme
        applier.apply(test_theme_data)

        -- Save theme
        storage.save("integration-test")

        -- Clear theme
        vim.g.colors_name = nil
        vim.api.nvim_set_hl(0, "Normal", { fg = "#ffffff", bg = "#000000" })

        -- Load theme
        storage.load("integration-test")

        -- Verify theme was restored
        assert.equals("test-theme", vim.g.colors_name)

        local normal_hl = vim.api.nvim_get_hl(0, { name = "Normal" })
        assert.equals(test_theme_data.colors.fg, normal_hl.fg)
        assert.equals(test_theme_data.colors.bg, normal_hl.bg)
      end)

      it("should handle theme listing before and after operations", function()
        -- Initial state
        local initial_themes = storage.list()

        -- Save a theme
        storage.save("list-test-theme")
        local after_save_themes = storage.list()

        assert.equals(#initial_themes + 1, #after_save_themes)
        assert.is_true(vim.tbl_contains(after_save_themes, "list-test-theme"))

        -- Delete the theme
        storage.delete("list-test-theme")
        local after_delete_themes = storage.list()

        assert.equals(#initial_themes, #after_delete_themes)
        assert.is_false(vim.tbl_contains(after_delete_themes, "list-test-theme"))
      end)
    end)
  end)
end)