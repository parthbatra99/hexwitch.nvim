local schema = require("hexwitch.config.schema")
-- Use the shared module instance (do NOT reload it — reloading creates a second
-- instance while other modules still hold the first, splitting the config state).
local config = require("hexwitch.config")

describe("hexwitch.config", function()
  before_each(function()
    -- Reset to pristine defaults on the shared instance
    config.setup(vim.deepcopy(schema.defaults))
  end)

  describe("defaults", function()
    it("ships a minimal, sensible default set", function()
      local c = config.get()
      assert.equals("https://api.openai.com/v1", c.base_url)
      assert.equals("gpt-4o-mini", c.model)
      assert.is_nil(c.max_tokens)
      assert.is_true(c.save_themes)
      assert.equals(50, c.max_history)
    end)

    it("no longer exposes locked options (temperature/timeout/structured_output/contrast_threshold)", function()
      local c = config.get()
      assert.is_nil(c.temperature)
      assert.is_nil(c.timeout)
      assert.is_nil(c.structured_output)
      assert.is_nil(c.contrast_threshold)
    end)
  end)

  describe("setup / merge", function()
    it("applies user overrides", function()
      config.setup({ base_url = "https://openrouter.ai/api/v1", max_history = 10 })
      local c = config.get()
      assert.equals("https://openrouter.ai/api/v1", c.base_url)
      assert.equals(10, c.max_history)
    end)

    it("merges incrementally over the current config (repeated setup is additive)", function()
      config.setup({ api_key = "first-key", model = "m1" })
      config.setup({ save_themes = false }) -- partial update
      local c = config.get()
      assert.equals("first-key", c.api_key) -- preserved
      assert.equals("m1", c.model)          -- preserved
      assert.is_false(c.save_themes)        -- updated
    end)

    it("resolves api_key from $OPENAI_API_KEY when none is set", function()
      local original = vim.env.OPENAI_API_KEY
      vim.env.OPENAI_API_KEY = "env-key-123"
      config.setup({ api_key = nil })
      assert.equals("env-key-123", config.get().api_key)
      vim.env.OPENAI_API_KEY = original
    end)

    it("does NOT overwrite an explicit api_key with the env value", function()
      local original = vim.env.OPENAI_API_KEY
      vim.env.OPENAI_API_KEY = "env-key-123"
      config.setup({ api_key = "explicit-key" })
      assert.equals("explicit-key", config.get().api_key)
      vim.env.OPENAI_API_KEY = original
    end)

    it("creates the themes directory when save_themes is true", function()
      local dir = vim.fn.tempname() .. "/themes"
      config.setup({ themes_dir = dir, save_themes = true })
      assert.equals(1, vim.fn.isdirectory(dir))
    end)
  end)

  describe("validation", function()
    it("rejects a base_url that is not http(s)", function()
      local ok, err = config.setup({ base_url = "ftp://nope" })
      assert.is_false(ok)
      assert.truthy(err:match("base_url"))
    end)

    it("rejects a non-number model", function()
      local ok = config.setup({ model = 123 })
      assert.is_false(ok)
    end)

    it("does not persist an invalid config (current config is unchanged)", function()
      config.setup({ model = "good-model" })
      config.setup({ base_url = "not-a-url" }) -- invalid
      assert.equals("good-model", config.get().model)
    end)
  end)
end)
