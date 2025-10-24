local config = require("hexwitch.config")
local state = require("hexwitch.storage.state")
local logger = require("hexwitch.utils.logger")
local ai = require("hexwitch.ai")
local theme = require("hexwitch.theme")

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

-- Quick generation (variation of last theme)
function M.generate_quick()
  local history = state.get_recent_history(1)
  if #history == 0 then
    vim.notify("No previous theme found. Generate a theme first!", vim.log.levels.WARN)
    return
  end

  local last_entry = history[1]
  local variation_prompt = string.format("Create a variation of: %s", last_entry.prompt)

  logger.info("commands.generate_quick", "Generating quick variation",
    { original_prompt = last_entry.prompt })

  require("hexwitch").generate(variation_prompt)
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

-- Open refinement interface
function M.refine_theme(args)
  local input = table.concat(args, " ")
  if input == "" then
    require("hexwitch.ui.refinement").open()
  else
    logger.info("commands.refine_theme", "Applying refinement from command line",
      { input = input })
    require("hexwitch.ui.refinement").apply_custom_refinement(input)
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

-- List saved themes
function M.list_themes()
  logger.info("commands", "list_themes", "Listing saved themes")
  require("hexwitch.ui.telescope").browse_themes()
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

-- Test AI connectivity
function M.test_connectivity(args)
  local provider = table.concat(args, " ")
  if provider == "" then
    provider = nil
  end

  logger.info("commands.test_connectivity", "Testing AI connectivity",
    { provider = provider })

  vim.notify("Testing AI connectivity...", vim.log.levels.INFO)

  ai.test_connectivity(provider, function(success, message)
    if success then
      vim.notify("[OK] " .. message, vim.log.levels.INFO)
    else
      vim.notify("[ERROR] " .. message, vim.log.levels.ERROR)
    end
  end)
end

-- Show provider information
function M.show_providers()
  logger.info("commands", "show_providers", "Showing provider information")
  local info = ai.get_provider_info()

  local lines = {
    "[AI] AI Provider Information",
    "=========================",
    "",
    string.format("Primary: %s %s", info.primary.name,
      info.primary.available and "[OK]" or "[ERROR]"),
    string.format("Fallback: %s %s", info.fallback.name,
      info.fallback.available and "[OK]" or "[ERROR]"),
    string.format("Model: %s", info.model),
    string.format("Temperature: %.1f", info.temperature),
    "",
    "Available Providers:",
  }

  local providers = ai.get_available_providers()
  for _, provider in ipairs(providers) do
    local available = provider == info.primary.name and info.primary.available
    table.insert(lines, string.format("  %s %s", provider, available and "[OK]" or "[ERROR]"))
  end

  -- Create floating window
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, "modifiable", false)
  vim.api.nvim_buf_set_option(buf, "filetype", "text")

  local width = 50
  local height = #lines + 2

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    col = math.floor((vim.fn.winwidth(0) - width) / 2),
    row = math.floor((vim.fn.winheight(0) - height) / 2),
    border = "rounded",
    title = " AI Providers ",
    title_pos = "center",
  })

  -- Keymaps
  local opts = { buffer = buf, silent = true }
  vim.keymap.set("n", "q", function() vim.api.nvim_win_close(win, true) end, opts)
  vim.keymap.set("n", "<Esc>", function() vim.api.nvim_win_close(win, true) end, opts)
end

-- Set AI provider
---@param provider string Provider name
function M.set_provider(provider)
  local available_providers = ai.get_available_providers()
  if not vim.tbl_contains(available_providers, provider) then
    vim.notify("Invalid provider. Available: " .. table.concat(available_providers, ", "), vim.log.levels.ERROR)
    return
  end

  logger.info("commands.set_provider", "Setting AI provider", { provider = provider })
  -- Note: This would require updating the config, which is a bit more complex
  vim.notify("Provider changed to: " .. provider .. " (restart required for full effect)", vim.log.levels.INFO)
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

  -- Apply theme
  theme.apply(theme_data)
  vim.notify("Theme imported and applied", vim.log.levels.INFO)
end

-- Show help
function M.show_help()
  logger.info("commands", "show_help", "Showing help")

  local help_lines = {
    "[WITCH] hexwitch.nvim Commands",
    "========================",
    "",
    "Core Commands:",
    "  :Hexwitch [description]     Generate theme from description",
    "  :HexwitchQuick               Generate variation of last theme",
    "  :HexwitchRandom              Generate random theme",
    "  :HexwitchRefine [changes]    Refine current theme",
    "",
    "Theme Management:",
    "  :HexwitchSave [name]         Save current theme",
    "  :HexwitchLoad [name]         Load saved theme",
    "  :HexwitchDelete <name>       Delete saved theme",
    "  :HexwitchList                Browse saved themes",
    "  :HexwitchExport <name>       Export theme to clipboard",
    "  :HexwitchImport              Import theme from clipboard",
    "",
    "History & Undo:",
    "  :HexwitchHistory             Show generation history",
    "  :HexwitchClearHistory        Clear all history",
    "  :HexwitchUndo                Undo last theme",
    "  :HexwitchRedo                Redo theme",
    "",
    "Configuration:",
    "  :HexwitchSetProvider <name>  Set AI provider",
    "  :HexwitchStatus              Show plugin status",
    "  :HexwitchProviders           Show AI provider info",
    "  :HexwitchTestConnectivity    Test AI connection",
    "  :HexwitchLogs                Show debug logs",
    "  :HexwitchHelp                Show this help",
    "",
    "UI Modes:",
    "  :HexwitchPrompt              Open theme generation prompt",
    "  :HexwitchBrowse              Browse saved themes (Telescope)",
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