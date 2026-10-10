local applier = require("hexwitch.theme.applier")
local storage = require("hexwitch.theme.storage")

local M = {}

---Apply a colorscheme from AI-generated data
---@param colorscheme_data hexwitch.ColorschemeData
function M.apply(colorscheme_data)
  applier.apply(colorscheme_data)
end

---Save the current theme
---@param theme_name string Name to save theme as
function M.save(theme_name)
  storage.save(theme_name)
end

---Load a saved theme
---@param theme_name string Name of saved theme to load
function M.load(theme_name)
  storage.load(theme_name)
end

return M