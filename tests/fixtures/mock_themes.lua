-- Test fixtures for hexwitch theme data
-- Provides consistent mock data for testing

local M = {}

---Valid theme data with all required colors
M.valid_theme = {
  name = "test-theme",
  description = "A test theme for unit testing",
  colors = {
    bg = "#1a1b26",
    fg = "#c0caf5",
    bg_sidebar = "#1a1b26",
    bg_float = "#24283b",
    bg_statusline = "#1f2335",
    red = "#f7768e",
    orange = "#ff9e64",
    yellow = "#e0af68",
    green = "#9ece6a",
    cyan = "#73daca",
    blue = "#7aa2f7",
    purple = "#bb9af7",
    magenta = "#d18616",
    comment = "#565f89",
    selection = "#33467c",
    cursor = "#c0caf5"
  }
}

---Dark theme variant
M.dark_theme = {
  name = "dark-test-theme",
  description = "A dark theme for testing",
  colors = {
    bg = "#000000",
    fg = "#ffffff",
    bg_sidebar = "#0a0a0a",
    bg_float = "#1a1a1a",
    bg_statusline = "#0f0f0f",
    red = "#ff6b6b",
    orange = "#ffa726",
    yellow = "#ffd93d",
    green = "#6bcf7f",
    cyan = "#4ecdc4",
    blue = "#4dabf7",
    purple = "#9775fa",
    magenta = "#f06595",
    comment = "#6c757d",
    selection = "#2d3748",
    cursor = "#ffffff"
  }
}

---Light theme variant
M.light_theme = {
  name = "light-test-theme",
  description = "A light theme for testing",
  colors = {
    bg = "#ffffff",
    fg = "#212529",
    bg_sidebar = "#f8f9fa",
    bg_float = "#e9ecef",
    bg_statusline = "#f1f3f4",
    red = "#dc3545",
    orange = "#fd7e14",
    yellow = "#ffc107",
    green = "#28a745",
    cyan = "#17a2b8",
    blue = "#007bff",
    purple = "#6f42c1",
    magenta = "#e83e8c",
    comment = "#6c757d",
    selection = "#e9ecef",
    cursor = "#212529"
  }
}

---High contrast theme
M.high_contrast_theme = {
  name = "high-contrast-theme",
  description = "A high contrast theme for accessibility testing",
  colors = {
    bg = "#000000",
    fg = "#ffffff",
    bg_sidebar = "#000000",
    bg_float = "#1a1a1a",
    bg_statusline = "#000000",
    red = "#ff0000",
    orange = "#ff8800",
    yellow = "#ffff00",
    green = "#00ff00",
    cyan = "#00ffff",
    blue = "#0088ff",
    purple = "#ff00ff",
    magenta = "#ff0088",
    comment = "#aaaaaa",
    selection = "#333333",
    cursor = "#ffffff"
  }
}

---Low contrast theme (for accessibility testing)
M.low_contrast_theme = {
  name = "low-contrast-theme",
  description = "A theme with low contrast for testing",
  colors = {
    bg = "#303030",
    fg = "#a0a0a0",
    bg_sidebar = "#2a2a2a",
    bg_float = "#353535",
    bg_statusline = "#2d2d2d",
    red = "#a07070",
    orange = "#a08060",
    yellow = "#a09050",
    green = "#70a070",
    cyan = "#60a0a0",
    blue = "#6080a0",
    purple = "#8070a0",
    magenta = "#a06090",
    comment = "#606060",
    selection = "#3a3a3a",
    cursor = "#a0a0a0"
  }
}

---Monochromatic theme
M.monochromatic_theme = {
  name = "monochromatic-theme",
  description = "A monochromatic theme with grayscale colors",
  colors = {
    bg = "#1a1a1a",
    fg = "#e0e0e0",
    bg_sidebar = "#1f1f1f",
    bg_float = "#2a2a2a",
    bg_statusline = "#252525",
    red = "#d0d0d0",
    orange = "#c8c8c8",
    yellow = "#b8b8b8",
    green = "#a0a0a0",
    cyan = "#989898",
    blue = "#888888",
    purple = "#787878",
    magenta = "#686868",
    comment = "#606060",
    selection = "#3a3a3a",
    cursor = "#f0f0f0"
  }
}

---Theme with unusual color names (for testing edge cases)
M.unusual_colors_theme = {
  name = "unusual-colors-theme",
  description = "Theme with unusual but valid hex colors",
  colors = {
    bg = "#123abc",
    fg = "#def456",
    bg_sidebar = "#789abc",
    bg_float = "#abc123",
    bg_statusline = "#456def",
    red = "#ff0011",
    orange = "#ff8800",
    yellow = "#00ff88",
    green = "#1100ff",
    cyan = "#88ff00",
    blue = "#0088ff",
    purple = "#ff0088",
    magenta = "#00ffff",
    comment = "#444444",
    selection = "#222222",
    cursor = "#ffffff"
  }
}

---Invalid theme data for error testing
M.invalid_themes = {
  -- Missing name
  {
    description = "Theme without name",
    colors = M.valid_theme.colors
  },

  -- Missing colors
  {
    name = "no-colors",
    description = "Theme without colors"
  },

  -- Empty colors object
  {
    name = "empty-colors",
    description = "Theme with empty colors",
    colors = {}
  },

  -- Missing required color
  {
    name = "missing-bg",
    description = "Theme missing background color",
    colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = nil })
  },

  -- Invalid hex color format
  {
    name = "invalid-hex",
    description = "Theme with invalid hex colors",
    colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = "not-a-color" })
  },

  -- Too short hex color
  {
    name = "short-hex",
    description = "Theme with short hex color",
    colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = "#123" })
  },

  -- Too long hex color
  {
    name = "long-hex",
    description = "Theme with long hex color",
    colors = vim.tbl_extend("force", M.valid_theme.colors, { bg = "#1234567" })
  }
}

---Minimal valid theme (just required colors)
M.minimal_theme = {
  name = "minimal-theme",
  description = "Minimal theme with only required colors",
  colors = {
    bg = "#000000",
    fg = "#ffffff",
    bg_sidebar = "#000000",
    bg_float = "#000000",
    bg_statusline = "#000000",
    red = "#ff0000",
    orange = "#ff8800",
    yellow = "#ffff00",
    green = "#00ff00",
    cyan = "#00ffff",
    blue = "#0000ff",
    purple = "#ff00ff",
    magenta = "#ff0088",
    comment = "#888888",
    selection = "#444444",
    cursor = "#ffffff"
  }
}

---Collection of multiple themes for batch testing
M.theme_collection = {
  M.valid_theme,
  M.dark_theme,
  M.light_theme,
  M.high_contrast_theme,
  M.monochromatic_theme,
  M.unusual_colors_theme,
  M.minimal_theme
}

---Generate a theme with custom colors
---@param name string Theme name
---@param description string Theme description
---@param base_color string Base hex color
---@return table Theme data
M.generate_theme = function(name, description, base_color)
  -- Simple color variation based on base_color
  local r = tonumber(base_color:sub(2, 3), 16)
  local g = tonumber(base_color:sub(4, 5), 16)
  local b = tonumber(base_color:sub(6, 7), 16)

  local function vary_color(factor)
    local nr = math.min(255, math.max(0, math.floor(r * factor)))
    local ng = math.min(255, math.max(0, math.floor(g * factor)))
    local nb = math.min(255, math.max(0, math.floor(b * factor)))
    return string.format("#%02x%02x%02x", nr, ng, nb)
  end

  return {
    name = name,
    description = description,
    colors = {
      bg = base_color,
      fg = vary_color(3),
      bg_sidebar = vary_color(0.8),
      bg_float = vary_color(1.2),
      bg_statusline = vary_color(0.9),
      red = "#ff0000",
      orange = "#ff8800",
      yellow = "#ffff00",
      green = "#00ff00",
      cyan = "#00ffff",
      blue = "#0000ff",
      purple = "#ff00ff",
      magenta = "#ff0088",
      comment = vary_color(0.5),
      selection = vary_color(1.5),
      cursor = "#ffffff"
    }
  }
end

---Create theme variations for testing
---@param base_theme table Base theme to create variations from
---@return table Collection of theme variations
M.create_variations = function(base_theme)
  local variations = {}

  -- Brighter variation
  local brighter = vim.deepcopy(base_theme)
  brighter.name = brighter.name .. "-brighter"
  brighter.description = brighter.description .. " (brighter)"
  for key, color in pairs(brighter.colors) do
    if type(color) == "string" and color:match("^#") then
      local r = tonumber(color:sub(2, 3), 16)
      local g = tonumber(color:sub(4, 5), 16)
      local b = tonumber(color:sub(6, 7), 16)
      r = math.min(255, math.floor(r * 1.3))
      g = math.min(255, math.floor(g * 1.3))
      b = math.min(255, math.floor(b * 1.3))
      brighter.colors[key] = string.format("#%02x%02x%02x", r, g, b)
    end
  end
  table.insert(variations, brighter)

  -- Darker variation
  local darker = vim.deepcopy(base_theme)
  darker.name = darker.name .. "-darker"
  darker.description = darker.description .. " (darker)"
  for key, color in pairs(darker.colors) do
    if type(color) == "string" and color:match("^#") then
      local r = tonumber(color:sub(2, 3), 16)
      local g = tonumber(color:sub(4, 5), 16)
      local b = tonumber(color:sub(6, 7), 16)
      r = math.floor(r * 0.7)
      g = math.floor(g * 0.7)
      b = math.floor(b * 0.7)
      darker.colors[key] = string.format("#%02x%02x%02x", r, g, b)
    end
  end
  table.insert(variations, darker)

  return variations
end

return M