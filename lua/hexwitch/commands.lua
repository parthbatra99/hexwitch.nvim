local config = require("hexwitch.config")
local state = require("hexwitch.storage.state")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Core command implementations

-- Generate theme from description
---@param args table Command arguments
function M.generate(args)
  local input = table.concat(args, " ")
  if input == "" then
    require("hexwitch").prompt()
  else
    logger.info("commands.generate", "Generating theme from command line", { input = input })
    require("hexwitch").generate(input)
  end
end

-- Generate random theme
function M.generate_random()
  local random_prompts = {
    "surprise me with creative colors",
    "generate a unique theme with unexpected colors",
    "create an innovative color palette",
    "design a theme with bold color choices",
    "make something completely different",
  }

  local random_prompt = random_prompts[math.random(#random_prompts)]

  logger.info("commands.generate_random", "Generating random theme",
    { prompt = random_prompt })

  require("hexwitch").generate(random_prompt)
end

-- Refine the current theme via AI (re-prompts the model with the change)
function M.refine_theme(args)
  local input = table.concat(args, " ")
  if input == "" then
    require("hexwitch.ui.telescope.input").show_refinement_input(function(change)
      require("hexwitch").refine(change)
    end)
  else
    logger.info("commands.refine_theme", "Refining theme via AI", { input = input })
    require("hexwitch").refine(input)
  end
end

-- Save current theme
---@param args table Command arguments
function M.save_theme(args)
  local theme_name = table.concat(args, " ")
  if theme_name == "" then
    require("hexwitch.ui.telescope.input").show_save_dialog(nil, function(name)
      require("hexwitch").save(name)
    end)
  else
    logger.info("commands.save_theme", "Saving theme from command line",
      { theme_name = theme_name })
    require("hexwitch").save(theme_name)
  end
end

-- Load saved theme
---@param args table Command arguments
function M.load_theme(args)
  local theme_name = table.concat(args, " ")
  if theme_name == "" then
    require("hexwitch.ui.telescope").browse_themes()
  else
    logger.info("commands.load_theme", "Loading theme from command line",
      { theme_name = theme_name })
    require("hexwitch").load(theme_name)
  end
end

-- Delete saved theme
---@param args table Command arguments
function M.delete_theme(args)
  local theme_name = table.concat(args, " ")
  if theme_name == "" then
    vim.notify("Please provide a theme name to delete", vim.log.levels.ERROR)
    return
  end

  logger.info("commands.delete_theme", "Deleting theme", { theme_name = theme_name })
  require("hexwitch.theme.storage").delete(theme_name)
end

-- Browse saved themes directly
function M.browse_themes()
  logger.info("commands", "browse_themes", "Opening saved themes browser")
  require("hexwitch.ui.telescope").browse_themes()
end

-- Show generation history
function M.show_history()
  logger.info("commands", "show_history", "Showing generation history")
  require("hexwitch.ui.telescope").show_history()
end

-- Clear history
function M.clear_history()
  logger.info("commands", "clear_history", "Clearing generation history")
  state.clear_history()
  vim.notify("Generation history cleared", vim.log.levels.INFO)
end

-- Undo last theme
function M.undo_theme()
  logger.info("commands", "undo_theme", "Undoing last theme")
  require("hexwitch").undo()
end

-- Redo theme
function M.redo_theme()
  logger.info("commands", "redo_theme", "Redoing theme")
  require("hexwitch").redo()
end

-- Show plugin status
function M.show_status()
  logger.info("commands", "show_status", "Showing plugin status")
  require("hexwitch.ui.telescope").show_status()
end

-- Show debug logs
function M.show_logs()
  logger.info("commands", "show_logs", "Showing debug logs")
  logger.show_recent_logs()
end

-- Utility commands

-- Export theme
---@param args table Command arguments
function M.export_theme(args)
  local theme_name = table.concat(args, " ")
  if theme_name == "" then
    vim.notify("Please provide a theme name to export", vim.log.levels.ERROR)
    return
  end

  logger.info("commands.export_theme", "Exporting theme", { theme_name = theme_name })

  -- Read theme file
  local theme_path = config.get().themes_dir .. "/" .. theme_name .. ".json"
  local file = io.open(theme_path, "r")
  if not file then
    vim.notify("Theme not found: " .. theme_name, vim.log.levels.ERROR)
    return
  end

  local content = file:read("*all")
  file:close()

  -- Copy to clipboard
  vim.fn.setreg("+", content)
  vim.notify("Theme '" .. theme_name .. "' copied to clipboard", vim.log.levels.INFO)
end

-- Import theme from clipboard
function M.import_theme()
  local content = vim.fn.getreg("+")
  if content == "" then
    vim.notify("Clipboard is empty", vim.log.levels.WARN)
    return
  end

  logger.info("commands.import_theme", "Importing theme from clipboard")

  local ok, theme_data = pcall(vim.json.decode, content)
  if not ok then
    vim.notify("Invalid theme data in clipboard", vim.log.levels.ERROR)
    return
  end

  -- Security: Validate theme data before applying
  local applier = require("hexwitch.theme.applier")
  local success = applier.apply(theme_data)
  if not success then
    vim.notify("Theme validation failed: clipboard contains invalid or malicious theme data", vim.log.levels.ERROR)
    return
  end

  vim.notify("Theme imported and applied successfully", vim.log.levels.INFO)
end

-- Show help
function M.show_help()
  logger.info("commands", "show_help", "Showing help")

  local help_lines = {
    "hexwitch.nvim Commands",
    "========================",
    "",
    "Core:",
    "  :Hexwitch [description]     Generate a theme from a description",
    "  :Hexwitch refine [change]   Refine the current theme via AI",
    "  :Hexwitch random            Generate a random theme",
    "",
    "Theme management:",
    "  :Hexwitch save [name]       Save the current theme",
    "  :Hexwitch load [name]       Load a saved theme",
    "  :Hexwitch browse            Browse saved themes",
    "  :Hexwitch delete <name>     Delete a saved theme",
    "  :Hexwitch export <name>     Copy a saved theme to the clipboard",
    "  :Hexwitch import            Apply a theme from the clipboard",
    "",
    "History & undo:",
    "  :Hexwitch history           Show generation history",
    "  :Hexwitch clear-history     Clear history",
    "  :Hexwitch undo              Undo last theme",
    "  :Hexwitch redo              Redo theme",
    "",
    "Diagnostics:",
    "  :Hexwitch status            Show plugin status",
    "  :Hexwitch logs              Show debug logs",
    "  :Hexwitch help              Show this help",
    "",
    "Configure your AI endpoint in setup():",
    "  OpenAI:      { base_url = 'https://api.openai.com/v1' }",
    "  OpenRouter:  { base_url = 'https://openrouter.ai/api/v1' }",
    "  Ollama:      { base_url = 'http://localhost:11434/v1', structured_output = false }",
    "",
    "For more information, see :help hexwitch",
  }

  -- Create floating window for help
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, help_lines)
  vim.api.nvim_buf_set_option(buf, "modifiable", false)
  vim.api.nvim_buf_set_option(buf, "filetype", "help")

  local width = 70
  local height = #help_lines + 2

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    col = math.floor((vim.fn.winwidth(0) - width) / 2),
    row = math.floor((vim.fn.winheight(0) - height) / 2),
    border = "rounded",
    title = " hexwitch.nvim Help ",
    title_pos = "center",
  })

  -- Keymaps
  local opts = { buffer = buf, silent = true }
  vim.keymap.set("n", "q", function() vim.api.nvim_win_close(win, true) end, opts)
  vim.keymap.set("n", "<Esc>", function() vim.api.nvim_win_close(win, true) end, opts)
end

return M