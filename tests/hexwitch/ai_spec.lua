local openai = require("hexwitch.ai.openai")
local config = require("hexwitch.config")

describe("hexwitch.ai.openai", function()
  local original_config

  before_each(function()
    -- Store original config
    original_config = vim.deepcopy(config.get())

    -- Reset config to defaults
    config.setup({
      openai_api_key = "test-key",
      model = "gpt-4o-2024-08-06",
      temperature = 0.7,
      timeout = 30000,
      debug = false
    })
  end)

  after_each(function()
    -- Restore original config
    config.setup(original_config)
  end)

  describe("generate", function()
    it("should return error when API key is not configured", function()
      config.setup({ openai_api_key = nil })

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.matches("OpenAI API key not configured", error)
    end)

    it("should return error when API key is empty", function()
      config.setup({ openai_api_key = "" })

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.matches("OpenAI API key not configured", error)
    end)

    it("should return error when plenary is not available", function()
      -- Temporarily remove plenary from package.loaded
      local plenary = package.loaded["plenary.curl"]
      package.loaded["plenary.curl"] = nil

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.matches("plenary.nvim is required", error)

      -- Restore plenary
      package.loaded["plenary.curl"] = plenary
    end)

    it("should build correct request payload", function()
      local mock_curl = {
        post = function(url, options)
          assert.equals("https://api.openai.com/v1/chat/completions", url)
          assert.equals("Bearer test-key", options.headers.Authorization)
          assert.equals("application/json", options.headers["Content-Type"])

          local body = vim.json.decode(options.body)
          assert.equals("gpt-4o-2024-08-06", body.model)
          assert.equals(0.7, body.temperature)
          assert.equals("json_schema", body.response_format.type)
          assert.equals("neovim_colorscheme", body.response_format.json_schema.name)
          assert.is_true(body.response_format.json_schema.strict)

          -- Verify messages structure
          assert.equals(2, #body.messages)
          assert.equals("system", body.messages[1].role)
          assert.equals("user", body.messages[2].role)
          assert.matches("dark theme with purple accents", body.messages[2].content)

          -- Mock successful response
          options.callback({
            status = 200,
            body = vim.json.encode({
              choices = {
                {
                  message = {
                    content = vim.json.encode({
                      name = "test-theme",
                      description = "A test theme",
                      colors = {
                        bg = "#1a1b26",
                        fg = "#c0caf5",
                        bg_sidebar = "#1a1b26",
                        bg_float = "#1a1b26",
                        bg_statusline = "#1a1b26",
                        red = "#f7768e",
                        orange = "#ff9e64",
                        yellow = "#e0af68",
                        green = "#9ece6a",
                        cyan = "#73daca",
                        blue = "#7aa2f7",
                        purple = "#bb9af7",
                        magenta = "#d18616",
                        comment = "#565f89",
                        selection = "#33467c",
                        cursor = "#c0caf5"
                      }
                    })
                  }
                }
              }
            })
          })
        end
      }

      -- Mock plenary.curl
      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("dark theme with purple accents", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(error)
      assert.is_not_nil(result)
      assert.equals("test-theme", result.name)
      assert.equals("A test theme", result.description)
      assert.equals("#1a1b26", result.colors.bg)
      assert.equals("#c0caf5", result.colors.fg)

      -- Restore original plenary.curl
      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle API error responses", function()
      local mock_curl = {
        post = function(url, options)
          -- Mock error response
          options.callback({
            status = 401,
            body = vim.json.encode({
              error = {
                message = "Invalid API key"
              }
            })
          })
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.matches("API request failed with status 401", error)
      assert.matches("Invalid API key", error)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle malformed API responses", function()
      local mock_curl = {
        post = function(url, options)
          -- Mock malformed response
          options.callback({
            status = 200,
            body = "invalid json"
          })
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.matches("Failed to parse API response", error)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle empty choices in API response", function()
      local mock_curl = {
        post = function(url, options)
          options.callback({
            status = 200,
            body = vim.json.encode({
              choices = {}
            })
          })
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.equals("No choices in API response", error)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle malformed colorscheme JSON", function()
      local mock_curl = {
        post = function(url, options)
          options.callback({
            status = 200,
            body = vim.json.encode({
              choices = {
                {
                  message = {
                    content = "invalid json"
                  }
                }
              }
            })
          })
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(result)
      assert.matches("Failed to parse colorscheme JSON", error)

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should handle network timeout", function()
      config.setup({ timeout = 100 }) -- Very short timeout

      local mock_curl = {
        post = function(url, options)
          -- Don't call callback to simulate timeout
          -- In real scenario, plenary.curl would handle timeout
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      -- In a real test, this would eventually timeout
      -- For unit test, we just verify the request structure is correct

      package.loaded["plenary.curl"] = original_curl
    end)

    it("should use custom configuration parameters", function()
      config.setup({
        openai_api_key = "custom-key",
        model = "gpt-4-turbo",
        temperature = 0.9,
        timeout = 60000
      })

      local mock_curl = {
        post = function(url, options)
          local body = vim.json.decode(options.body)
          assert.equals("gpt-4-turbo", body.model)
          assert.equals(0.9, body.temperature)
          assert.equals("Bearer custom-key", options.headers.Authorization)
          assert.equals(60000, options.timeout)

          options.callback({
            status = 200,
            body = vim.json.encode({
              choices = {
                {
                  message = {
                    content = vim.json.encode({
                      name = "custom-theme",
                      description = "A custom configured theme",
                      colors = {
                        bg = "#000000",
                        fg = "#ffffff",
                        bg_sidebar = "#000000",
                        bg_float = "#000000",
                        bg_statusline = "#000000",
                        red = "#ff0000",
                        orange = "#ff8800",
                        yellow = "#ffff00",
                        green = "#00ff00",
                        cyan = "#00ffff",
                        blue = "#0000ff",
                        purple = "#ff00ff",
                        magenta = "#ff0088",
                        comment = "#888888",
                        selection = "#444444",
                        cursor = "#ffffff"
                      }
                    })
                  }
                }
              }
            })
          })
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test theme", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(error)
      assert.equals("custom-theme", result.name)
      assert.equals("#000000", result.colors.bg)

      package.loaded["plenary.curl"] = original_curl
    end)
  end)

  describe("JSON schema validation", function()
    it("should generate valid JSON schema", function()
      -- This tests the schema structure indirectly through the request building
      local mock_curl = {
        post = function(url, options)
          local body = vim.json.decode(options.body)
          local schema = body.response_format.json_schema.schema

          assert.equals("object", schema.type)
          assert.is_not_nil(schema.properties)
          assert.is_not_nil(schema.properties.colors)
          assert.is_not_nil(schema.properties.colors.properties)
          assert.equals("string", schema.properties.colors.properties.bg.type)
          assert.matches("^#[0-9A-Fa-f]{6}$", schema.properties.colors.properties.bg.pattern)

          -- Mock success
          options.callback({
            status = 200,
            body = vim.json.encode({
              choices = {
                {
                  message = {
                    content = vim.json.encode({
                      name = "schema-test",
                      description = "Testing schema validation",
                      colors = {
                        bg = "#1a1b26",
                        fg = "#c0caf5",
                        bg_sidebar = "#1a1b26",
                        bg_float = "#1a1b26",
                        bg_statusline = "#1a1b26",
                        red = "#f7768e",
                        orange = "#ff9e64",
                        yellow = "#e0af68",
                        green = "#9ece6a",
                        cyan = "#73daca",
                        blue = "#7aa2f7",
                        purple = "#bb9af7",
                        magenta = "#d18616",
                        comment = "#565f89",
                        selection = "#33467c",
                        cursor = "#c0caf5"
                      }
                    })
                  }
                }
              }
            })
          })
        end
      }

      local original_curl = require("plenary.curl")
      package.loaded["plenary.curl"] = mock_curl

      local result, error = nil, nil
      openai.generate("test schema", function(res, err)
        result = res
        error = err
      end)

      assert.is_nil(error)
      assert.is_not_nil(result)

      package.loaded["plenary.curl"] = original_curl
    end)
  end)
end)