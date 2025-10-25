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
      -- UI Configuration
      ui_mode = "telescope",
      ui = {
        auto_preview = true,
        compact_mode = false,
        icons = true,
      },

      -- Theme Configuration
      save_themes = true,
      themes_dir = vim.fn.stdpath("data") .. "/hexwitch-test/themes",

      -- AI Configuration (keys read from env)
      ai_provider = "openrouter", -- or "openrouter"
      model = "google/gemini-2.5-flash", -- for openrouter

      -- Performance
      timeout = 30000,
      temperature = 0.7,
      contrast_threshold = 4.5,
    })

    -- Handy test keymaps
    vim.keymap.set("n", "<leader>hw", ":Hexwitch<CR>", { desc = "Hexwitch prompt" })
    vim.keymap.set("n", "<leader>hs", ":HexwitchStatus<CR>", { desc = "Hexwitch status" })
    vim.keymap.set("n", "<leader>hl", ":HexwitchLogs<CR>", { desc = "Hexwitch logs" })
    vim.keymap.set("n", "<leader>hb", ":HexwitchBrowse<CR>", { desc = "Browse saved themes" })
    vim.keymap.set("n", "<leader>hh", ":HexwitchHistory<CR>", { desc = "View generation history" })
    vim.keymap.set("n", "<leader>hq", function()
      require("hexwitch.ui").quick_actions()
    end, { desc = "Quick actions menu" })
    vim.keymap.set("n", "<leader>hp", function()
      require("hexwitch.ui.telescope").browse_presets()
    end, { desc = "Browse preset themes" })
    vim.keymap.set("n", "<leader>hu", ":HexwitchUndo<CR>", { desc = "Undo last theme" })
    vim.keymap.set("n", "<leader>hr", ":HexwitchRedo<CR>", { desc = "Redo theme" })
  end,
}

local local_dir = vim.env.HEXWITCH_PLUGIN_DIR
if local_dir and local_dir ~= "" then
  plugin_spec.dir = local_dir
else
  plugin_spec[1] = "parthbatra99/hexwitch.nvim"
end

require("lazy").setup({ plugin_spec })

