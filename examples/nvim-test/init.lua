-- Minimal NVIM_APPNAME config for testing hexwitch.nvim with Lazy
--
-- Usage:
--   NVIM_APPNAME=hexwitch-test nvim
-- Optional (to test a local checkout without installing):
--   HEXWITCH_PLUGIN_DIR=/path/to/hexwitch.nvim NVIM_APPNAME=hexwitch-test nvim

-- Leader and UI basics
vim.g.mapleader = " "
vim.opt.termguicolors = true

-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    "git", "clone", "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable", lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

-- Build plugin spec allowing local dir via HEXWITCH_PLUGIN_DIR
local plugin_spec = {
  name = "hexwitch.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "nvim-telescope/telescope.nvim",
  },
  config = function()
    require("hexwitch").setup({
      ui = {
        auto_preview = true,
        compact_mode = false,
        icons = true,
      },

      -- Theme Configuration
      save_themes = true,
      themes_dir = vim.fn.stdpath("data") .. "/hexwitch-test/themes",

      -- AI Configuration: one OpenAI-compatible client, pointed via base_url.
      -- Set the matching env var for whichever endpoint you use:
      --   OpenAI:     OPENAI_API_KEY      (base_url https://api.openai.com/v1)
      --   OpenRouter: OPENROUTER_API_KEY  (base_url https://openrouter.ai/api/v1)
      base_url = vim.env.HEXWITCH_BASE_URL or "https://openrouter.ai/api/v1",
      api_key = vim.env.OPENROUTER_API_KEY or vim.env.OPENAI_API_KEY or vim.env.HEXWITCH_API_KEY,
      model = vim.env.HEXWITCH_MODEL or "google/gemini-2.5-flash",
      -- temperature, timeout, structured output are locked plugin defaults
    })

    -- Handy test keymaps
    vim.keymap.set("n", "<leader>hw", ":Hexwitch<CR>", { desc = "Hexwitch prompt" })
    vim.keymap.set("n", "<leader>hf", ":HexwitchRefine<CR>", { desc = "Refine current theme" })
    vim.keymap.set("n", "<leader>hs", ":HexwitchStatus<CR>", { desc = "Hexwitch status" })
    vim.keymap.set("n", "<leader>hb", ":HexwitchBrowse<CR>", { desc = "Browse saved themes" })
    vim.keymap.set("n", "<leader>hh", ":HexwitchHistory<CR>", { desc = "View generation history" })
    vim.keymap.set("n", "<leader>hq", function()
      require("hexwitch.ui").quick_actions()
    end, { desc = "Quick actions menu" })
    vim.keymap.set("n", "<leader>hp", function()
      require("hexwitch.ui.telescope").browse_presets()
    end, { desc = "Browse preset themes" })
  end,
}

local local_dir = vim.env.HEXWITCH_PLUGIN_DIR
if local_dir and local_dir ~= "" then
  plugin_spec.dir = local_dir
else
  plugin_spec[1] = "parthbatra99/hexwitch.nvim"
end

require("lazy").setup({ plugin_spec })

