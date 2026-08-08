-- Prevent double loading
if vim.g.loaded_hexwitch then
  return
end
vim.g.loaded_hexwitch = 1

-- Load commands module
local commands = require("hexwitch.commands")

-- Main Hexwitch command with comprehensive subcommands
vim.api.nvim_create_user_command("Hexwitch", function(opts)
  local args = opts.fargs

  if #args == 0 then
    -- No args: show main prompt
    require("hexwitch").prompt()
    return
  end

  local subcommand = args[1]
  local remaining_args = {}
  for i = 2, #args do
    table.insert(remaining_args, args[i])
  end

  -- Route to appropriate command handler
  if subcommand == "save" then
    commands.save_theme(remaining_args)
  elseif subcommand == "load" then
    commands.load_theme(remaining_args)
  elseif subcommand == "delete" then
    commands.delete_theme(remaining_args)
  elseif subcommand == "history" then
    commands.show_history()
  elseif subcommand == "clear" or subcommand == "clear-history" then
    commands.clear_history()
  elseif subcommand == "undo" then
    commands.undo_theme()
  elseif subcommand == "redo" then
    commands.redo_theme()
  elseif subcommand == "refine" then
    commands.refine_theme(remaining_args)
  elseif subcommand == "random" then
    commands.generate_random()
  elseif subcommand == "browse" then
    commands.browse_themes()
  elseif subcommand == "export" then
    commands.export_theme(remaining_args)
  elseif subcommand == "import" then
    commands.import_theme()
  elseif subcommand == "status" then
    commands.show_status()
  elseif subcommand == "logs" then
    commands.show_logs()
  elseif subcommand == "help" then
    commands.show_help()
  else
    -- Treat as theme description
    commands.generate(args)
  end
end, {
  nargs = "*",
  desc = "Generate AI-powered colorscheme",
  complete = function(arg_lead, cmdline, _)
    -- Subcommand completion
    if cmdline:match("^Hexwitch%s+%w*$") then
      local subcommands = {
        "save", "load", "delete", "history", "clear-history",
        "undo", "redo", "refine", "random", "browse",
        "export", "import", "status", "logs", "help",
      }
      return vim.tbl_filter(function(cmd)
        return cmd:find(arg_lead) == 1
      end, subcommands)
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

    -- Saved theme completion for 'delete' subcommand
    if cmdline:match("^Hexwitch%s+delete%s+") then
      local config = require("hexwitch.config")
      local themes_dir = config.get().themes_dir
      local themes = vim.fn.glob(themes_dir .. "/*.json", false, true)
      return vim.tbl_map(function(path)
        return vim.fn.fnamemodify(path, ":t:r")
      end, themes)
    end
  end,
})

-- Additional convenience commands
vim.api.nvim_create_user_command("HexwitchRandom", function()
  commands.generate_random()
end, {
  desc = "Generate random theme"
})

vim.api.nvim_create_user_command("HexwitchRefine", function(opts)
  commands.refine_theme(opts.fargs)
end, {
  nargs = "*",
  desc = "Refine current theme"
})

vim.api.nvim_create_user_command("HexwitchHistory", function()
  commands.show_history()
end, {
  desc = "Show generation history"
})

vim.api.nvim_create_user_command("HexwitchUndo", function()
  commands.undo_theme()
end, {
  desc = "Undo last theme change"
})

vim.api.nvim_create_user_command("HexwitchRedo", function()
  commands.redo_theme()
end, {
  desc = "Redo theme change"
})

vim.api.nvim_create_user_command("HexwitchClearHistory", function()
  commands.clear_history()
end, {
  desc = "Clear generation history"
})

vim.api.nvim_create_user_command("HexwitchBrowse", function()
  commands.browse_themes()
end, {
  desc = "Browse saved themes"
})

vim.api.nvim_create_user_command("HexwitchStatus", function()
  commands.show_status()
end, {
  desc = "Show plugin status"
})

vim.api.nvim_create_user_command("HexwitchLogs", function()
  commands.show_logs()
end, {
  desc = "Show debug logs"
})

vim.api.nvim_create_user_command("HexwitchHelp", function()
  commands.show_help()
end, {
  desc = "Show hexwitch help"
})

-- Optional: Create <Plug> mappings for users who prefer that
vim.keymap.set("n", "<Plug>(HexwitchPrompt)", function()
  require("hexwitch").prompt()
end, { desc = "Hexwitch: Prompt for theme" })

vim.keymap.set("n", "<Plug>(HexwitchRandom)", function()
  commands.generate_random()
end, { desc = "Hexwitch: Random theme" })

vim.keymap.set("n", "<Plug>(HexwitchRefine)", function()
  commands.refine_theme()
end, { desc = "Hexwitch: Refine theme" })

vim.keymap.set("n", "<Plug>(HexwitchBrowse)", function()
  commands.browse_themes()
end, { desc = "Hexwitch: Browse themes" })

vim.keymap.set("n", "<Plug>(HexwitchHistory)", function()
  commands.show_history()
end, { desc = "Hexwitch: Show history" })

vim.keymap.set("n", "<Plug>(HexwitchUndo)", function()
  commands.undo_theme()
end, { desc = "Hexwitch: Undo theme" })

vim.keymap.set("n", "<Plug>(HexwitchRedo)", function()
  commands.redo_theme()
end, { desc = "Hexwitch: Redo theme" })

vim.keymap.set("n", "<Plug>(HexwitchStatus)", function()
  commands.show_status()
end, { desc = "Hexwitch: Show status" })

vim.keymap.set("n", "<Plug>(HexwitchLogs)", function()
  commands.show_logs()
end, { desc = "Hexwitch: Show logs" })

