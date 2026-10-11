-- Load using scripts/nvim-standalone.sh; no user config or plugin manager.
local source = debug.getinfo(1, "S").source:gsub("^@", "")
local root = vim.fn.fnamemodify(source, ":p:h:h:h:h")
vim.opt.rtp:prepend(root .. "/adapters/nvim")
vim.cmd("runtime plugin/hexwitch.lua")
vim.o.termguicolors = true
local hx = require("hexwitch")
hx.setup({
  cmd = vim.env.HEXWITCH_BINARY or root .. "/core/target/release/hexwitch",
  base_url = vim.env.OPENAI_BASE_URL or "https://api.openai.com/v1",
  api_key_env = vim.env.HEXWITCH_API_KEY_ENV or "OPENAI_API_KEY",
  model = vim.env.HEXWITCH_MODEL or "gpt-4o-mini",
  response_format = vim.env.HEXWITCH_RESPONSE_FORMAT or "json_schema",
  interpretation = "v1",
  auto_apply = false,
})
if vim.fn.argc() == 0 then
  vim.api.nvim_buf_set_lines(0, 0, -1, false, {
    "-- Hexwitch standalone playground: no installed plugins or user config.",
    "-- Try :Hexwitch no italics anywhere", "-- Then :HexwitchStatus and :HexwitchApply",
    "-- :HexwitchUndo / :HexwitchRedo restore versions. :checkhealth hexwitch diagnoses setup.",
    "", "local function greet(name)", "  local message = 'Hello, ' .. name", "  return message", "end",
    "", "for i = 1, 3 do", "  print(greet('theme explorer'))", "end",
  })
  vim.bo.buftype = "nofile"
  vim.bo.bufhidden = "hide"
  vim.bo.swapfile = false
  vim.bo.filetype = "lua"
end
vim.cmd("syntax enable")
hx.reference(root .. "/core/eval/themes/gruvbox-dark-medium.json", function(err)
  if not err then
    vim.notify("Standalone ready. :Hexwitch <request> -> :HexwitchStatus -> :HexwitchApply", vim.log.levels.INFO)
  end
end)
