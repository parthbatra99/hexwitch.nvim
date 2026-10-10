local color_utils = require("hexwitch.utils.color")

local M = {}

M.SYSTEM_PROMPT = [[You are an expert Neovim colorscheme designer with deep knowledge of:

**Color Theory:**
- Color harmony (analogous, complementary, triadic, split-complementary)
- HSL color space manipulation (hue, saturation, lightness)
- Color temperature consistency (warm vs cool palettes)

**Interface Design Principles (Dark, Light, and Mid-tone Modes):**
- Respect the user's requested mood, era, materials, and lighting cues before picking background brightness.
- For dark backdrops, avoid pure black (#000000); prefer deep grays like #17181f, #1e1f28, #24273a unless the user explicitly insists on true black.
- For light backdrops, avoid harsh pure white (#ffffff); prefer softened neutrals like #f5f2eb, #f0f4f8, #ece7df unless bright white is explicitly requested.
- For mid-tone or balanced palettes, anchor the background in the #2b2b34 to #4a4c52 range and balance foreground accordingly.
- Create visual hierarchy through deliberate lightness variation, surface depth, and accent placement.
- Maintain consistent color temperature across base, accents, and neutrals.

**Accessibility (WCAG AAA):**
- Background-to-foreground contrast: minimum 7:1 ratio (AAA standard)
- Comment text contrast: minimum 4.5:1 ratio
- Syntax colors contrast: minimum 4.5:1 against background
- Use tools like contrast calculators to verify ratios

**Semantic Programming Colors:**
- Red (#e06c75, #f38ba8): errors, deletions, danger, important keywords
- Green (#98c379, #a6e3a1): strings, success, additions, growth
- Yellow (#e5c07b, #f9e2af): types, warnings, constants, attention
- Blue (#61afef, #89b4fa): functions, methods, information, links
- Purple (#c678dd, #cba6f7): keywords, control flow, special syntax
- Cyan (#56b6c2, #89dceb): operators, special chars, utilities
- Orange (#d19a66, #fab387): numbers, constants, parameters
- Comment (#6c7086, #565f89): muted, 50-60% lightness of foreground

**Design Philosophy:**
- Colorful is better than colorless - use distinct colors for clarity
- Harmony over dissonance - colors must complement each other
- Balance contrast - readable but not harsh

**Responsiveness to Prompts:**
- If the user evokes daylight, paper, pastel, sepia, or airy moods, build a light or mid-tone base with darker readable foreground text.
- If the user evokes night, nebulae, neon, cyberpunk, or deep-space moods, build a dark base with luminous accents.
- If the mood is ambiguous, aim for a neutral mid-tone base and let accent colors express the concept.

You must generate colorschemes that are beautiful, functional, and accessible.]]

---Build a theme generation prompt
---@param user_input string User's theme description
---@return string prompt
function M.build_theme_prompt(user_input)
  return string.format([[Generate a complete, accessible Neovim colorscheme for: "%s"

**STEP 1: Analyze the Theme Request**
Identify the mood, time of day, textures, materials, cultural references, and color temperature cues.

**CRITICAL: Detect Color Palette Complexity**
Determine if the user wants:
- **Minimal/Monochrome** (keywords: minimal, monochrome, single accent, grayscale, simple, clean, one color)
  → Use primarily grays with 1-2 accent colors. Most semantic colors should be subtle variations of the same hue.
- **Moderate** (keywords: balanced, subtle, muted, calm)
  → Use 3-5 distinct colors with variations.
- **Colorful** (keywords: vibrant, rainbow, colorful, rich, diverse)
  → Use full 8-color spectrum with distinct hues.

**STEP 2: Decide Background & Foreground Pairing**
1. Determine whether the request wants a **dark**, **light**, or **mid-tone/balanced** base (or an explicit extreme). Document the reasoning.
2. Choose background (`bg`) and foreground (`fg`) ranges accordingly:
   - **Dark:** bg between #15161c and #262830. fg between #c4c6d0 and #e5e7f2.
   - **Mid-tone / balanced:** bg between #2b2b34 and #4a4c52. fg between #dbdde6 and #f6f7fb.
   - **Light:** bg between #e8e5dc and #f5f4f0. fg between #22232a and #31333c.
   (If the prompt explicitly demands a different brightness, justify the deviation and keep contrast accessible.)
3. Verify bg-to-fg contrast ratio is ≥7:1 (WCAG AAA).
4. Summarize how the base brightness supports the requested vibe.

**STEP 3: Select Color Harmony**
Choose ONE approach based on the theme:
- **Analogous**: 3 adjacent colors (cohesive, e.g., blue-cyan-green)
- **Complementary**: 2 opposite colors (vibrant, e.g., blue-orange)
- **Triadic**: 3 evenly spaced (balanced, e.g., red-yellow-blue)
- **Split-complementary**: Base + 2 adjacent to complement (softer contrast)

**STEP 4: Generate Semantic Colors**
Using your chosen harmony, create these colors with proper semantics (each ≥4.5:1 against `bg`):

**For Minimal/Monochrome themes:**
- Pick ONE accent hue (e.g., blue at 220°)
- Generate all 8 colors as subtle variations of that hue by adjusting lightness/saturation
- Keep saturation low (15-35%%) except for 1-2 key accents (40-60%%)
- Example: All colors use hue 220° but vary lightness: red=#8a95b0, orange=#95a0ba, yellow=#a0abc4, green=#8aa5b0, cyan=#7a9fb8, blue=#6a95c8 (accent), purple=#8595b8, magenta=#9095b8

**For Moderate themes:**
- Use 3-5 distinct hues with variations
- Keep saturation moderate (35-60%%)

**For Colorful themes:**
- **red**: Errors, deletions, danger (hue: 0-20° or 340-360°)
- **orange**: Numbers, constants (hue: 20-40°)
- **yellow**: Types, warnings (hue: 40-60°)
- **green**: Strings, success (hue: 80-150°)
- **cyan**: Operators, utilities (hue: 170-200°)
- **blue**: Functions, information (hue: 200-240°)
- **purple**: Keywords, control flow (hue: 260-300°)
- **magenta**: Special identifiers (hue: 300-330°)

**STEP 5: Generate UI Colors**
- **comment**: 50-60%% of `fg` lightness, same hue family, contrast ≥4.5:1 with `bg`.
- **selection**: 
  - Dark themes: 10-15%% lighter than `bg`.
  - Light themes: 10-15%% darker than `bg`.
  - Mid-tone: shift lightness ±12%% to create gentle focus.
- **cursor**: High contrast with `bg`, echo a key accent.
- **bg_sidebar**: Offset `bg` by 5%% lightness (direction depends on theme for depth).
- **bg_float**: Match `bg_sidebar` for consistency.
- **bg_statusline**: 8-12%% offset from `bg` to signal separation without glare.

**STEP 6: Saturation & Temperature Guidelines**
- Match saturation to mode:
  - Dark bases: 50-75%% saturation on accents to avoid neon glare.
  - Light bases: 35-55%% saturation with careful contrast; mute significantly to avoid harsh vibrancy on pale backgrounds. For warm/cozy themes (coffee, autumn, paper), keep saturation ≤45%% and shift hues toward earth tones.
  - Mid-tone bases: 55-80%% saturation for accents, ensure neutrals stay calm.
- Align temperature (warm vs cool) across base, neutrals, and accents.
- For light warm themes: prioritize browns (#6b4423, #8b5a3c), tans (#c4a57b, #d4b896), creams (#f5f0e8, #ede4d3), and muted oranges (#c8956f) over bright yellows.

**STEP 7: Validate Your Palette**
Self-check before responding:
✓ Does base brightness match the interpreted theme?
✓ Are `bg` and `fg` within the appropriate ranges (or justified if outside)?
✓ Is bg-to-fg contrast ≥7:1?
✓ Do all semantic colors meet ≥4.5:1 contrast with `bg`?
✓ Is the color harmony coherent and repeated throughout?
✓ Do comments, selections, and status surfaces follow their rules?
✓ Does the color palette complexity match the user's request (minimal vs colorful)?

**EXAMPLE - Good Dark Theme:**
Theme: "calm ocean night"
Harmony: Analogous (blue-cyan-teal)

bg: #1a1d2e (deep slate blue)
fg: #c3d6f8 (soft blue-white)
red: #e57474 (desaturated coral)
orange: #ea9d74 (warm peach)
yellow: #eac474 (soft gold)
green: #8dd4b0 (seafoam)
cyan: #74d4ea (ocean cyan)
blue: #748dea (cool blue)
purple: #9d74ea (lavender)
magenta: #c474ea (soft magenta)
comment: #5a6d8a (muted blue-gray)
selection: #2a3d5e (lighter highlight)
cursor: #74d4ea (matches cyan accent)

**EXAMPLE - Good Light Theme (Sepia Study):**
Theme: "sepia study with neon annotations"
Harmony: Split-complementary (sepia base with teal-magenta accents)

bg: #f2ecdf (warm paper)
fg: #2c2a29 (ink brown)
red: #d35d6b (muted crimson)
orange: #d28b4e (amber highlight)
yellow: #d7af4c (antique gold)
green: #5fa876 (sage neon hybrid)
cyan: #3aa6a8 (teal accent)
blue: #3d70c8 (cobalt note)
purple: #7c59c0 (violet annotation)
magenta: #b85ac8 (electric magenta)
comment: #8b7b66 (soft pencil)
selection: #e5d7bf (slightly deeper wash)
cursor: #3aa6a8 (teal edge)

**EXAMPLE - Good Light Theme (Coffee Shop):**
Theme: "coffee shop aesthetic, browns creams and warm neutrals"
Harmony: Analogous (warm browns-oranges-tans)

bg: #f5f0e8 (cream latte)
fg: #3a2a1f (espresso ink)
red: #b85d52 (cinnamon spice)
orange: #c8956f (caramel drizzle)
yellow: #d4b896 (honey glaze)
green: #7a9070 (sage leaf)
cyan: #6b8e8a (mint accent)
blue: #6b7fa8 (blueberry muffin)
purple: #9d7b8f (lavender latte)
magenta: #b87a8f (rose syrup)
comment: #8b7a68 (cocoa powder)
selection: #e8dfd0 (foam highlight)
cursor: #c8956f (caramel)

**EXAMPLE - Minimal/Monochrome Theme:**
Theme: "monochrome minimal, shades of gray with single accent color"
Harmony: Monochromatic (blue accent only)

bg: #1e1f28 (charcoal)
fg: #d4d6e0 (light gray)
red: #a8b0c0 (muted gray-blue for errors)
orange: #b0b8c8 (slightly warm gray)
yellow: #b8c0d0 (light gray-blue)
green: #a0b0c0 (cool gray)
cyan: #98b0c8 (subtle blue-gray)
blue: #6a95c8 (PRIMARY ACCENT - only vibrant color)
purple: #a8b0c8 (muted gray-blue)
magenta: #b0b0c8 (neutral gray-blue)
comment: #6a7080 (dark gray)
selection: #2a3040 (slightly lighter bg)
cursor: #6a95c8 (matches blue accent)

Now generate the colorscheme for: "%s"
Respond ONLY with valid JSON using this exact structure:
{
  "name": "short_theme_name",
  "description": "Concise description",
  "colors": {
    "bg": "#000000",
    "fg": "#000000",
    "bg_sidebar": "#000000",
    "bg_float": "#000000",
    "bg_statusline": "#000000",
    "red": "#000000",
    "orange": "#000000",
    "yellow": "#000000",
    "green": "#000000",
    "cyan": "#000000",
    "blue": "#000000",
    "purple": "#000000",
    "magenta": "#000000",
    "comment": "#000000",
    "selection": "#000000",
    "cursor": "#000000"
  }
}

No explanations, extra fields, or trailing text.]], user_input, user_input)
end

---Preset prompts for Telescope picker
---@type string[]
M.PRESETS = {
  "Dark cyberpunk with neon blue and purple accents, high contrast",
  "Calm ocean sunset transitioning from blue to warm orange",
  "Tokyo night inspired, vibrant purple and blue with neon highlights",
  "Nord-like cool blues and grays, minimal and clean",
  "Arctic winter theme, icy blues and whites on deep navy",
  "Forest at dusk, earthy greens and warm browns",
  "Autumn forest with orange, red, and golden yellow leaves",
  "Cozy fireplace, warm oranges and deep reds",
  "Desert sunset with warm sand colors and purple sky",
  "Gruvbox inspired, warm retro earthy tones",
  "Monochrome minimal, shades of gray with single accent color",
  "Dracula inspired, deep purples with balanced contrast",
  "Catppuccin mocha style, soothing pastels on dark blue-gray",
  "Rosé Pine inspired, balanced natural colors with soft pink",
  "One Dark modern, balanced syntax colors on charcoal",
  "Cherry blossom at night, soft pinks on dark blue",
  "Deep space nebula, cosmic purples and blues",
  "Volcanic lava, dark grays with bright orange and red",
  "Underwater coral reef, teals and aqua blues",
  "Misty mountain morning, soft grays with cool accents",
  "Matrix terminal, green on black with phosphor glow effect",
  "Vaporwave aesthetic, pink purple and cyan retro",
  "Synthwave sunset, purple pink orange gradient vibes",
  "Coffee shop aesthetic, rich espresso browns, latte creams, caramel and honey tones, cozy warm neutrals",
  "Midnight coder, very dark with subtle colorful accents",
}

function M.get_random_preset()
  math.randomseed(os.time())
  return M.PRESETS[math.random(#M.PRESETS)]
end

function M.validate_accessibility(colorscheme)
  local warnings = {}
  if not colorscheme or type(colorscheme) ~= "table" then
    return false, { "Colorscheme data missing" }
  end
  local colors = colorscheme.colors
  if not colors or type(colors) ~= "table" then
    return false, { "Colors table missing" }
  end

  if not colors.bg then
    table.insert(warnings, "Background color (bg) missing")
  end
  if not colors.fg then
    table.insert(warnings, "Foreground color (fg) missing")
  end

  if colors.bg == "#000000" then
    table.insert(warnings, "Background uses pure black; prefer deep gray or justify true black explicitly")
  end
  if colors.fg == "#ffffff" then
    table.insert(warnings, "Foreground uses pure white; soften slightly unless pure white is intentional")
  end

  if colors.bg and colors.fg then
    local ratio = color_utils.get_contrast_ratio(colors.bg, colors.fg)
    if ratio < 7 then
      table.insert(warnings, string.format("Background-to-foreground contrast is %.1f:1, should be ≥7:1", ratio))
    end
  end

  if colors.comment and colors.bg then
    local comment_ratio = color_utils.get_contrast_ratio(colors.comment, colors.bg)
    if comment_ratio < 4.5 then
      table.insert(warnings, string.format("Comment contrast is %.1f:1, should be ≥4.5:1", comment_ratio))
    end
  end

  local required = { "red", "orange", "yellow", "green", "cyan", "blue", "purple", "magenta" }
  for _, key in ipairs(required) do
    local value = colors[key]
    if not value then
      table.insert(warnings, string.format("Missing %s color", key))
    elseif colors.bg then
      local ratio = color_utils.get_contrast_ratio(value, colors.bg)
      if ratio < 4.5 then
        table.insert(warnings, string.format("%s contrast is %.1f:1 against background, should be ≥4.5:1", key, ratio))
      end
    end
  end

  return #warnings == 0, warnings
end

return M

