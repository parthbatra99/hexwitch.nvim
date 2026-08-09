-- Minimal init for testing
local plenary_dir = vim.fn.expand("~/.local/share/nvim/lazy/plenary.nvim")
local telescope_dir = vim.fn.expand("~/.local/share/nvim/lazy/telescope.nvim")

vim.opt.rtp:append(".")
vim.opt.rtp:append(plenary_dir)
vim.opt.rtp:append(telescope_dir)

-- Allow `require("fixtures.xxx")` to resolve to tests/fixtures/xxx.lua
package.path = package.path .. ";./tests/?.lua;./tests/?/init.lua"

vim.cmd("runtime! plugin/plenary.vim")

