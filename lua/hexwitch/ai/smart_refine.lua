local M = {}

function M.parse_semantic_request(request)
  local result = {
    temperature_shift = "neutral",
    saturation_change = "none",
    brightness_change = "none",
    contrast_change = "none",
    mood_analysis = {}
  }

  -- Semantic mapping for common themes
  if request:match("autumn") or request:match("sunset") then
    result.temperature_shift = "warm"
    result.saturation_change = "medium"
  elseif request:match("ocean") or request:match("sky") then
    result.temperature_shift = "cool"
  elseif request:match("forest") or request:match("nature") then
    result.temperature_shift = "neutral"
    result.saturation_change = "high"
  end

  return result
end

function M.generate_suggestions(theme)
  return {
    {
      description = "Increase contrast for better readability",
      changes = "increase contrast slightly",
      priority = "high"
    },
    {
      description = "Add warmth for a cozier feel",
      changes = "make colors slightly warmer",
      priority = "medium"
    }
  }
end

return M