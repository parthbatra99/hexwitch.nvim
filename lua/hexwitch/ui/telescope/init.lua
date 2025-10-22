local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local entry_display = require("telescope.pickers.entry_display")

local config = require("hexwitch.config")
local state = require("hexwitch.storage.state")
local storage = require("hexwitch.theme.storage")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Helper function to parse ISO 8601 date string to timestamp
local function parse_iso8601(date_str)
  if not date_str or date_str == "" then
    return nil
  end
  
  -- Parse ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
  local pattern = "(%d+)-(%d+)-(%d+)T(%d+):(%d+):(%d+)"
  local year, month, day, hour, min, sec = date_str:match(pattern)
  
  if not year then
    return nil
  end
  
  return os.time({
    year = tonumber(year),
    month = tonumber(month),
    day = tonumber(day),
    hour = tonumber(hour),
    min = tonumber(min),
    sec = tonumber(sec)
  })
end

-- Import our new telescope modules
local notifications = require("hexwitch.ui.telescope.notifications")
local input_module = require("hexwitch.ui.telescope.input")
local status_module = require("hexwitch.ui.telescope.status")

-- Expose the new telescope modules
M.notifications = notifications
M.input = input_module
M.status = status_module

-- Create displayer for theme entries
local function create_theme_displayer()
  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 20 }, -- theme name
      { remaining = true }, -- description/prompt
      { width = 15 }, -- date
    },
  })
  return displayer
end

-- Create enhanced theme entry with color preview
local function create_theme_entry_with_preview(theme_name, theme_data)
  local name = theme_name or "unnamed"
  local description = theme_data.description or "No description"
  local date = ""
  if theme_data.created_at then
    local timestamp = parse_iso8601(theme_data.created_at)
    date = timestamp and os.date("%m/%d %H:%M", timestamp) or ""
  end

  -- Create color preview string
  local preview = ""
  if theme_data.colors then
    local colors = { theme_data.colors.bg, theme_data.colors.fg, theme_data.colors.red, theme_data.colors.green, theme_data.colors.blue }
    preview = "  " .. table.concat(colors, "  ")
  end

  return {
    value = theme_name,
    display = function()
      local displayer = create_theme_displayer()
      return displayer({
        { name, "Function" },
        { description .. preview, "String" },
        { date, "Comment" },
      })
    end,
    ordinal = name .. " " .. description,
    theme_data = theme_data,
  }
end

-- Enhanced theme browser with preview
function M.browse_themes(opts)
  opts = opts or {}

  local themes = storage.list()
  if #themes == 0 then
    vim.notify("No saved themes found. Generate and save some themes first!", vim.log.levels.WARN)
    return
  end

  logger.info("ui.telescope", "browse_themes", "Opening enhanced theme browser", { theme_count = #themes })

  pickers.new(opts, {
    prompt_title = "🎨 Browse Saved Themes",
    finder = finders.new_table({
      results = themes,
      entry_maker = function(theme_name)
        -- Load theme data for display
        local theme_path = config.get().themes_dir .. "/" .. theme_name .. ".json"
        local file = io.open(theme_path, "r")
        local theme_data = {}

        if file then
          local content = file:read("*all")
          file:close()
          local ok, parsed = pcall(vim.json.decode, content)
          if ok then
            theme_data = parsed
          end
        end

        return create_theme_entry_with_preview(theme_name, theme_data)
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
        local selection = action_state.get_selected_entry()
        if selection then
          logger.info("ui.telescope", "browse_themes", "Loading selected theme",
            { theme_name = selection.value })
          require("hexwitch").load(selection.value)
        end
      end)

      -- Enhanced actions
      map("i", "<C-d>", function()
        local selection = action_state.get_selected_entry()
        if selection then
          vim.ui.input({
            prompt = "Delete theme '" .. selection.value .. "'? [y/N] ",
          }, function(input)
            if input and input:lower() == "y" then
              storage.delete(selection.value)
              actions.close(prompt_bufnr)
              M.browse_themes(opts)
            end
          end)
        end
      end)

      -- Preview theme
      map("i", "<C-p>", function()
        local selection = action_state.get_selected_entry()
        if selection and selection.theme_data then
          M.show_theme_preview(selection.theme_data)
        end
      end)

      return true
    end,
  }):find()
end

-- Show theme preview
---@param theme_data table Theme data
function M.show_theme_preview(theme_data)
  if not theme_data or not theme_data.colors then
    vim.notify("No theme data available for preview", vim.log.levels.WARN)
    return
  end

  local preview_items = {}

  -- Add theme info
  table.insert(preview_items, {
    name = "Theme Name",
    value = theme_data.name or "unnamed",
    icon = "🏷️",
  })

  table.insert(preview_items, {
    name = "Description",
    value = theme_data.description or "No description",
    icon = "📝",
  })

  table.insert(preview_items, {
    name = "Provider",
    value = theme_data.provider or "unknown",
    icon = "🤖",
  })

  table.insert(preview_items, {
    name = "Generated",
    value = theme_data.generated_at or "unknown",
    icon = "⏰",
  })

  -- Add colors
  table.insert(preview_items, { name = "", value = "", icon = "" }) -- separator
  table.insert(preview_items, { name = "Colors", value = "", icon = "🎨" })

  for color_name, color_value in pairs(theme_data.colors) do
    table.insert(preview_items, {
      name = color_name,
      value = color_value,
      icon = "●",
    })
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { width = 15 }, -- name
      { remaining = true }, -- value
    },
  })

  pickers.new({}, {
    prompt_title = "👁️ Theme Preview",
    finder = finders.new_table({
      results = preview_items,
      entry_maker = function(item)
        return {
          value = item.name,
          display = function()
            return displayer({
              { item.icon, "Special" },
              { item.name, "Function" },
              { item.value, "String" },
            })
          end,
          ordinal = item.name .. " " .. item.value,
        }
      end,
    }),
    sorter = conf.generic_sorter({}),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
      end)

      return true
    end,
  }):find()
end

-- Enhanced generation history with better preview
function M.show_history(opts)
  opts = opts or {}

  local history = state.get_recent_history(50)
  if #history == 0 then
    vim.notify("No generation history found", vim.log.levels.INFO)
    return
  end

  logger.info("ui.telescope", "show_history", "Opening enhanced generation history",
    { history_entries = #history })

  pickers.new(opts, {
    prompt_title = "📚 Generation History",
    finder = finders.new_table({
      results = history,
      entry_maker = function(entry)
        local name = entry.theme.name or "unnamed"
        local prompt = entry.prompt:sub(1, 60)
        if #entry.prompt > 60 then
          prompt = prompt .. "..."
        end
        local timestamp = parse_iso8601(entry.generated_at)
        local date = timestamp and os.date("%m/%d %H:%M", timestamp) or "N/A"
        local provider = entry.provider or "unknown"

        return {
          value = entry,
          display = function()
            local displayer = create_theme_displayer()
            return displayer({
              { name, "Function" },
              { prompt .. " [" .. provider .. "]", "String" },
              { date, "Comment" },
            })
          end,
          ordinal = entry.theme.name .. " " .. entry.prompt .. " " .. (entry.provider or ""),
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
        local selection = action_state.get_selected_entry()
        if selection then
          local entry = selection.value
          logger.info("ui.telescope", "show_history", "Applying theme from history",
            { theme_name = entry.theme.name, entry_id = entry.id })
          require("hexwitch.theme").apply(entry.theme)
          vim.notify("Applied theme: " .. (entry.theme.name or "unnamed"), vim.log.levels.INFO)
        end
      end)

      -- Add save mapping
      map("i", "<C-s>", function()
        local selection = action_state.get_selected_entry()
        if selection then
          local entry = selection.value
          vim.ui.input({
            prompt = "Save theme as: ",
            default = entry.theme.name,
          }, function(name)
            if name and name ~= "" then
              require("hexwitch").save(name)
              vim.notify("Theme saved as: " .. name, vim.log.levels.INFO)
            end
          end)
        end
      end)

      -- Preview theme
      map("i", "<C-p>", function()
        local selection = action_state.get_selected_entry()
        if selection then
          M.show_theme_preview(selection.value.theme)
        end
      end)

      return true
    end,
  }):find()
end

-- Enhanced preset browser with categories
function M.browse_presets(opts)
  opts = opts or {}

  -- Try to get presets from prompts module, fallback to built-in
  local presets = {}
  local ok, prompts = pcall(require, "hexwitch.ai.prompts")
  if ok and prompts.PRESETS then
    presets = prompts.PRESETS
  else
    -- Fallback presets with categories
    presets = {
      { name = "Dark cyberpunk with neon accents", category = "dark", description = "High contrast neon theme" },
      { name = "Calm ocean sunset with warm colors", category = "warm", description = "Peaceful evening colors" },
      { name = "Forest green with earthy tones", category = "nature", description = "Natural green palette" },
      { name = "Monochrome minimal high contrast", category = "minimal", description = "Clean black and white" },
      { name = "Tokyo night inspired vibrant purple and blue", category = "dark", description = "Japanese cityscape colors" },
      { name = "Dracula inspired deep purples", category = "dark", description = "Classic dark theme" },
      { name = "Solarized light warm and soft", category = "light", description = "Gentle eye-friendly theme" },
      { name = "Nord-like cool blues and grays", category = "cool", description = "Nordic minimal colors" },
      { name = "Gruvbox inspired warm retro", category = "warm", description = "Retro warm palette" },
      { name = "One Dark modern balanced", category = "modern", description = "Popular dark theme" },
    }
  end

  logger.info("ui.telescope", "browse_presets", "Opening enhanced preset browser", { preset_count = #presets })

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 8 }, -- category
      { remaining = true }, -- preset name
    },
  })

  pickers.new(opts, {
    prompt_title = "💡 Preset Theme Ideas",
    finder = finders.new_table({
      results = presets,
      entry_maker = function(preset)
        local name = type(preset) == "string" and preset or preset.name
        local category = type(preset) == "string" and "classic" or (preset.category or "general")
        local description = type(preset) == "string" and "" or (preset.description or "")

        return {
          value = name,
          display = function()
            return displayer({
              { "[" .. category .. "]", "Comment" },
              { name .. (description ~= "" and " - " .. description or ""), "String" },
            })
          end,
          ordinal = name .. " " .. category,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
        local selection = action_state.get_selected_entry()
        if selection then
          logger.info("ui.telescope", "browse_presets", "Selected preset theme",
            { preset = selection.value })
          require("hexwitch").generate(selection.value)
        end
      end)

      return true
    end,
  }):find()
end

-- Simple entry point to match README examples
function M.open_hexwitch(opts)
  return M.quick_actions(opts)
end

-- Enhanced quick actions with better organization
function M.quick_actions(opts)
  opts = opts or {}

  local action_groups = {
    {
      name = "Theme Generation",
      icon = "🎨",
      actions = {
        { name = "Generate New Theme", value = "generate", icon = "✨", description = "Create a new theme" },
        { name = "Browse Preset Ideas", value = "presets", icon = "💡", description = "Get inspiration from presets" },
        { name = "Generate Random Theme", value = "random", icon = "🎲", description = "Surprise me" },
        { name = "Refine Current Theme", value = "refine", icon = "🔧", description = "Tweak current theme" },
      }
    },
    {
      name = "Theme Management",
      icon = "📚",
      actions = {
        { name = "Browse Saved Themes", value = "browse", icon = "📖", description = "View your themes" },
        { name = "View Generation History", value = "history", icon = "📜", description = "Past generations" },
        { name = "Undo Last Theme", value = "undo", icon = "↩️", description = "Go back one theme" },
        { name = "Redo Theme", value = "redo", icon = "↪️", description = "Restore undone theme" },
      }
    },
    {
      name = "System",
      icon = "⚙️",
      actions = {
        { name = "Plugin Status", value = "status", icon = "📊", description = "Check system status" },
        { name = "Show Debug Logs", value = "logs", icon = "🐛", description = "View recent logs" },
      }
    },
  }

  -- Flatten actions with group info
  local all_actions = {}
  for _, group in ipairs(action_groups) do
    -- Add group header
    table.insert(all_actions, {
      value = "group_" .. group.name,
      display = group.icon .. " " .. group.name,
      ordinal = group.name,
      is_group = true,
    })

    -- Add group actions
    for _, action in ipairs(group.actions) do
      action.group = group.name
      table.insert(all_actions, action)
    end
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { width = 25 }, -- action name
      { remaining = true }, -- description
    },
  })

  pickers.new(opts, {
    prompt_title = "⚡ Hexwitch Quick Actions",
    finder = finders.new_table({
      results = all_actions,
      entry_maker = function(action)
        if action.is_group then
          return {
            value = action.value,
            display = function()
              return " " .. action.display
            end,
            ordinal = action.display,
            is_group = true,
          }
        end

        return {
          value = action.value,
          display = function()
            return displayer({
              { action.icon, "Special" },
              { action.name, "Function" },
              { action.description, "String" },
            })
          end,
          ordinal = action.name .. " " .. action.description,
          action = action,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()

        if selection and not selection.is_group then
          actions.close(prompt_bufnr)
          M.handle_quick_action(selection.action.value)
        end
      end)

      return true
    end,
  }):find()
end

-- Handle quick action selection
function M.handle_quick_action(action)
  logger.info("ui.telescope", "handle_quick_action", "Executing quick action", { action = action })

  if action == "generate" then
    input_module.show_examples()
  elseif action == "browse" then
    M.browse_themes()
  elseif action == "history" then
    M.show_history()
  elseif action == "presets" then
    M.browse_presets()
  elseif action == "random" then
    require("hexwitch").generate("surprise me with creative colors")
  elseif action == "refine" then
    require("hexwitch.ui.refinement").open()
  elseif action == "undo" then
    require("hexwitch").undo()
  elseif action == "redo" then
    require("hexwitch").redo()
  elseif action == "logs" then
    logger.show_recent_logs()
  elseif action == "status" then
    status_module.show_status()
  else
    vim.notify("Unknown action: " .. action, vim.log.levels.WARN)
  end
end

-- Enhanced plugin status with actionable items
function M.show_status()
  status_module.show_status()
end

return M
