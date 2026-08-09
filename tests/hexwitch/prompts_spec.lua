local prompts = require("hexwitch.ai.prompts")

describe("hexwitch.ai.prompts", function()
  describe("SCHEMA (single source of truth)", function()
    it("targets a 16-color palette with hex patterns", function()
      assert.equals("HexwitchThemeSchema", prompts.SCHEMA_NAME)
      local color_props = prompts.SCHEMA.properties.colors.properties
      local expected = {
        "bg", "fg", "bg_sidebar", "bg_float", "bg_statusline",
        "red", "orange", "yellow", "green", "cyan", "blue", "purple", "magenta",
        "comment", "selection", "cursor",
      }
      for _, key in ipairs(expected) do
        assert.is_not_nil(color_props[key], "missing color " .. key)
        assert.equals("^#[0-9A-Fa-f]{6}$", color_props[key].pattern)
      end
      assert.equals(#expected, #prompts.SCHEMA.properties.colors.required)
    end)

    it("requires name, description, colors and nothing else", function()
      assert.is_true(vim.tbl_contains(prompts.SCHEMA.required, "name"))
      assert.is_true(vim.tbl_contains(prompts.SCHEMA.required, "colors"))
      assert.equals(false, prompts.SCHEMA.additionalProperties)
    end)
  end)

  describe("build_theme_prompt", function()
    it("embeds the user description", function()
      local p = prompts.build_theme_prompt("a misty mountain dawn")
      assert.truthy(p:match("misty mountain dawn"))
      -- instructs a JSON response with the 16-color structure
      assert.truthy(p:match('"bg"'))
      assert.truthy(p:match('"cursor"'))
    end)
  end)

  describe("build_refinement_prompt", function()
    it("embeds both the change and the current theme", function()
      local p = prompts.build_refinement_prompt(
        { name = "ocean", colors = { bg = "#101010", fg = "#eeeeee" } },
        "make it warmer and softer"
      )
      assert.truthy(p:match("make it warmer and softer"), "embeds the change")
      assert.truthy(p:match("101010"), "embeds the current palette")
      assert.truthy(p:match('"bg"'), "asks for a full colorscheme")
    end)
  end)

  describe("validate_accessibility", function()
    it("warns on pure black/white backgrounds", function()
      local t = { name = "x", colors = {
        bg = "#000000", fg = "#ffffff", comment = "#888888",
        red = "#ff0000", orange = "#ff8800", yellow = "#ffff00", green = "#00ff00",
        cyan = "#00ffff", blue = "#0000ff", purple = "#ff00ff", magenta = "#ff0088",
      } }
      local valid, warnings = prompts.validate_accessibility(t)
      assert.is_false(valid)
      assert.truthy(#warnings > 0)
    end)
  end)

  describe("presets", function()
    it("ships a non-empty preset list and a random picker", function()
      assert.is_true(#prompts.PRESETS > 0)
      assert.is_string(prompts.get_random_preset())
    end)
  end)
end)
