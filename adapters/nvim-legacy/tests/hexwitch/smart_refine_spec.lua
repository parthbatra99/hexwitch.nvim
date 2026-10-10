describe("hexwitch.ai.smart_refine", function()
  local smart_refine

  before_each(function()
    package.loaded["hexwitch.ai.smart_refine"] = nil
    smart_refine = require("hexwitch.ai.smart_refine")
  end)

  it("should parse semantic refinement requests", function()
    local theme = require("tests.fixtures.mock_themes").dark_theme
    local result = smart_refine.parse_semantic_request("make it feel more like autumn sunset")

    assert.is_not_nil(result)
    assert.equals("warm", result.temperature_shift)
    assert.equals("medium", result.saturation_change)
    assert.is_not_nil(result.mood_analysis)
  end)

  it("should generate refinement suggestions", function()
    local theme = require("tests.fixtures.mock_themes").dark_theme
    local suggestions = smart_refine.generate_suggestions(theme)

    assert.is_table(suggestions)
    assert.is_true(#suggestions > 0)
    assert.is_not_nil(suggestions[1].description)
    assert.is_not_nil(suggestions[1].changes)
  end)
end)