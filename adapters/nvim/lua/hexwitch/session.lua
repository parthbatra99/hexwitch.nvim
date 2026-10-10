-- Session state and artifact history. Theme documents are copied, never scraped.
local M = {
  versions = {}, history = {}, cursor = 0, pending = nil,
  busy = false, original = nil, root = nil,
}

function M.directory()
  if not M.root then
    local id = vim.fn.fnamemodify(vim.fn.tempname(), ":t")
    M.root = vim.fn.stdpath("data") .. "/hexwitch/sessions/" .. id
    vim.fn.mkdir(M.root, "p", "0700")
  end
  local dir = M.root .. "/" .. tostring(#M.versions + 1) .. "-" .. vim.fn.fnamemodify(vim.fn.tempname(), ":t")
  vim.fn.mkdir(dir, "p", "0700")
  return dir
end

function M.snapshot()
  local terminal = {}
  for i = 0, 15 do
    terminal[i] = vim.g["terminal_color_" .. i]
  end
  return {
    highlights = vim.api.nvim_get_hl(0, {}), terminal = terminal,
    background = vim.o.background, colors_name = vim.g.colors_name,
    termguicolors = vim.o.termguicolors,
  }
end

function M.restore(saved)
  vim.o.background = saved.background
  vim.cmd("highlight clear")
  for name, hl in pairs(saved.highlights) do
    vim.api.nvim_set_hl(0, name, hl)
  end
  for i = 0, 15 do
    vim.g["terminal_color_" .. i] = saved.terminal[i]
  end
  vim.g.colors_name = saved.colors_name
  vim.o.termguicolors = saved.termguicolors
end

-- Roll back partial loading if an artifact raises a Lua error.
function M.load(version)
  local saved = M.snapshot()
  local ok, err = pcall(function()
    vim.o.termguicolors = true
    -- Clear even when the original colorscheme did not set colors_name.
    vim.cmd("highlight clear")
    for i = 0, 15 do
      vim.g["terminal_color_" .. i] = nil
    end
    dofile(version.lua)
  end)
  if not ok then
    M.restore(saved)
    return nil, tostring(err)
  end
  if not M.original then
    M.original = saved
  end
  return true
end

function M.active()
  local id = M.history[M.cursor]
  return id and M.versions[id] or nil
end

function M.accept(version, is_reference)
  local ok, err = M.load(version)
  if not ok then
    return nil, err
  end
  if is_reference then
    M.history, M.cursor = {}, 0
  end
  for i = #M.history, M.cursor + 1, -1 do
    M.history[i] = nil
  end
  table.insert(M.history, version.id)
  M.cursor = #M.history
  M.pending = nil
  return version
end

return M
