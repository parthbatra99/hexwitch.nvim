local M = {}
local color = require("hexwitch.utils.color")

function M.generate_contextual_suggestions(theme, user_intent)
  local suggestions = {}

  -- Extract colors from theme structure
  local colors = theme.colors or theme

  -- Always suggest contrast improvements if needed
  local bg_fg_ratio = color.get_contrast_ratio(colors.bg or colors.background, colors.fg or colors.foreground)
  if bg_fg_ratio < 7 then
    table.insert(suggestions, {
      title = "Improve Readability",
      description = "Increase contrast for better readability",
      category = "accessibility",
      preview_changes = "Increase background-foreground contrast to WCAG AAA",
      priority = "high",
      auto_fixable = true
    })
  end

  -- Contextual suggestions based on intent
  if user_intent:match("professional") or user_intent:match("corporate") then
    table.insert(suggestions, {
      title = "Professional Palette",
      description = "Use more subdued, business-appropriate colors",
      category = "style",
      preview_changes = "Reduce saturation, use neutral base colors",
      priority = "medium"
    })
  end

  if user_intent:match("warm") then
    table.insert(suggestions, {
      title = "Add Warmth",
      description = "Shift color temperature towards warmer tones",
      category = "mood",
      preview_changes = "Add red/orange undertones to color palette",
      priority = "medium"
    })
  end

  -- Always add a creative suggestion
  table.insert(suggestions, {
    title = "Complementary Accent",
    description = "Add a complementary accent color for visual interest",
    category = "enhancement",
    preview_changes = "Introduce accent color for highlighting",
    priority = "low"
  })

  return suggestions
end

function M.suggest_accessibility_improvements(theme)
  local improvements = {}

  -- Extract colors from theme structure
  local colors = theme.colors or theme

  -- Simple accessibility checks
  local bg_fg_ratio = color.get_contrast_ratio(colors.bg or colors.background, colors.fg or colors.foreground)
  if bg_fg_ratio < 4.5 then
    table.insert(improvements, {
      category = "accessibility",
      severity = "high",
      description = "Background-foreground contrast is too low",
      suggestion = "Increase contrast between background and foreground colors",
      auto_fixable = true
    })
  elseif bg_fg_ratio < 7 then
    table.insert(improvements, {
      category = "accessibility",
      severity = "medium",
      description = "Background-foreground contrast could be improved",
      suggestion = "Consider increasing contrast for better readability",
      auto_fixable = true
    })
  end

  -- Check comment visibility if comment color exists
  if colors.comment then
    local comment_ratio = color.get_contrast_ratio(colors.comment, colors.bg or colors.background)
    if comment_ratio < 4.5 then
      table.insert(improvements, {
        category = "accessibility",
        severity = "medium",
        description = "Comment contrast is insufficient",
        suggestion = "Make comments more visible against the background",
        auto_fixable = true
      })
    end
  end

  return improvements
end

return M