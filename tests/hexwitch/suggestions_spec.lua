describe("hexwitch.ai.suggestions", function()
  local suggestions

  before_each(function()
    package.loaded["hexwitch.ai.suggestions"] = nil
    suggestions = require("hexwitch.ai.suggestions")
  end)

  it("should generate contextual theme suggestions", function()
    local theme = require("tests.fixtures.mock_themes").dark_theme
    local result = suggestions.generate_contextual_suggestions(theme, "more professional")

    assert.is_table(result)
    assert.is_true(#result >= 1)
    assert.is_not_nil(result[1].title)
    assert.is_not_nil(result[1].description)
    assert.is_not_nil(result[1].preview_changes)
  end)

  it("should suggest accessibility improvements", function()
    local theme = require("tests.fixtures.mock_themes").low_contrast_theme
    local improvements = suggestions.suggest_accessibility_improvements(theme)

    assert.is_table(improvements)
    -- At least one accessibility issue should be found with low contrast theme
    assert.is_true(#improvements > 0)
    assert.equals("accessibility", improvements[1].category)
  end)
end)