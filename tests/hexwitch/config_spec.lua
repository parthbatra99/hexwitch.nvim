describe("hexwitch.config", function()
  local config

  before_each(function()
    package.loaded["hexwitch.config"] = nil
    config = require("hexwitch.config")
  end)

  it("has default configuration", function()
    assert.is_not_nil(config.get())
    assert.equals("https://api.openai.com/v1", config.get().base_url)
    assert.equals("gpt-4o-mini", config.get().model)
    assert.is_true(config.get().structured_output)
  end)

  it("merges user configuration", function()
    local success = config.setup({
      base_url = "https://openrouter.ai/api/v1",
      temperature = 1.5,
    })

    assert.is_true(success)
    assert.equals("https://openrouter.ai/api/v1", config.get().base_url)
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

  it("validates base_url", function()
    local success, err = config.setup({
      base_url = "not-a-valid-url",
    })

    assert.is_false(success)
    assert.is_not_nil(err)
    assert.is_true(err:find("base_url") ~= nil)
  end)
end)
