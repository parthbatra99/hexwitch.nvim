describe("hexwitch.config", function()
  local config

  before_each(function()
    package.loaded["hexwitch.config"] = nil
    config = require("hexwitch.config")
  end)

  it("has default configuration", function()
    assert.is_not_nil(config.get())
    assert.equals("input", config.get().ui_mode)
  end)

  it("merges user configuration", function()
    local success = config.setup({
      ui_mode = "telescope",
      temperature = 1.5,
    })

    assert.is_true(success)
    assert.equals("telescope", config.get().ui_mode)
    assert.equals(1.5, config.get().temperature)
  end)

  it("validates configuration", function()
    local success, err = config.setup({
      temperature = 5, -- Invalid: should be 0-2
    })

    assert.is_false(success)
    assert.is_not_nil(err)
    assert.is_true(err:find("temperature") ~= nil)
  end)

  it("validates ui_mode", function()
    local success, err = config.setup({
      ui_mode = "invalid",
    })

    assert.is_false(success)
    assert.is_not_nil(err)
  end)
end)

