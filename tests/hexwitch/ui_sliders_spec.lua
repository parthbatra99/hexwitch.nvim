describe("hexwitch.ui.sliders", function()
  local sliders

  before_each(function()
    package.loaded["hexwitch.ui.sliders"] = nil
    sliders = require("hexwitch.ui.sliders")
  end)

  it("should create slider window with correct values", function()
    local theme = {
      background = "#1e1e2e",
      foreground = "#cdd6f4"
    }
    local buf = sliders.create_slider_window(theme)

    assert.is_not_nil(buf)
    assert.is_number(buf)
  end)

  it("should update theme based on slider adjustments", function()
    local theme = {
      background = "#1e1e2e",
      foreground = "#cdd6f4",
      comment = "#6c7086",
      red = "#f38ba8",
      green = "#a6e3a1"
    }
    local adjusted = sliders.apply_slider_adjustments(theme, {
      temperature = 20,
      contrast = 10,
      saturation = -15,
      brightness = 5
    })

    assert.is_not_nil(adjusted)
    -- The theme structure should be preserved and have the same keys
    assert.is_not_nil(adjusted.background)
    assert.is_not_nil(adjusted.foreground)
  end)
end)