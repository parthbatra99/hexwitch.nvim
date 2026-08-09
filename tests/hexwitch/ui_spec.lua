-- Light UI spec: the telescope UI modules load and the facade exposes the
-- expected surface. (Full picker interaction is intentionally not exercised
-- headless — it's fragile and not the unit under test here.)
local ui = require("hexwitch.ui")

describe("hexwitch.ui", function()
  describe("facade", function()
    it("exposes the core UI entry points", function()
      for _, fn in ipairs({ "prompt", "browse_themes", "show_history", "show_loading", "show_success", "show_error", "show_warning", "show_info" }) do
        assert.is_function(ui[fn], "missing ui." .. fn)
      end
    end)

    it("no longer exposes the removed refinement/slider wrappers", function()
      assert.is_nil(ui.refine_theme)
      assert.is_nil(ui.open_sliders)
      assert.is_nil(ui.apply_slider_adjustments)
    end)

    it("reports telescope availability", function()
      assert.is_true(ui.is_available())
    end)
  end)

  describe("telescope modules load", function()
    it("loads the main telescope UI", function()
      local t = require("hexwitch.ui.telescope")
      assert.is_function(t.browse_themes)
      assert.is_function(t.show_history)
      assert.is_function(t.show_status)
    end)

    it("loads the input module with a refinement dialog", function()
      local input = require("hexwitch.ui.telescope.input")
      assert.is_function(input.show_examples)
      assert.is_function(input.show_save_dialog)
      assert.is_function(input.show_refinement_input) -- used by :Hexwitch refine
    end)

    it("loads the notifications module", function()
      local n = require("hexwitch.ui.telescope.notifications")
      assert.is_function(n.show_loading)
      assert.is_function(n.show_success)
      assert.is_function(n.show_error)
    end)

    it("loads the status module", function()
      local s = require("hexwitch.ui.telescope.status")
      assert.is_function(s.show_status)
    end)
  end)
end)
