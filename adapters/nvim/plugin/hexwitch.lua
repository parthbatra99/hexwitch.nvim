if vim.g.loaded_hexwitch then return end
vim.g.loaded_hexwitch = true

local function api() return require("hexwitch") end
local function command(name, fn, opts)
  vim.api.nvim_create_user_command(name, fn, opts or {})
end

command("Hexwitch", function(o)
  if o.args == "" then api().prompt() else api().refine(o.args) end
end, { nargs = "*", desc = "Refine the active Hexwitch theme" })
command("HexwitchReference", function(o) api().reference(o.args) end,
  { nargs = 1, complete = "file", desc = "Load an IR or VS Code JSON reference" })
command("HexwitchSpec", function(o) api().refine("Edit spec", { spec = o.args }) end,
  { nargs = 1, complete = "file", desc = "Prepare a revision from a reviewed EditSpec" })
command("HexwitchReplay", function(o) api().refine("Offline replay", { from_llm = o.args }) end,
  { nargs = 1, complete = "file", desc = "Interpret a saved model response offline" })
command("HexwitchApply", function(o) api().apply(o.args ~= "" and o.args or nil) end,
  { nargs = "?", desc = "Accept a pending or saved version" })
command("HexwitchDiscard", function() api().discard() end)
command("HexwitchUndo", function() api().undo() end)
command("HexwitchRedo", function() api().redo() end)
command("HexwitchStatus", function() api().status() end)
command("HexwitchAnalyze", function() api().analyze() end)
