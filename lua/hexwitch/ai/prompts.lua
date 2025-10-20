local M = {}

M.SYSTEM_PROMPT = [[You are an expert Neovim colorscheme designer specializing in creating beautiful, accessible, and harmonious color palettes. Your expertise includes:

- Understanding color theory and visual accessibility (WCAG contrast ratios)
- Creating cohesive palettes that work well together
- Balancing aesthetics with readability
- Semantic color assignments (red for errors, green for success, etc.)

Always ensure:
- All colors are in hex format (#RRGGBB)
- Proper contrast between background and foreground (minimum 7:1 for WCAG AAA)
- Semantic meaning in color choices
- Harmony and visual balance across the palette]]

---Build a theme generation prompt
---@param user_input string User's theme description
---@return string prompt
function M.build_theme_prompt(user_input)
  return string.format([[Generate a complete Neovim colorscheme based on this description: "%s"

Requirements:
1. **Base Colors**: Provide bg (background) and fg (foreground) with excellent contrast
2. **UI Elements**: Define bg_sidebar, bg_float, and bg_statusline for consistent UI
3. **Semantic Colors**:
   - red: errors, deletions, warnings
   - green: success, additions, strings
   - yellow: types, warnings, constants
   - blue: functions, information
   - cyan: operators, special characters
   - purple: keywords, control flow
   - magenta: special identifiers
   - orange: numbers, constants
4. **Accents**: comment (muted), selection (highlight), cursor (standout)

Ensure the palette is cohesive, accessible, and captures the essence of: "%s"]], user_input, user_input)
end

---Preset prompts for Telescope picker
---@type string[]
M.PRESETS = {
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

return M

