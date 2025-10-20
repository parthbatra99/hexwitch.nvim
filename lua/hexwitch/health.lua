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
  local valid, err = pcall(function()
    assert(cfg.openai_api_key and cfg.openai_api_key ~= "", "OpenAI API key not configured")
  end)

  if valid then
    health.ok("Configuration valid")
    health.ok("API key configured")
  else
    health.warn("Configuration issue: " .. (err or "unknown"), {
      "Set OPENAI_API_KEY environment variable",
      "Or configure in setup(): require('hexwitch').setup({ openai_api_key = 'sk-...' })",
    })
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

  -- Check telescope (optional)
  if cfg.ui_mode == "telescope" then
    if validate_utils.can_require("telescope") then
      health.ok("Telescope UI mode: telescope.nvim installed")
    else
      health.warn("UI mode set to 'telescope' but telescope.nvim not found", {
        "Install telescope.nvim or change ui_mode to 'input'",
      })
    end
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

