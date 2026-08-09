local helpers = require("fixtures.helpers")
local ai = require("hexwitch.ai")
local hexwitch = require("hexwitch")
local applier = require("hexwitch.theme.applier")
local state = require("hexwitch.storage.state")

describe("AI refine flow", function()
  before_each(function()
    vim.g.hexwitch_test_sync_mode = true
    helpers.setup_config()
    state.clear_history()
  end)

  after_each(function()
    vim.g.hexwitch_test_sync_mode = nil
  end)

  describe("ai.generate", function()
    it("returns the theme and records history", function()
      helpers.mock_curl(function() return helpers.completion(helpers.valid_theme) end)
      ai.generate("forest at full moon", function(data, err)
        assert.is_nil(err)
        assert.equals("forest-moon", data.name)
      end)
      local recent = state.get_recent_history(1)
      assert.is_true(#recent >= 1)
      assert.truthy(recent[1].prompt:match("forest at full moon"))
    end)

    it("rejects an empty description", function()
      ai.generate("", function(data, err)
        assert.is_nil(data)
        assert.truthy(err:match("cannot be empty"))
      end)
    end)
  end)

  describe("ai.refine", function()
    it("re-prompts with the current theme and applies the result", function()
      applier.apply(helpers.valid_theme) -- establish a current theme

      local captured, restore = helpers.mock_curl(function() return helpers.completion(helpers.refined_theme) end)
      ai.refine("make it warmer", function(data, err)
        assert.is_nil(err)
        assert.equals("forest-moon-warm", data.name)
      end)
      restore()

      -- the refinement request embedded the current theme + the change
      assert.truthy(captured[1].body.messages[2].content:match("make it warmer"))
      assert.truthy(captured[1].body.messages[2].content:match("forest%-moon"))

      -- (ai.refine is the AI+history layer; applying the result is the public
      --  hexwitch.refine layer's job — covered by the test below.)
    end)

    it("errors when there is no current theme to refine", function()
      state.set_current_theme(nil)
      ai.refine("warmer", function(data, err)
        assert.is_nil(data)
        assert.truthy(err:match("No theme to refine"))
      end)
    end)

    it("rejects an empty change", function()
      applier.apply(helpers.valid_theme)
      ai.refine("", function(data, err)
        assert.is_nil(data)
        assert.truthy(err:match("cannot be empty"))
      end)
    end)
  end)

  describe("public API forwards callbacks", function()
    it("hexwitch.generate applies the theme and invokes the callback", function()
      helpers.mock_curl(function() return helpers.completion(helpers.valid_theme) end)
      local done = false
      hexwitch.generate("forest at full moon", function(data, err)
        done = true
        assert.is_nil(err)
        assert.equals("forest-moon", data.name)
        assert.equals("forest-moon", vim.g.colors_name)
      end)
      assert.is_true(done)
    end)

    it("hexwitch.refine applies the refined theme and invokes the callback", function()
      applier.apply(helpers.valid_theme)
      helpers.mock_curl(function() return helpers.completion(helpers.refined_theme) end)
      local done = false
      hexwitch.refine("warmer", function(data, err)
        done = true
        assert.is_nil(err)
        assert.equals("forest-moon-warm", vim.g.colors_name)
      end)
      assert.is_true(done)
    end)
  end)

  describe("get_provider_info", function()
    it("returns flat endpoint info (no registry/fallback)", function()
      local info = ai.get_provider_info()
      assert.equals("https://api.openai.com/v1", info.base_url)
      assert.equals("gpt-4o-mini", info.model)
      assert.is_nil(info.provider) -- flat shape, no nested provider object
      assert.is_nil(info.fallback)
      assert.is_boolean(info.available)
    end)
  end)
end)
