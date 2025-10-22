local logger = require("hexwitch.utils.logger")

local M = {}

-- Convert hex to RGB
---@param hex string Hex color (#RRGGBB)
---@return number r Red 0-255
---@return number g Green 0-255
---@return number b Blue 0-255
function M.hex_to_rgb(hex)
  hex = hex:gsub("#", "")
  local r = tonumber(hex:sub(1, 2), 16)
  local g = tonumber(hex:sub(3, 4), 16)
  local b = tonumber(hex:sub(5, 6), 16)
  return r, g, b
end

-- Convert RGB to hex
---@param r number Red 0-255
---@param g number Green 0-255
---@param b number Blue 0-255
---@return string hex Hex color (#RRGGBB)
function M.rgb_to_hex(r, g, b)
  return string.format("#%02x%02x%02x", r, g, b)
end

-- Convert RGB to HSL
---@param r number Red 0-255
---@param g number Green 0-255
---@param b number Blue 0-255
---@return number h Hue 0-360
---@return number s Saturation 0-100
---@return number l Lightness 0-100
function M.rgb_to_hsl(r, g, b)
  r, g, b = r / 255, g / 255, b / 255

  local max = math.max(r, g, b)
  local min = math.min(r, g, b)
  local h, s, l = 0, 0, (max + min) / 2

  if max == min then
    h, s = 0, 0 -- achromatic
  else
    local d = max - min
    s = l > 0.5 and d / (2 - max - min) or d / (max + min)

    if max == r then
      h = (g - b) / d + (g < b and 6 or 0)
    elseif max == g then
      h = (b - r) / d + 2
    else -- max == b
      h = (r - g) / d + 4
    end

    h = h / 6 * 360
  end

  return h, s * 100, l * 100
end

-- Convert HSL to RGB
---@param h number Hue 0-360
---@param s number Saturation 0-100
---@param l number Lightness 0-100
---@return number r Red 0-255
---@return number g Green 0-255
---@return number b Blue 0-255
function M.hsl_to_rgb(h, s, l)
  h, s, l = h / 360, s / 100, l / 100

  local r, g, b

  if s == 0 then
    r, g, b = l, l, l -- achromatic
  else
    local function hue_to_rgb(p, q, t)
      if t < 0 then t = t + 1 end
      if t > 1 then t = t - 1 end
      if t < 1/6 then return p + (q - p) * 6 * t end
      if t < 1/2 then return q end
      if t < 2/3 then return p + (q - p) * (2/3 - t) * 6 end
      return p
    end

    local q = l < 0.5 and l * (1 + s) or l + s - l * s
    local p = 2 * l - q

    r = hue_to_rgb(p, q, h + 1/3)
    g = hue_to_rgb(p, q, h)
    b = hue_to_rgb(p, q, h - 1/3)
  end

  return math.floor(r * 255 + 0.5), math.floor(g * 255 + 0.5), math.floor(b * 255 + 0.5)
end

-- Convert hex to HSL
---@param hex string Hex color (#RRGGBB)
---@return number h Hue 0-360
---@return number s Saturation 0-100
---@return number l Lightness 0-100
function M.hex_to_hsl(hex)
  local r, g, b = M.hex_to_rgb(hex)
  return M.rgb_to_hsl(r, g, b)
end

-- Convert HSL to hex
---@param h number Hue 0-360
---@param s number Saturation 0-100
---@param l number Lightness 0-100
---@return string hex Hex color (#RRGGBB)
function M.hsl_to_hex(h, s, l)
  local r, g, b = M.hsl_to_rgb(h, s, l)
  return M.rgb_to_hex(r, g, b)
end

-- Calculate relative luminance (WCAG formula)
---@param hex string Hex color (#RRGGBB)
---@return number luminance Relative luminance 0-1
function M.get_luminance(hex)
  local r, g, b = M.hex_to_rgb(hex)

  -- Normalize to 0-1 range
  r, g, b = r / 255, g / 255, b / 255

  -- Apply gamma correction
  local function correct(c)
    return c <= 0.03928 and c / 12.92 or math.pow((c + 0.055) / 1.055, 2.4)
  end

  r, g, b = correct(r), correct(g), correct(b)

  -- Calculate luminance
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
end

-- Calculate contrast ratio
---@param hex1 string First hex color
---@param hex2 string Second hex color
---@return number ratio Contrast ratio 1-21
function M.get_contrast_ratio(hex1, hex2)
  local lum1 = M.get_luminance(hex1)
  local lum2 = M.get_luminance(hex2)

  local lighter = math.max(lum1, lum2)
  local darker = math.min(lum1, lum2)

  return (lighter + 0.05) / (darker + 0.05)
end

-- Check if color meets contrast threshold
---@param foreground string Foreground hex color
---@param background string Background hex color
---@param threshold? number Minimum contrast ratio (default 4.5)
---@return boolean passes True if meets threshold
function M.meets_contrast(foreground, background, threshold)
  threshold = threshold or 4.5
  local ratio = M.get_contrast_ratio(foreground, background)
  return ratio >= threshold
end

-- Adjust color lightness
---@param hex string Hex color
---@param amount number Amount to adjust (-100 to 100, positive = lighter)
---@return string new_hex Adjusted hex color
function M.adjust_lightness(hex, amount)
  local h, s, l = M.hex_to_hsl(hex)
  l = math.max(0, math.min(100, l + amount))
  return M.hsl_to_hex(h, s, l)
end

-- Adjust color saturation
---@param hex string Hex color
---@param amount number Amount to adjust (-100 to 100, positive = more saturated)
---@return string new_hex Adjusted hex color
function M.adjust_saturation(hex, amount)
  local h, s, l = M.hex_to_hsl(hex)
  s = math.max(0, math.min(100, s + amount))
  return M.hsl_to_hex(h, s, l)
end

-- Adjust color hue (temperature)
---@param hex string Hex color
---@param degrees number Degrees to rotate hue (-360 to 360, positive = warmer)
---@return string new_hex Adjusted hex color
function M.adjust_hue(hex, degrees)
  local h, s, l = M.hex_to_hsl(hex)
  h = (h + degrees) % 360
  if h < 0 then h = h + 360 end
  return M.hsl_to_hex(h, s, l)
end

-- Increase contrast between two colors
---@param foreground string Foreground hex color
---@param background string Background hex color
---@param min_ratio number Target minimum contrast ratio
---@return string new_foreground Adjusted foreground color
function M.increase_contrast(foreground, background, min_ratio)
  local current_ratio = M.get_contrast_ratio(foreground, background)

  if current_ratio >= min_ratio then
    return foreground -- Already meets requirement
  end

  logger.debug("color.utils", "increase_contrast",
    string.format("Increasing contrast from %.2f to %.2f", current_ratio, min_ratio))

  local new_foreground = foreground
  local step = 5 -- Adjust in 5% increments

  -- Try lightening first
  for i = 1, 10 do
    new_foreground = M.adjust_lightness(foreground, i * step)
    if M.get_contrast_ratio(new_foreground, background) >= min_ratio then
      return new_foreground
    end
  end

  -- If lightening doesn't work, try darkening
  for i = 1, 10 do
    new_foreground = M.adjust_lightness(foreground, -i * step)
    if M.get_contrast_ratio(new_foreground, background) >= min_ratio then
      return new_foreground
    end
  end

  -- If still no success, try both saturation and lightness adjustments
  new_foreground = M.adjust_saturation(foreground, 20) -- Increase saturation
  for i = 1, 10 do
    local test_color = M.adjust_lightness(new_foreground, i * step)
    if M.get_contrast_ratio(test_color, background) >= min_ratio then
      return test_color
    end
  end

  logger.warn("color.utils", "increase_contrast",
    "Could not achieve desired contrast ratio, returning original")
  return foreground
end

-- Reduce contrast by nudging foreground lightness toward background
---@param foreground string Foreground hex color
---@param background string Background hex color
---@param amount number Step amount to move lightness toward background (positive)
---@return string new_foreground
function M.decrease_contrast(foreground, background, amount)
  amount = math.abs(amount or 5)
  local hf, sf, lf = M.hex_to_hsl(foreground)
  local hb, sb, lb = M.hex_to_hsl(background)

  -- Move lightness toward background
  if lf > lb then
    lf = math.max(0, lf - amount)
  elseif lf < lb then
    lf = math.min(100, lf + amount)
  end

  -- Slightly reduce saturation to soften edges
  sf = math.max(0, math.min(100, sf - amount * 0.5))

  return M.hsl_to_hex(hf, sf, lf)
end

-- Apply theme adjustment function to all colors
---@param colors table Color palette
---@param adjustment_func function Adjustment function
---@param ... args Arguments for adjustment function
---@return table new_colors Adjusted color palette
function M.apply_theme_adjustment(colors, adjustment_func, ...)
  local new_colors = vim.deepcopy(colors)

  for key, color in pairs(colors) do
    if type(color) == "string" and color:match("^#%x%x%x%x%x%x$") then
      new_colors[key] = adjustment_func(color, ...)
    end
  end

  return new_colors
end

-- Generate color palette variations
function M.generate_variations(base_color)
  local h, s, l = M.hex_to_hsl(base_color)

  return {
    original = base_color,
    lighter = M.adjust_lightness(base_color, 20),
    darker = M.adjust_lightness(base_color, -20),
    more_saturated = M.adjust_saturation(base_color, 20),
    less_saturated = M.adjust_saturation(base_color, -20),
    warmer = M.adjust_hue(base_color, 15),
    cooler = M.adjust_hue(base_color, -15),
    high_contrast = l > 50 and M.adjust_lightness(base_color, -30) or M.adjust_lightness(base_color, 30),
  }
end

return M