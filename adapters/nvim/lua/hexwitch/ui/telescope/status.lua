local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local entry_display = require("telescope.pickers.entry_display")

local config = require("hexwitch.config")
local logger = require("hexwitch.utils.logger")
local system_utils = require("hexwitch.utils.system")

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

-- Show plugin status with actionable items
---@param opts table Options
function M.show_status(opts)
  opts = opts or {}

  logger.info("ui.telescope.status", "show_status", "Showing plugin status")

  local cfg = config.get()
  local ai_info = require("hexwitch.ai").get_provider_info()
  local stats = require("hexwitch.storage.state").get_stats()
  local stack_sizes = require("hexwitch.storage.state").get_stack_sizes()

  -- Create status sections with actions
  local status_items = {
    -- AI Configuration Section
    {
      category = "AI Configuration",
      items = {
        {
          name = "AI Provider",
          value = "provider",
          description = ai_info.provider.name,
          icon = ai_info.provider.available and "✓" or "✗",
          action = ai_info.provider.available and "change_provider" or "configure_provider",
        },
        {
          name = "Model",
          value = "model",
          description = cfg.model,
          icon = "M",
          action = "change_model",
        },
      }
    },
    -- Statistics Section
    {
      category = "Statistics",
      items = {
        {
          name = "Themes Generated",
          value = "generated_count",
          description = tostring(stats.commands_used.generate or 0),
          icon = "T",
          action = "view_history",
        },
        {
          name = "Themes Saved",
          value = "saved_count",
          description = tostring(stats.commands_used.save or 0),
          icon = "S",
          action = "browse_themes",
        },
        {
          name = "Avg Generation Time",
          value = "avg_time",
          description = string.format("%.1fms", stats.average_generation_time),
          icon = "⏱",
          action = "view_performance",
        },
      }
    },
    -- Storage Section
    {
      category = "Storage",
      items = {
        {
          name = "Themes Directory",
          value = "themes_dir",
          description = cfg.themes_dir,
          icon = "D",
          action = "open_themes_dir",
        },
        {
          name = "Undo Stack",
          value = "undo_stack",
          description = string.format("%d items", stack_sizes.undo),
          icon = "↶",
          action = "manage_undo",
        },
        {
          name = "Redo Stack",
          value = "redo_stack",
          description = string.format("%d items", stack_sizes.redo),
          icon = "↷",
          action = "manage_redo",
        },
      }
    },
    -- Settings Section
    {
      category = "Settings",
      items = {
        {
          name = "Debug Mode",
          value = "debug_mode",
          description = cfg.debug and "Enabled" or "Disabled",
          icon = cfg.debug and "●" or "○",
          action = "toggle_debug",
        },
      }
    },
  }

  -- Flatten all items for telescope
  local all_items = {}
  for _, section in ipairs(status_items) do
    -- Add section header
    table.insert(all_items, {
      value = "section_" .. section.category,
      display = section.category,
      ordinal = section.category,
      is_section = true,
    })

    -- Add section items
    for _, item in ipairs(section.items) do
      item.section = section.category
      table.insert(all_items, item)
    end
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { width = 20 }, -- name
      { width = 25 }, -- description
      { remaining = true }, -- action hint
    },
  })

  pickers.new(opts, {
    prompt_title = "Hexwitch Plugin Status",
    finder = finders.new_table({
      results = all_items,
      entry_maker = function(item)
        if item.is_section then
          return {
            value = item.value,
            display = function()
              return " " .. item.display
            end,
            ordinal = item.display,
            is_section = true,
          }
        end

        local action_hint = "Press <CR> to " .. item.action:gsub("_", " ")
        return {
          value = item.value,
          display = function()
            return displayer({
              { item.icon, "Special" },
              { item.name, "Function" },
              { item.description, "String" },
              { action_hint, "Comment" },
            })
          end,
          ordinal = item.name .. " " .. item.description,
          item = item,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()

        if selection and not selection.is_section then
          actions.close(prompt_bufnr)
          M.handle_status_action(selection.item)
        end
      end)

      -- Add key mappings for quick actions
      map("i", "<C-r>", function()
        actions.close(prompt_bufnr)
        M.refresh_status()
      end)
      map("n", "r", function()
        actions.close(prompt_bufnr)
        M.refresh_status()
      end)

      return true
    end,
  }):find()
end

-- Handle status action selection
---@param item table Selected status item
function M.handle_status_action(item)
  logger.info("ui.telescope.status", "handle_status_action", "Executing status action",
    { action = item.action, value = item.value })

  if item.action == "change_provider" or item.action == "configure_provider" then
    vim.notify("Configure providers in your setup file", vim.log.levels.INFO)
  elseif item.action == "change_model" then
    M.show_model_selection()
  elseif item.action == "view_history" then
    require("hexwitch.ui.telescope").show_history()
  elseif item.action == "browse_themes" then
    require("hexwitch.ui.telescope").browse_themes()
  elseif item.action == "view_performance" then
    M.show_performance_details()
  elseif item.action == "open_themes_dir" then
    local cfg = config.get()
    system_utils.open_path(cfg.themes_dir)
  elseif item.action == "manage_undo" then
    M.show_undo_stack()
  elseif item.action == "manage_redo" then
    M.show_redo_stack()
  elseif item.action == "toggle_debug" then
    M.toggle_debug_mode()
  end
end

-- Show model selection
---@param opts table Options
function M.show_model_selection(opts)
  opts = opts or {}

  local cfg = config.get()
  local ai_info = require("hexwitch.ai").get_provider_info()

  local models = {}
  if ai_info.provider.name == "OpenAI" then
    models = {
      "gpt-4o",
      "gpt-4o-mini",
      "gpt-4",
      "gpt-3.5-turbo",
    }
  elseif ai_info.provider.name == "OpenRouter" then
    models = {
      "anthropic/claude-3.5-sonnet",
      "anthropic/claude-3-haiku",
      "openai/gpt-4o",
      "openai/gpt-4o-mini",
      "google/gemini-pro",
    }
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- indicator
      { remaining = true }, -- model name
    },
  })

  pickers.new(opts, {
    prompt_title = "Select AI Model",
    finder = finders.new_table({
      results = models,
      entry_maker = function(model)
        local is_current = model == cfg.model
        return {
          value = model,
          display = function()
            return displayer({
              { is_current and "[>]" or "  ", "Special" },
              { model, is_current and "Function" or "String" },
            })
          end,
          ordinal = model,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection then
          vim.notify(string.format("Model changed to: %s\nAdd this to your config: require('hexwitch').setup({ model = '%s' })",
            selection.value, selection.value), vim.log.levels.INFO)
        end
      end)

      return true
    end,
  }):find()
end

-- Show performance details
---@param opts table Options
function M.show_performance_details(opts)
  opts = opts or {}

  local stats = require("hexwitch.storage.state").get_stats()
  local recent_history = require("hexwitch.storage.state").get_recent_history(10)

  local performance_items = {
    {
      name = "Average Generation Time",
      value = string.format("%.1fms", stats.average_generation_time),
      icon = "",
    },
    {
      name = "Total Generations",
      value = tostring(stats.commands_used.generate or 0),
      icon = "[THEME]",
    },
    {
      name = "Recent Activity",
      value = string.format("%d themes in last 10", #recent_history),
      icon = "[CHART]",
    },
  }

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { width = 25 }, -- name
      { remaining = true }, -- value
    },
  })

  pickers.new(opts, {
    prompt_title = "Performance Details",
    finder = finders.new_table({
      results = performance_items,
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
          ordinal = item.name,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
      end)

      return true
    end,
  }):find()
end

-- Show frequency selection
---@param opts table Options
function M.show_frequency_selection(opts)
  opts = opts or {}

  local cfg = config.get()
  local frequencies = {
    { name = "Manual", value = "manual", description = "Only when you request it" },
    { name = "Daily", value = "daily", description = "Fresh theme each morning" },
    { name = "Weekly", value = "weekly", description = "New theme every week" },
    { name = "Never", value = "never", description = "No automatic prompts" },
  }

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- indicator
      { width = 10 }, -- frequency
      { remaining = true }, -- description
    },
  })

  pickers.new(opts, {
    prompt_title = "Set Prompt Frequency",
    finder = finders.new_table({
      results = frequencies,
      entry_maker = function(freq)
        local is_current = freq.value == cfg.prompt_frequency
        return {
          value = freq.value,
          display = function()
            return displayer({
              { is_current and "👉" or "  ", "Special" },
              { freq.name, is_current and "Function" or "String" },
              { freq.description, "String" },
            })
          end,
          ordinal = freq.name,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection then
          require("hexwitch.storage.state").update_autoprompt(selection.value, os.date("%Y-%m-%d"))
          vim.notify(string.format("Prompt frequency set to: %s", selection.value), vim.log.levels.INFO)
        end
      end)

      return true
    end,
  }):find()
end

-- Toggle debug mode
function M.toggle_debug_mode()
  local cfg = config.get()
  local new_debug = not cfg.debug

  -- Update config (this would need to be persisted)
  -- For now, just show what would happen
  local status = new_debug and "enabled" or "disabled"
  vim.notify(string.format("Debug mode %s\nAdd this to your config: require('hexwitch').setup({ debug = %s })",
    status, tostring(new_debug)), vim.log.levels.INFO)
end

-- Show undo stack
---@param opts table Options
function M.show_undo_stack(opts)
  opts = opts or {}

  local state = require("hexwitch.storage.state")
  local current_state = state.get()
  local undo_stack = current_state.undo_stack

  if #undo_stack == 0 then
    vim.notify("No themes in undo stack", vim.log.levels.INFO)
    return
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { width = 20 }, -- theme name
      { remaining = true }, -- timestamp
    },
  })

  pickers.new(opts, {
    prompt_title = "Undo Stack",
    finder = finders.new_table({
      results = undo_stack,
      entry_maker = function(item, index)
        return {
          value = index,
          display = function()
            local timestamp = parse_iso8601(item.applied_at)
            local date_str = timestamp and os.date("%Y-%m-%d %H:%M", timestamp) or "N/A"
            return displayer({
              { "[UNDO]", "Special" },
              { item.theme.name or "unnamed", "Function" },
              { date_str, "String" },
            })
          end,
          ordinal = (item.theme.name or "") .. " " .. (item.applied_at or ""),
          item = item,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection then
          -- Undo specific number of times
          for _ = 1, selection.value do
            require("hexwitch").undo()
          end
          vim.notify(string.format("Undid %d theme(s)", selection.value), vim.log.levels.INFO)
        end
      end)

      return true
    end,
  }):find()
end

-- Show redo stack
---@param opts table Options
function M.show_redo_stack(opts)
  opts = opts or {}

  local state = require("hexwitch.storage.state")
  local current_state = state.get()
  local redo_stack = current_state.redo_stack

  if #redo_stack == 0 then
    vim.notify("No themes in redo stack", vim.log.levels.INFO)
    return
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { width = 20 }, -- theme name
      { remaining = true }, -- timestamp
    },
  })

  pickers.new(opts, {
    prompt_title = "Redo Stack",
    finder = finders.new_table({
      results = redo_stack,
      entry_maker = function(item, index)
        return {
          value = index,
          display = function()
            local timestamp = parse_iso8601(item.applied_at)
            local date_str = timestamp and os.date("%Y-%m-%d %H:%M", timestamp) or "N/A"
            return displayer({
              { "[REDO]", "Special" },
              { item.theme.name or "unnamed", "Function" },
              { date_str, "String" },
            })
          end,
          ordinal = (item.theme.name or "") .. " " .. (item.applied_at or ""),
          item = item,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection then
          -- Redo specific number of times
          for _ = 1, selection.value do
            require("hexwitch").redo()
          end
          vim.notify(string.format("Redid %d theme(s)", selection.value), vim.log.levels.INFO)
        end
      end)

      return true
    end,
  }):find()
end

-- Refresh status (placeholder for now)
function M.refresh_status()
  vim.notify("Status refreshed!", vim.log.levels.INFO)
  M.show_status()
end

return M