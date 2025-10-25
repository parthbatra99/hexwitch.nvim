describe("hexwitch.config", function()
  local config

  before_each(function()
    package.loaded["hexwitch.config"] = nil
    config = require("hexwitch.config")
  end)

  it("has default configuration", function()
    assert.is_not_nil(config.get())
    assert.equals("openai", config.get().ai_provider)
  end)

  it("merges user configuration", function()
    local success = config.setup({
      ai_provider = "openrouter",
      temperature = 1.5,
    })

    assert.is_true(success)
    assert.equals("openrouter", config.get().ai_provider)
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

  it("validates ai_provider", function()
    local success, err = config.setup({
      ai_provider = "invalid",
    })

    assert.is_false(success)
    assert.is_not_nil(err)
  end)
end)

