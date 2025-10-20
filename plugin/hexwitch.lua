-- Prevent double loading
if vim.g.loaded_hexwitch then
  return
end
vim.g.loaded_hexwitch = 1

-- Create commands (lazily load the plugin)
vim.api.nvim_create_user_command("Hexwitch", function(opts)
  local hexwitch = require("hexwitch")

  local args = opts.fargs
  if #args == 0 then
    -- No args: prompt for input
    hexwitch.prompt()
  elseif args[1] == "save" then
    -- :Hexwitch save <name>
    hexwitch.save(args[2])
  elseif args[1] == "load" then
    -- :Hexwitch load <name>
    hexwitch.load(args[2])
  else
    -- :Hexwitch <description...>
    hexwitch.generate(table.concat(args, " "))
  end
end, {
  nargs = "*",
  desc = "Generate AI-powered colorscheme",
  complete = function(arg_lead, cmdline, _)
    -- Subcommand completion
    if cmdline:match("^Hexwitch%s+%w*$") then
      return vim.tbl_filter(function(cmd)
        return cmd:find(arg_lead) == 1
      end, { "save", "load" })
    end

    -- Saved theme completion for 'load' subcommand
    if cmdline:match("^Hexwitch%s+load%s+") then
      local config = require("hexwitch.config")
      local themes_dir = config.get().themes_dir
      local themes = vim.fn.glob(themes_dir .. "/*.json", false, true)
      return vim.tbl_map(function(path)
        return vim.fn.fnamemodify(path, ":t:r")
      end, themes)
    end
  end,
})

-- Optional: Create <Plug> mappings for users who prefer that
vim.keymap.set("n", "<Plug>(HexwitchPrompt)", function()
  require("hexwitch").prompt()
end, { desc = "Hexwitch: Prompt for theme" })

