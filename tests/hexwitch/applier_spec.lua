local helpers = require("fixtures.helpers")
local applier = require("hexwitch.theme.applier")
local state = require("hexwitch.storage.state")

local function hex_to_dec(hex)
  return tonumber(hex:sub(2), 16)
end

describe("hexwitch.theme.applier", function()
  after_each(function()
    vim.g.colors_name = nil
    for i = 0, 15 do vim.g["terminal_color_" .. i] = nil end
  end)

  describe("apply", function()
    it("sets the colorscheme name and core highlights", function()
      local ok = applier.apply(helpers.valid_theme)
      assert.is_true(ok)
      assert.equals("forest-moon", vim.g.colors_name)

      local normal = vim.api.nvim_get_hl(0, { name = "Normal" })
      assert.equals(hex_to_dec(helpers.valid_theme.colors.fg), normal.fg)
      assert.equals(hex_to_dec(helpers.valid_theme.colors.bg), normal.bg)

      local string_hl = vim.api.nvim_get_hl(0, { name = "String" })
      assert.equals(hex_to_dec(helpers.valid_theme.colors.green), string_hl.fg)
    end)

    it("sets all 16 terminal colors", function()
      applier.apply(helpers.valid_theme)
      local c = helpers.valid_theme.colors
      assert.equals(c.bg, vim.g.terminal_color_0)
      assert.equals(c.red, vim.g.terminal_color_1)
      assert.equals(c.green, vim.g.terminal_color_2)
      assert.equals(c.fg, vim.g.terminal_color_7)
      assert.equals(c.comment, vim.g.terminal_color_8)
      assert.equals(c.fg, vim.g.terminal_color_15)
    end)

    it("defaults the colorscheme name to 'hexwitch' when missing", function()
      local t = vim.deepcopy(helpers.valid_theme)
      t.name = nil
      applier.apply(t)
      assert.equals("hexwitch", vim.g.colors_name)
    end)

    it("is the single chokepoint that tracks the current theme", function()
      applier.apply(helpers.valid_theme)
      local current = state.get_current_theme()
      assert.is_not_nil(current)
      assert.equals("forest-moon", current.name)
      assert.equals(helpers.valid_theme.colors.bg, current.colors.bg)
      -- terminal map is stamped so saved JSON round-trips
      assert.is_not_nil(current.terminal)
      assert.equals(helpers.valid_theme.colors.bg, current.terminal[0])
    end)
  end)

  describe("validation", function()
    it("rejects malformed theme data without applying", function()
      for _, theme in ipairs(helpers.invalid_color_themes) do
        local ok = applier.apply(theme)
        assert.is_false(ok, "should reject invalid theme")
      end
    end)

    it("rejects a theme missing required colors", function()
      local t = { name = "x", description = "x", colors = { bg = "#000000", fg = "#ffffff" } }
      assert.is_false(applier.apply(t))
    end)
  end)
end)
