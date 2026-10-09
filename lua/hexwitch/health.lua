local M = {}

local health = vim.health
local config = require("hexwitch.config")
local validate_utils = require("hexwitch.utils.validate")

function M.check()
  health.start("hexwitch.nvim")

  -- Check Neovim version
  local nvim_version = vim.version()
  if nvim_version.major == 0 and nvim_version.minor < 9 then
    health.error("Neovim 0.9+ required, you have " .. vim.fn.execute("version"))
  else
    health.ok("Neovim version: " .. vim.version().major .. "." .. vim.version().minor)
  end

  -- Check plenary.nvim
  if validate_utils.can_require("plenary.curl") then
    health.ok("plenary.nvim installed")
  else
    health.error("plenary.nvim not found", {
      "Install with your plugin manager",
      "lazy.nvim: { 'nvim-lua/plenary.nvim' }",
    })
  end

  -- Check configuration
  local cfg = config.get()
  local ai = require("hexwitch.ai")
  local state = require("hexwitch.storage.state")

  -- Check AI endpoint configuration
  local provider_info = ai.get_provider_info()
  health.info(string.format("Endpoint: %s (model: %s)", provider_info.base_url, provider_info.model))
  if provider_info.available then
    health.ok("AI endpoint configured and ready")
  else
    local client = require("hexwitch.ai.client")
    local reason = not client.is_available() and "plenary.curl not found" or "API key not set"
    health.warn("AI endpoint not available: " .. reason, {
      "Set an API key: require('hexwitch').setup({ api_key = '...' }) or $OPENAI_API_KEY",
      "Point at any OpenAI-compatible endpoint via `base_url` (OpenAI, OpenRouter, Ollama, LM Studio)",
    })
  end

  -- Check state management
  local ok, current_state = pcall(state.get)
  if ok and current_state then
    health.ok("State management initialized")
  else
    health.error("State management failed to initialize")
  end

  -- Check themes directory
  if cfg.save_themes then
    if vim.fn.isdirectory(cfg.themes_dir) == 1 then
      local theme_count = #vim.fn.glob(cfg.themes_dir .. "/*.json", false, true)
      health.ok(string.format("Themes directory exists (%d saved themes)", theme_count))
    else
      health.warn("Themes directory doesn't exist: " .. cfg.themes_dir, {
        "Will be created automatically when saving themes",
      })
    end
  end

  -- Check telescope (required)
  if validate_utils.can_require("telescope") then
    health.ok("Telescope UI: telescope.nvim installed")
  else
    health.error("Telescope.nvim not found", {
      "Install telescope.nvim: { 'nvim-telescope/telescope.nvim' }",
    })
  end

  -- Check curl
  if validate_utils.command_exists("curl") then
    health.ok("curl available")
  else
    health.error("curl not found in PATH", {
      "Install curl: sudo apt install curl (Debian/Ubuntu)",
      "Or: brew install curl (macOS)",
    })
  end
end

return M

