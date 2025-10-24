local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local entry_display = require("telescope.pickers.entry_display")

local config = require("hexwitch.config")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Loading state management
local loading_state = {
  active = false,
  message = "",
  progress = 0,
  stages = {},
  current_stage = 1,
  timer = nil,
  picker = nil,
  spinner_index = 1,
  spinner_chars = { "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏" },
  start_time = nil,
}

-- Create loading display items with stages and progress
function M._create_loading_display_items()
  local items = {}

  -- Add main loading line
  table.insert(items, {
    value = "loading_main",
    display = function()
      local spinner = loading_state.spinner_chars[loading_state.spinner_index]
      return spinner .. " " .. loading_state.message
    end,
    ordinal = loading_state.message,
  })

  -- Add stages with progress indicators
  for i, stage in ipairs(loading_state.stages) do
    local status = ""
    if i < loading_state.current_stage then
      status = "✅"
    elseif i == loading_state.current_stage then
      status = "⏳"
    else
      status = "⏸️"
    end

    table.insert(items, {
      value = "stage_" .. i,
      display = function()
        return status .. " " .. stage
      end,
      ordinal = stage,
    })
  end

  -- Add progress bar
  if #loading_state.stages > 1 then
    table.insert(items, {
      value = "progress",
      display = function()
        local progress = math.floor((loading_state.current_stage - 1) / #loading_state.stages * 100)
        local bar_length = 20
        local filled = math.floor(progress / 100 * bar_length)
        local bar = string.rep("█", filled) .. string.rep("░", bar_length - filled)
        return string.format("Progress: [%s] %d%%", bar, progress)
      end,
      ordinal = "progress",
    })
  end

  -- Add elapsed time
  if loading_state.start_time then
    table.insert(items, {
      value = "timer",
      display = function()
        local elapsed = os.time() - loading_state.start_time
        return string.format("⏱️ Time elapsed: %ds", elapsed)
      end,
      ordinal = "timer",
    })
  end

  return items
end

-- Update loading animation with stages
function M._update_loading_animation()
  if not loading_state.active or not loading_state.picker then
    return
  end

  -- Update spinner
  loading_state.spinner_index = (loading_state.spinner_index % #loading_state.spinner_chars) + 1

  -- Refresh picker with updated display
  if loading_state.picker.refresh then
    loading_state.picker:refresh(finders.new_table({
      results = M._create_loading_display_items(),
    }), { reset_prompt = false })
  end
end

-- Show enhanced loading state with stages
---@param message string Loading message
---@param opts table Options including stages and progress
---@return table handle Handle for closing the loading state
function M.show_loading(message, opts)
  opts = opts or {}

  if loading_state.active then
    M.close_loading()
  end

  loading_state.active = true
  loading_state.message = message
  loading_state.progress = 0
  loading_state.current_stage = 1
  loading_state.stages = opts.stages or {
    "🤖 Connecting to AI provider...",
    "📝 Building theme prompt...",
    "🎨 Generating colors...",
    "✨ Finalizing theme...",
  }
  loading_state.spinner_index = 1
  loading_state.start_time = os.time()

  logger.info("ui.telescope.notifications", "show_loading", "Showing enhanced loading state", {
    message = message,
    stages = #loading_state.stages
  })

  local picker = pickers.new({}, {
    prompt_title = "✨ Generating Theme...",
    finder = finders.new_table({
      results = M._create_loading_display_items(),
    }),
    sorter = conf.generic_sorter({}),
    attach_mappings = function(prompt_bufnr, map)
      -- Allow closing with escape or q
      map("i", "<Esc>", function()
        actions.close(prompt_bufnr)
        M.close_loading()
      end)
      map("n", "q", function()
        actions.close(prompt_bufnr)
        M.close_loading()
      end)
      map("n", "<Esc>", function()
        actions.close(prompt_bufnr)
        M.close_loading()
      end)
      return true
    end,
  })

  loading_state.picker = picker
  picker:find()

  -- Start animation
  loading_state.timer = vim.loop.new_timer()
  loading_state.timer:start(100, 100, vim.schedule_wrap(M._update_loading_animation))

  return {
    close = M.close_loading,
    update_message = M.update_loading_message,
    update_stage = M.update_loading_stage,
    update_progress = M.update_progress,
  }
end

-- Close loading state
function M.close_loading()
  if loading_state.timer and not loading_state.timer:is_closing() then
    loading_state.timer:close()
    loading_state.timer = nil
  end

  if loading_state.picker then
    local bufnr = vim.api.nvim_win_get_buf(loading_state.picker.prompt_win)
    if vim.api.nvim_buf_is_valid(bufnr) then
      vim.api.nvim_win_close(loading_state.picker.prompt_win, true)
    end
    loading_state.picker = nil
  end

  loading_state.active = false
  loading_state.message = ""
end

-- Update loading message
---@param new_message string New loading message
function M.update_loading_message(new_message)
  loading_state.message = new_message
  M._update_loading_animation()
end

-- Update loading stage
---@param stage_index number Current stage index (1-based)
---@param stage_message string Optional new stage message
function M.update_loading_stage(stage_index, stage_message)
  loading_state.current_stage = math.max(1, math.min(stage_index, #loading_state.stages))
  if stage_message then
    loading_state.stages[loading_state.current_stage] = stage_message
  end
  M._update_loading_animation()
end

-- Update loading progress (0-100)
---@param progress number Progress percentage (0-100)
function M.update_progress(progress)
  if #loading_state.stages > 1 then
    local stage_index = math.floor(progress / 100 * #loading_state.stages) + 1
    loading_state.current_stage = math.max(1, math.min(stage_index, #loading_state.stages))
  end
  loading_state.progress = progress
  M._update_loading_animation()
end

-- Show enhanced success message with preview
---@param message string Success message
---@param theme_name string Theme name
---@param theme_data table Optional theme data for preview
---@param opts table Options
function M.show_success(message, theme_name, theme_data, opts)
  opts = opts or {}
  local cfg = config.get()

  logger.info("ui.telescope.notifications", "show_success", "Showing enhanced success message",
    { message = message, theme_name = theme_name })

  local success_items = {
    -- Success header
    {
      value = "header",
      display = function()
        return "🎉 " .. message
      end,
      ordinal = "success",
      is_header = true,
    },
    -- Theme info
    {
      value = "theme_info",
      display = function()
        return "🎨 Theme: " .. (theme_name or "unnamed")
      end,
      ordinal = "theme " .. (theme_name or ""),
    },
  }

  -- Add color preview if theme data is available
  if theme_data and theme_data.colors then
    table.insert(success_items, {
      value = "color_preview",
      display = function()
        local colors = {
          theme_data.colors.bg or "#000000",
          theme_data.colors.fg or "#ffffff",
          theme_data.colors.red or "#ff0000",
          theme_data.colors.green or "#00ff00",
          theme_data.colors.blue or "#0000ff",
        }
        return "🎨 Colors: " .. table.concat(colors, "  ")
      end,
      ordinal = "colors",
    })
  end

  -- Add actions
  local actions_list = {
    { name = "Love it!", value = "love", icon = "❤️", desc = "Mark as favorite" },
    { name = "Tweak it", value = "tweak", icon = "🔧", desc = "Refine the theme" },
    { name = "Save Theme", value = "save", icon = "💾", desc = "Save to collection" },
    { name = "Create Variant", value = "variant", icon = "🎭", desc = "Generate similar theme" },
    { name = "Undo", value = "undo", icon = "↩️", desc = "Revert to previous" },
    { name = "Close", value = "close", icon = "✖️", desc = "Dismiss" },
  }

  for _, action in ipairs(actions_list) do
    table.insert(success_items, {
      value = action.value,
      display = function()
        return action.icon .. " " .. action.name
      end,
      ordinal = action.name,
      description = action.desc,
    })
  end

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 4 }, -- icon
      { remaining = true }, -- content
    },
  })

  pickers.new(opts, {
    prompt_title = "✨ Theme Generated Successfully",
    finder = finders.new_table({
      results = success_items,
      entry_maker = function(item)
        if item.is_header then
          return {
            value = item.value,
            display = function()
              return " " .. item.display()
            end,
            ordinal = item.display(),
            is_header = true,
          }
        end

        return {
          value = item.value,
          display = function()
            if item.value == "color_preview" then
              return item.display()
            else
              return displayer({
                { item.display():sub(1, 4), "Special" },
                { item.display():sub(5), "String" },
              })
            end
          end,
          ordinal = item.display(),
          item = item,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()

        if selection and not selection.is_header then
          actions.close(prompt_bufnr)
          M.handle_success_action(selection.item.value, theme_name, theme_data)
        end
      end)

      -- Add quick key mappings
      map("i", "<Esc>", function()
        actions.close(prompt_bufnr)
      end)
      map("n", "q", function()
        actions.close(prompt_bufnr)
      end)

      -- Auto-close after 10 seconds if no interaction
      local close_timer = vim.loop.new_timer()
      close_timer:start(10000, 0, vim.schedule_wrap(function()
        if vim.api.nvim_buf_is_valid(prompt_bufnr) then
          actions.close(prompt_bufnr)
        end
        close_timer:close()
      end))

      return true
    end,
  }):find()
end

-- Handle success action selection
---@param action string Action value
---@param theme_name string Theme name
---@param theme_data table Theme data for variant generation
function M.handle_success_action(action, theme_name, theme_data)
  logger.info("ui.telescope.notifications", "handle_success_action", "Executing success action",
    { action = action, theme_name = theme_name })

  if action == "love" then
    vim.notify("Theme loved! 💖", vim.log.levels.INFO)
  elseif action == "tweak" then
    require("hexwitch.ui.refinement").open()
  elseif action == "save" then
    require("hexwitch.ui.telescope.input").show_save_dialog(theme_name, function(name)
      require("hexwitch").save(name)
    end)
  elseif action == "variant" then
    -- Generate a variant of the current theme
    if theme_data and theme_data.description then
      local variant_prompt = "Create a variant of this theme with similar colors but different mood: " .. theme_data.description
      require("hexwitch").generate(variant_prompt)
    else
      require("hexwitch").generate("Create a variant of the current theme")
    end
  elseif action == "undo" then
    require("hexwitch").undo()
  elseif action == "close" then
    -- Just close
  end
end

-- Show error message as telescope picker
---@param message string Error message
---@param error_details string Detailed error information
---@param opts table Options
function M.show_error(message, error_details, opts)
  opts = opts or {}

  logger.warn("ui.telescope.notifications", "show_error", "Showing error message",
    { message = message, details = error_details })

  local actions_list = {
    { name = "Retry", value = "retry", icon = "🔄" },
    { name = "View Debug Logs", value = "debug", icon = "🐛" },
    { name = "Close", value = "close", icon = "✖️" },
  }

  local error_content = {
    "❌ " .. message,
    "",
  }

  if error_details then
    local max_line_length = 60
    local details = error_details
    while #details > max_line_length do
      table.insert(error_content, details:sub(1, max_line_length))
      details = details:sub(max_line_length + 1)
    end
    if #details > 0 then
      table.insert(error_content, details)
    end
  end

  table.insert(error_content, "")
  table.insert(error_content, "Choose an action:")

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- icon
      { remaining = true }, -- action name
    },
  })

  pickers.new(opts, {
    prompt_title = "⚠️ Error Occurred",
    finder = finders.new_table({
      results = actions_list,
      entry_maker = function(action)
        return {
          value = action.value,
          display = function()
            return displayer({
              { action.icon, "Special" },
              { action.name, "String" },
            })
          end,
          ordinal = action.name,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection then
          M.handle_error_action(selection.value, message, error_details)
        end
      end)

      map("i", "<Esc>", function()
        actions.close(prompt_bufnr)
      end)
      map("n", "q", function()
        actions.close(prompt_bufnr)
      end)

      return true
    end,
  }):find()
end

-- Handle error action selection
---@param action string Action value
---@param message string Original error message
---@param error_details string Error details
---@param opts table Options including retry function
function M.handle_error_action(action, message, error_details, opts)
  opts = opts or {}
  logger.info("ui.telescope.notifications", "handle_error_action", "Executing error action",
    { action = action, message = message })

  if action == "retry" then
    if opts.retry_function and type(opts.retry_function) == "function" then
      vim.notify("Retrying theme generation...", vim.log.levels.INFO)
      opts.retry_function()
    else
      vim.notify("Retry function not available. Please try again manually.", vim.log.levels.WARN)
    end
  elseif action == "config" then
    require("hexwitch.ui.telescope.status").show_status()
  elseif action == "debug" then
    logger.show_recent_logs()
  elseif action == "test" then
    vim.notify("Testing API connectivity...", vim.log.levels.INFO)
    require("hexwitch.ai").test_connectivity(nil, function(success, result)
      if success then
        vim.notify("✅ " .. result, vim.log.levels.INFO)
      else
        vim.notify("❌ " .. result, vim.log.levels.ERROR)
      end
    end)
  elseif action == "help" then
    vim.notify("Opening documentation...", vim.log.levels.INFO)
    -- Open GitHub docs or help
    vim.fn.system({ "open", "https://github.com/hexwitch/hexwitch.nvim" })
  elseif action == "close" then
    -- Just close
  end
end

-- Show warning message as telescope picker
---@param message string Warning message
---@param opts table Options
function M.show_warning(message, opts)
  opts = opts or {}

  logger.warn("ui.telescope.notifications", "show_warning", "Showing warning message", { message = message })

  local actions_list = {
    { name = "OK", value = "ok", icon = "✅" },
    { name = "View Details", value = "details", icon = "🔍" },
  }

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- icon
      { remaining = true }, -- action name
    },
  })

  pickers.new(opts, {
    prompt_title = "⚠️ Warning",
    finder = finders.new_table({
      results = actions_list,
      entry_maker = function(action)
        return {
          value = action.value,
          display = function()
            return displayer({
              { action.icon, "Special" },
              { action.name, "String" },
            })
          end,
          ordinal = action.name,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection then
          M.handle_warning_action(selection.value, message)
        end
      end)

      return true
    end,
  }):find()
end

-- Handle warning action selection
---@param action string Action value
---@param message string Original warning message
function M.handle_warning_action(action, message)
  logger.info("ui.telescope.notifications", "handle_warning_action", "Executing warning action",
    { action = action, message = message })

  if action == "ok" then
    -- Just close
  elseif action == "details" then
    logger.show_recent_logs()
  end
end

-- Show info message as telescope picker
---@param message string Info message
---@param opts table Options
function M.show_info(message, opts)
  opts = opts or {}

  logger.info("ui.telescope.notifications", "show_info", "Showing info message", { message = message })

  local actions_list = {
    { name = "OK", value = "ok", icon = "✅" },
  }

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- icon
      { remaining = true }, -- action name
    },
  })

  pickers.new(opts, {
    prompt_title = "ℹ️ Information",
    finder = finders.new_table({
      results = actions_list,
      entry_maker = function(action)
        return {
          value = action.value,
          display = function()
            return displayer({
              { action.icon, "Special" },
              { action.name, "String" },
            })
          end,
          ordinal = action.name,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
      end)

      -- Auto-close after 3 seconds
      local close_timer = vim.loop.new_timer()
      close_timer:start(3000, 0, vim.schedule_wrap(function()
        if vim.api.nvim_buf_is_valid(prompt_bufnr) then
          actions.close(prompt_bufnr)
        end
        close_timer:close()
      end))

      return true
    end,
  }):find()
end

-- Close all notification windows
function M.close_all()
  M.close_loading()
  -- Add other cleanup as needed
end

return M
