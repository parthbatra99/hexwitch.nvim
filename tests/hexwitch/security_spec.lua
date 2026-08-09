local helpers = require("fixtures.helpers")
local commands = require("hexwitch.commands")
local applier = require("hexwitch.theme.applier")
local system_utils = require("hexwitch.utils.system")

describe("hexwitch security", function()
  local original_clipboard

  before_each(function()
    original_clipboard = vim.fn.getreg("+")
    helpers.setup_config()
  end)

  after_each(function()
    vim.fn.setreg("+", original_clipboard)
  end)

  describe("clipboard import", function()
    it("applies a valid theme from the clipboard", function()
      vim.fn.setreg("+", vim.json.encode(helpers.valid_theme))
      vim.g.colors_name = nil
      commands.import_theme()
      assert.equals("forest-moon", vim.g.colors_name)
    end)

    it("rejects malicious theme data (e.g. shell injection in a color)", function()
      local malicious = vim.deepcopy(helpers.valid_theme)
      malicious.colors.bg = "rm -rf /"
      vim.fn.setreg("+", vim.json.encode(malicious))

      local current = vim.g.colors_name
      commands.import_theme()
      -- must NOT have applied the malicious theme
      assert.is_not_equals("rm -rf /", vim.g.colors_name)
    end)

    it("rejects non-JSON clipboard contents", function()
      vim.fn.setreg("+", "not json at all")
      local msgs = {}
      local orig_notify = vim.notify
      vim.notify = function(msg, level) table.insert(msgs, msg or "") end
      commands.import_theme()
      vim.notify = orig_notify
      assert.is_true(vim.tbl_contains(msgs, "Invalid theme data in clipboard"))
    end)
  end)

  describe("theme validation", function()
    it("rejects every color-invalid theme fixture", function()
      for _, theme in ipairs(helpers.invalid_color_themes) do
        assert.is_false(applier.apply(theme))
      end
    end)
  end)

  describe("system command guards", function()
    it("rejects shell-injection paths without executing them", function()
      local dangerous = {
        "rm -rf /",
        "; cat /etc/passwd",
        "$(whoami)",
        "`id`",
        "https://evil.com; rm -rf /",
      }
      for _, path in ipairs(dangerous) do
        assert.is_false(system_utils.open_path(path))
      end
    end)
  end)

  describe("name sanitization", function()
    local storage = require("hexwitch.theme.storage")
    after_each(function() vim.g.colors_name = nil end)

    it("rejects path-traversal theme names on save/load/delete", function()
      applier.apply(helpers.valid_theme)
      for _, name in ipairs({ "../escape", "..\\win", "a/b/c" }) do
        local msgs = {}
        local orig = require("hexwitch.utils.notify").error
        require("hexwitch.utils.notify").error = function(m) table.insert(msgs, m) end
        storage.save(name)
        storage.load(name)
        storage.delete(name)
        require("hexwitch.utils.notify").error = orig
        assert.truthy(#msgs > 0, "expected rejection for " .. name)
      end
    end)
  end)
end)
