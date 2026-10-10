local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local entry_display = require("telescope.pickers.entry_display")

local config = require("hexwitch.config")
local logger = require("hexwitch.utils.logger")

local M = {}

-- Show theme examples as telescope picker
---@param opts table Options
function M.show_examples(opts)
  opts = opts or {}

  -- Check if prompts module exists, fallback to examples if not
  local examples = {}
  local ok, prompts = pcall(require, "hexwitch.ai.prompts")
  if ok and prompts.PRESETS then
    examples = prompts.PRESETS
  else
    -- Fallback examples
    examples = {
      "Dark cyberpunk with neon accents",
      "Calm ocean sunset with warm colors",
      "Forest green with earthy tones",
      "Monochrome minimal high contrast",
      "Tokyo night inspired vibrant purple and blue",
      "Dracula inspired deep purples",
      "Solarized light warm and soft",
      "Nord-like cool blues and grays",
      "Gruvbox inspired warm retro",
      "One Dark modern balanced",
    }
  end

  logger.info("ui.telescope.input", "show_examples", "Showing theme examples", { count = #examples })

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- number
      { remaining = true }, -- example text
    },
  })

  pickers.new(opts, {
    prompt_title = "Choose Theme Description or Type Your Own",
    finder = finders.new_table({
      results = examples,
      entry_maker = function(example)
        local index = ""
        for i, ex in ipairs(examples) do
          if ex == example then
            index = tostring(i)
            break
          end
        end
        return {
          value = example,
          display = function()
            return displayer({
              { index .. ".", "Number" },
              { example, "String" },
            })
          end,
          ordinal = example,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        local prompt = action_state.get_current_line()
        actions.close(prompt_bufnr)

        local theme_input = prompt ~= "" and prompt or (selection and selection.value)

        if theme_input and theme_input ~= "" then
          -- Check if input is a number
          local num = tonumber(theme_input)
          if num and num >= 1 and num <= #examples then
            -- Use the selected example
            local selected_example = examples[num]
            logger.info("ui.telescope.input", "show_examples", "Selected example theme",
              { example = selected_example })
            require("hexwitch").generate(selected_example)
          else
            -- Use the custom input or selected theme
            logger.info("ui.telescope.input", "show_examples", "Using theme description",
              { input = theme_input })
            require("hexwitch").generate(theme_input)
          end
        else
          vim.notify("Please provide a theme description", vim.log.levels.WARN)
        end
      end)

      return true
    end,
  }):find()
end

-- Show simple input prompt
---@param opts table Options
function M.show_simple_input(opts)
  opts = opts or {}

  logger.info("ui.telescope.input", "show_simple_input", "Showing simple input prompt")

  local examples = {
    "Dark cyberpunk with neon accents",
    "Calm ocean sunset with warm colors",
    "Forest green with earthy tones",
    "Minimal monochrome high contrast",
  }

  pickers.new(opts, {
    prompt_title = "Describe Your Desired Theme",
    finder = finders.new_table({
      results = {},
      entry_maker = function(input)
        return {
          value = input,
          display = input,
          ordinal = input,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local prompt = action_state.get_current_line()
        actions.close(prompt_bufnr)

        if prompt and prompt ~= "" then
          logger.info("ui.telescope.input", "show_simple_input", "Generating theme from user input",
            { input = prompt })
          require("hexwitch").generate(prompt)
        else
          vim.notify("Theme description cannot be empty", vim.log.levels.WARN)
        end
      end)

      -- Add example selection with tab
      map("i", "<Tab>", function()
        local current_examples = {}
        for _, example in ipairs(examples) do
          table.insert(current_examples, {
            value = example,
            display = "💡 " .. example,
            ordinal = example,
          })
        end

        -- Show examples picker
        pickers.new(opts, {
          prompt_title = "Theme Examples",
          finder = finders.new_table({
            results = current_examples,
          }),
          sorter = conf.generic_sorter(opts),
          attach_mappings = function(example_bufnr, map)
            actions.select_default:replace(function()
              local selection = action_state.get_selected_entry()
              actions.close(example_bufnr)
              actions.close(prompt_bufnr)

              if selection then
                logger.info("ui.telescope.input", "show_simple_input", "Selected example theme",
                  { example = selection.value })
                require("hexwitch").generate(selection.value)
              end
            end)
            return true
          end,
        }):find()
      end)

      return true
    end,
  }):find()
end

-- Show save theme dialog
---@param default_name string Default theme name
---@param on_save function Callback when saved
---@param opts table Options
function M.show_save_dialog(default_name, on_save, opts)
  opts = opts or {}

  logger.info("ui.telescope.input", "show_save_dialog", "Showing save theme dialog",
    { default_name = default_name })

  -- Sanitize the default theme name to match validation requirements
  -- Convert spaces and invalid characters to underscores/hyphens
  local sanitized_name = default_name
    :gsub("%s+", "_")  -- Replace spaces with underscores
    :gsub("[^%w_%-]", "-")  -- Replace other invalid chars with hyphens
    :gsub("%-+", "_")  -- Replace multiple hyphens with single underscore
    :gsub("^[%-_]+", "")  -- Remove leading underscores/hyphens
    :gsub("[%-_]+$", "")  -- Remove trailing underscores/hyphens
    :lower()  -- Convert to lowercase for consistency

  -- If sanitization changed the name significantly, use a fallback
  if sanitized_name == "" then
    sanitized_name = "custom_theme"
  end

  -- Update default_name to the sanitized version
  default_name = sanitized_name

  pickers.new(opts, {
    prompt_title = "Save Current Theme (letters, numbers, _, - only)",
    default_text = default_name,
    finder = finders.new_table({
      results = {},
      entry_maker = function(input)
        return {
          value = input,
          display = input,
          ordinal = input,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)

      actions.select_default:replace(function()
        local name = action_state.get_current_line():gsub("^%s+", ""):gsub("%s+$", "")
        actions.close(prompt_bufnr)

        if name and name ~= "" then
          -- Validate theme name
          if not name:match("^[%w_%-]+$") then
            vim.notify("Theme name can only contain letters, numbers, underscores, and hyphens", vim.log.levels.ERROR)
            M.show_save_dialog(default_name, on_save, opts)
            return
          end

          logger.info("ui.telescope.input", "show_save_dialog", "Saving theme", { theme_name = name })
          if on_save then
            on_save(name)
          end
        else
          vim.notify("Theme name cannot be empty", vim.log.levels.WARN)
          M.show_save_dialog(default_name, on_save, opts)
        end
      end)

      return true
    end,
  }):find()
end

-- Show rename theme dialog
---@param old_name string Current theme name
---@param on_rename function Callback when renamed
---@param opts table Options
function M.show_rename_dialog(old_name, on_rename, opts)
  opts = opts or {}

  logger.info("ui.telescope.input", "show_rename_dialog", "Showing rename theme dialog",
    { old_name = old_name })

  pickers.new(opts, {
    prompt_title = "✏️ Rename Theme",
    finder = finders.new_table({
      results = {},
      entry_maker = function(input)
        return {
          value = input,
          display = input,
          ordinal = input,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      -- Set default value to old name
      vim.api.nvim_buf_set_lines(prompt_bufnr, 0, -1, false, { old_name })
      vim.api.nvim_win_set_cursor(0, { 1, #old_name })

      actions.select_default:replace(function()
        local new_name = action_state.get_current_line():gsub("^%s+", ""):gsub("%s+$", "")
        actions.close(prompt_bufnr)

        if new_name and new_name ~= "" then
          if new_name == old_name then
            vim.notify("New name is the same as the old name", vim.log.levels.WARN)
            M.show_rename_dialog(old_name, on_rename, opts)
            return
          end

          -- Validate theme name
          if not new_name:match("^[%w_%-]+$") then
            vim.notify("Theme name can only contain letters, numbers, underscores, and hyphens", vim.log.levels.ERROR)
            M.show_rename_dialog(old_name, on_rename, opts)
            return
          end

          logger.info("ui.telescope.input", "show_rename_dialog", "Renaming theme",
            { old_name = old_name, new_name = new_name })
          if on_rename then
            on_rename(old_name, new_name)
          end
        else
          vim.notify("Theme name cannot be empty", vim.log.levels.WARN)
          M.show_rename_dialog(old_name, on_rename, opts)
        end
      end)

      return true
    end,
  }):find()
end

-- Show confirmation dialog
---@param message string Confirmation message
---@param on_confirm function Callback when confirmed
---@param opts table Options
function M.show_confirmation(message, on_confirm, opts)
  opts = opts or {}

  logger.info("ui.telescope.input", "show_confirmation", "Showing confirmation dialog",
    { message = message })

  local options = {
    { name = "Confirm", value = "confirm", icon = "✓" },
    { name = "Cancel", value = "cancel", icon = "✗" },
  }

  local displayer = entry_display.create({
    separator = " ",
    items = {
      { width = 3 }, -- icon
      { remaining = true }, -- option name
    },
  })

  pickers.new(opts, {
    prompt_title = "Confirmation",
    finder = finders.new_table({
      results = options,
      entry_maker = function(option)
        return {
          value = option.value,
          display = function()
            return displayer({
              { option.icon, "Special" },
              { option.name, "String" },
            })
          end,
          ordinal = option.name,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local selection = action_state.get_selected_entry()
        actions.close(prompt_bufnr)

        if selection and selection.value == "confirm" then
          logger.info("ui.telescope.input", "show_confirmation", "User confirmed action")
          if on_confirm then
            on_confirm()
          end
        else
          logger.info("ui.telescope.input", "show_confirmation", "User cancelled action")
        end
      end)

      return true
    end,
  }):find()
end

-- Show refinement input dialog with examples
---@param on_refine function Callback when refinement description is provided
---@param opts table Options
function M.show_refinement_input(on_refine, opts)
  opts = opts or {}

  logger.info("ui.telescope.input", "show_refinement_input", "Showing refinement input dialog")

  -- Refinement-specific examples
  local examples = {
    "Make the colors more pastel and soft",
    "Increase contrast for better readability",
    "Make it warmer, more yellow/orange tones",
    "Add more vibrant accent colors",
    "Reduce saturation for a more muted look",
    "Make background darker and keep accents bright",
    "Convert to monochrome with high contrast",
    "Softer colors, easier on the eyes",
    "More professional, business-friendly colors",
    "Increase red tones for warmer feel",
  }

  pickers.new(opts, {
    prompt_title = "Describe Theme Refinements",
    finder = finders.new_table({
      results = {},
      entry_maker = function(input)
        return {
          value = input,
          display = input,
          ordinal = input,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        local prompt = action_state.get_current_line()
        actions.close(prompt_bufnr)

        if prompt and prompt ~= "" then
          logger.info("ui.telescope.input", "show_refinement_input",
            "Applying refinement from user input", { input = prompt })
          if on_refine then
            on_refine(prompt)
          end
        else
          vim.notify("Refinement description cannot be empty", vim.log.levels.WARN)
        end
      end)

      -- Add example selection with tab
      map("i", "<Tab>", function()
        local current_examples = {}
        for _, example in ipairs(examples) do
          table.insert(current_examples, {
            value = example,
            display = "💡 " .. example,
            ordinal = example,
          })
        end

        -- Show examples picker
        pickers.new(opts, {
          prompt_title = "Refinement Examples",
          finder = finders.new_table({
            results = current_examples,
          }),
          sorter = conf.generic_sorter(opts),
          attach_mappings = function(example_bufnr, map)
            actions.select_default:replace(function()
              local selection = action_state.get_selected_entry()
              actions.close(example_bufnr)
              actions.close(prompt_bufnr)

              if selection then
                logger.info("ui.telescope.input", "show_refinement_input",
                  "Selected refinement example", { example = selection.value })
                if on_refine then
                  on_refine(selection.value)
                end
              end
            end)
            return true
          end,
        }):find()
      end)

      return true
    end,
  }):find()
end

return M
