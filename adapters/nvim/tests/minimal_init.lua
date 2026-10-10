-- Minimal init for testing
local plenary_dir = vim.fn.expand("~/.local/share/nvim/lazy/plenary.nvim")

vim.opt.rtp:append(".")
vim.opt.rtp:append(plenary_dir)

vim.cmd("runtime! plugin/plenary.vim")

