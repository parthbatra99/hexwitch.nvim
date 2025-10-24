local notify = require("hexwitch.utils.notify")

local M = {}

-- Validate theme data structure and color format
local function validate_theme_data(data)
  if not data or type(data) ~= "table" then
    return false, "Theme data must be a table"
  end

  if not data.colors or type(data.colors) ~= "table" then
    return false, "Theme must have a colors table"
  end

  -- Define required color fields
  local required_colors = {
    "bg", "fg", "bg_sidebar", "bg_float", "bg_statusline",
    "red", "orange", "yellow", "green", "cyan", "blue", "purple", "magenta",
    "comment", "selection", "cursor"
  }

  -- Check all required colors exist and are valid hex colors
  for _, color_key in ipairs(required_colors) do
    local color = data.colors[color_key]
    if not color or type(color) ~= "string" then
      return false, string.format("Missing or invalid color: %s", color_key)
    end

    -- Validate hex color format (#RRGGBB)
    if not color:match("^#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]$") then
      return false, string.format("Invalid color format for %s: %s (must be #RRGGBB)", color_key, color)
    end
  end

  -- Validate optional terminal colors if present
  if data.terminal and type(data.terminal) == "table" then
    for i = 0, 15 do
      if data.terminal[i] then
        local term_color = data.terminal[i]
        if type(term_color) ~= "string" or not term_color:match("^#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]$") then
          return false, string.format("Invalid terminal color %d: %s", i, tostring(term_color))
        end
      end
    end
  end

  return true, "Theme data is valid"
end

---Apply a colorscheme from AI-generated data
---@param colorscheme_data hexwitch.ColorschemeData
function M.apply(colorscheme_data)
  -- Validate theme data before applying
  local is_valid, validation_message = validate_theme_data(colorscheme_data)
  if not is_valid then
    notify.error("Theme validation failed: " .. validation_message)
    return false
  end

  local colors = colorscheme_data.colors

  -- Define the highlight groups with the generated colors
  local highlights = {
    -- Basic highlights
    Normal = { fg = colors.fg, bg = colors.bg },
    NormalFloat = { fg = colors.fg, bg = colors.bg_float },
    NormalSB = { fg = colors.fg, bg = colors.bg_sidebar },

    -- Status line
    StatusLine = { fg = colors.fg, bg = colors.bg_statusline },
    StatusLineNC = { fg = colors.comment, bg = colors.bg_statusline },

    -- Cursor
    Cursor = { fg = colors.bg, bg = colors.cursor },
    CursorLine = { bg = colors.selection },
    CursorLineNr = { fg = colors.fg, bg = colors.bg },

    -- Selection
    Visual = { bg = colors.selection },
    VisualNOS = { bg = colors.selection },

    -- Basic colors
    Comment = { fg = colors.comment, italic = true },
    Constant = { fg = colors.magenta },
    String = { fg = colors.green },
    Character = { fg = colors.green },
    Number = { fg = colors.orange },
    Boolean = { fg = colors.orange },
    Float = { fg = colors.orange },

    -- Identifiers
    Identifier = { fg = colors.fg },
    Function = { fg = colors.blue },

    -- Keywords
    Keyword = { fg = colors.purple },
    Conditional = { fg = colors.purple },
    Repeat = { fg = colors.purple },
    Label = { fg = colors.purple },
    Operator = { fg = colors.cyan },
    Exception = { fg = colors.red },

    -- Preprocessor
    PreProc = { fg = colors.cyan },
    Include = { fg = colors.cyan },
    Define = { fg = colors.cyan },
    Macro = { fg = colors.cyan },
    PreCondit = { fg = colors.cyan },

    -- Types
    Type = { fg = colors.yellow },
    StorageClass = { fg = colors.yellow },
    Structure = { fg = colors.yellow },
    Typedef = { fg = colors.yellow },

    -- Special
    Special = { fg = colors.orange },
    SpecialChar = { fg = colors.orange },
    Tag = { fg = colors.orange },
    Delimiter = { fg = colors.orange },
    SpecialComment = { fg = colors.comment },

    -- Errors
    Error = { fg = colors.red, bg = colors.bg },
    Todo = { fg = colors.yellow, bg = colors.bg },

    -- Underline
    Underlined = { underline = true },

    -- Ignored
    Ignore = { fg = colors.comment },

    -- LSP
    DiagnosticError = { fg = colors.red },
    DiagnosticWarn = { fg = colors.yellow },
    DiagnosticInfo = { fg = colors.blue },
    DiagnosticHint = { fg = colors.cyan },
    DiagnosticUnderlineError = { sp = colors.red, undercurl = true },
    DiagnosticUnderlineWarn = { sp = colors.yellow, undercurl = true },
    DiagnosticUnderlineInfo = { sp = colors.blue, undercurl = true },
    DiagnosticUnderlineHint = { sp = colors.cyan, undercurl = true },

    -- TreeSitter
    ["@variable"] = { fg = colors.fg },
    ["@variable.builtin"] = { fg = colors.orange },
    ["@variable.parameter"] = { fg = colors.fg },
    ["@variable.member"] = { fg = colors.fg },
    ["@constant"] = { fg = colors.magenta },
    ["@constant.builtin"] = { fg = colors.orange },
    ["@constant.macro"] = { fg = colors.cyan },
    ["@string"] = { fg = colors.green },
    ["@string.regex"] = { fg = colors.green },
    ["@string.escape"] = { fg = colors.orange },
    ["@character"] = { fg = colors.green },
    ["@number"] = { fg = colors.orange },
    ["@boolean"] = { fg = colors.orange },
    ["@float"] = { fg = colors.orange },
    ["@function"] = { fg = colors.blue },
    ["@function.builtin"] = { fg = colors.blue },
    ["@function.macro"] = { fg = colors.cyan },
    ["@operator"] = { fg = colors.cyan },
    ["@keyword"] = { fg = colors.purple },
    ["@keyword.return"] = { fg = colors.purple },
    ["@conditional"] = { fg = colors.purple },
    ["@repeat"] = { fg = colors.purple },
    ["@label"] = { fg = colors.purple },
    ["@exception"] = { fg = colors.red },
    ["@type"] = { fg = colors.yellow },
    ["@type.builtin"] = { fg = colors.yellow },
    ["@type.definition"] = { fg = colors.yellow },
    ["@namespace"] = { fg = colors.cyan },
    ["@include"] = { fg = colors.cyan },
    ["@preproc"] = { fg = colors.cyan },
    ["@debug"] = { fg = colors.orange },
    ["@tag"] = { fg = colors.red },
    ["@tag.attribute"] = { fg = colors.yellow },
    ["@tag.delimiter"] = { fg = colors.comment },
  }

  -- Clear existing highlights
  if vim.g.colors_name then
    vim.cmd("hi clear")
  end

  -- Set colorscheme name
  local theme_name = colorscheme_data.name
  if not theme_name or theme_name == "" then
    theme_name = "hexwitch"
  end
  vim.g.colors_name = theme_name

  -- Apply highlights
  for group, hl in pairs(highlights) do
    vim.api.nvim_set_hl(0, group, hl)
  end

  -- Set terminal colors
  vim.g.terminal_color_0 = colors.bg
  vim.g.terminal_color_1 = colors.red
  vim.g.terminal_color_2 = colors.green
  vim.g.terminal_color_3 = colors.yellow
  vim.g.terminal_color_4 = colors.blue
  vim.g.terminal_color_5 = colors.purple
  vim.g.terminal_color_6 = colors.cyan
  vim.g.terminal_color_7 = colors.fg
  vim.g.terminal_color_8 = colors.comment
  vim.g.terminal_color_9 = colors.red
  vim.g.terminal_color_10 = colors.green
  vim.g.terminal_color_11 = colors.yellow
  vim.g.terminal_color_12 = colors.blue
  vim.g.terminal_color_13 = colors.purple
  vim.g.terminal_color_14 = colors.cyan
  vim.g.terminal_color_15 = colors.fg

  notify.debug("Theme applied successfully")
  return true
end

return M