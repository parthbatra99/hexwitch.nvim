local helpers = require("fixtures.helpers")
local storage = require("hexwitch.theme.storage")
local applier = require("hexwitch.theme.applier")
local config = require("hexwitch.config")

describe("hexwitch.theme.storage", function()
  local themes_dir

  before_each(function()
    themes_dir = vim.fn.tempname() .. "/themes"
    config.setup({ api_key = "k", themes_dir = themes_dir, save_themes = true })
    -- apply a theme so there is a "current theme" to save
    applier.apply(helpers.valid_theme)
  end)

  describe("save", function()
    it("persists the real palette (not a highlight re-derivation)", function()
      storage.save("my-theme")
      local path = themes_dir .. "/my-theme.json"
      assert.equals(1, vim.fn.filereadable(path))

      local saved = vim.json.decode(table.concat(vim.fn.readfile(path), "\n"))
      assert.equals("forest-moon", saved.name)
      assert.equals(helpers.valid_theme.colors.cyan, saved.colors.cyan)
      assert.is_not_nil(saved.terminal) -- terminal map round-trips
    end)

    it("errors when no theme is currently applied", function()
      -- clear current theme by loading fresh state with an empty undo stack
      require("hexwitch.storage.state").set_current_theme(nil)
      local msgs = {}
      local orig = require("hexwitch.utils.notify").error
      require("hexwitch.utils.notify").error = function(m) table.insert(msgs, m) end
      storage.save("nothing")
      require("hexwitch.utils.notify").error = orig
      assert.truthy(msgs[1] and msgs[1]:match("No theme is currently applied"))
    end)

    it("rejects path-traversal names", function()
      local msgs = {}
      local orig = require("hexwitch.utils.notify").error
      require("hexwitch.utils.notify").error = function(m) table.insert(msgs, m) end
      storage.save("../../etc/evil")
      require("hexwitch.utils.notify").error = orig
      assert.truthy(#msgs > 0)
      assert.is_false(1 == vim.fn.filereadable("/etc/evil.json"))
    end)
  end)

  describe("load", function()
    it("round-trips: save then load reapplies the exact theme", function()
      storage.save("roundtrip")
      vim.g.colors_name = nil
      storage.load("roundtrip")
      assert.equals("forest-moon", vim.g.colors_name)
      -- loaded colors match the saved palette exactly
      local normal = vim.api.nvim_get_hl(0, { name = "Normal" })
      assert.equals(tonumber(helpers.valid_theme.colors.fg:sub(2), 16), normal.fg)
    end)

    it("errors on a missing theme", function()
      local msgs = {}
      local orig = require("hexwitch.utils.notify").error
      require("hexwitch.utils.notify").error = function(m) table.insert(msgs, m) end
      storage.load("does-not-exist")
      require("hexwitch.utils.notify").error = orig
      assert.truthy(msgs[1] and msgs[1]:match("not found"))
    end)
  end)

  describe("list", function()
    it("lists saved themes and excludes the state.json file", function()
      storage.save("alpha")
      storage.save("beta")
      local themes = storage.list()
      assert.is_true(vim.tbl_contains(themes, "alpha"))
      assert.is_true(vim.tbl_contains(themes, "beta"))
      assert.is_false(vim.tbl_contains(themes, "state"), "state.json must not appear as a theme")
    end)
  end)

  describe("delete", function()
    it("removes a saved theme", function()
      storage.save("gone")
      assert.equals(1, vim.fn.filereadable(themes_dir .. "/gone.json"))
      storage.delete("gone")
      assert.equals(0, vim.fn.filereadable(themes_dir .. "/gone.json"))
    end)
  end)
end)
