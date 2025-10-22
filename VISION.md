# **hexwitch.nvim - Product Requirements Document (PRD)**

---

## **1. Product Overview**

### **1.1 Product Name**
**hexwitch.nvim** - AI-powered theme generator for Neovim

### **1.2 Tagline**
*"Cast the perfect colorscheme spell"*

### **1.3 Product Vision**
hexwitch.nvim transforms how developers interact with Neovim themes by allowing them to describe their desired aesthetic in natural language and instantly apply AI-generated colorschemes. The product eliminates the friction of browsing hundreds of themes or manually tweaking color values.

### **1.4 Target Users**
- **Primary**: Neovim users who frequently switch themes or are searching for "the perfect theme"
- **Secondary**: Creative developers who enjoy customization
- **Tertiary**: New Neovim users exploring personalization options

### **1.5 Core Value Proposition**
- **For users who want novelty**: Fresh themes on demand
- **For users who want specific aesthetics**: Natural language to perfect colorscheme
- **For users who want to experiment**: Zero-friction theme iteration

---

## **2. Feature Specification**

### **2.1 Core Features (v1.0 - MVP)**

#### **Feature 1: Theme Generation**

**Description**: Generate and apply colorschemes from natural language descriptions

**User Commands**:
```vim
:Hexwitch                    " Opens interactive prompt
:Hexwitch <description>      " Direct generation (skip UI)
:HexwitchQuick              " Variation of last theme
:HexwitchRandom             " Random theme generation
```

**Input Methods**:
- Floating window with text input
- Direct command with inline description
- Quick actions (buttons/shortcuts)

**Expected Outputs**:
- Applied colorscheme (immediately visible)
- Theme metadata (name, colors, timestamp)
- Success/error feedback

---

#### **Feature 2: Theme Refinement**

**Description**: Iteratively improve generated themes without starting over

**User Commands**:
```vim
:HexwitchRefine             " Opens refinement UI
:HexwitchRefine <changes>   " Direct refinement description
```

**Refinement Options**:
- Quick adjustments (buttons): Increase contrast, warmer, cooler, more vibrant, more muted
- Natural language changes: "make background darker", "less blue"
- Undo/redo stack

---

#### **Feature 3: Theme Persistence**

**Description**: Save and recall generated themes

**User Commands**:
```vim
:HexwitchSave <name>        " Save current theme
:HexwitchLoad <name>        " Load saved theme
:HexwitchList               " Browse saved themes
:HexwitchHistory            " View generation history
:HexwitchDelete <name>      " Delete saved theme
```

**Storage Location**:
```
~/.local/share/nvim/hexwitch/
├── themes/                 " Saved themes (JSON)
│   ├── cyberpunk_neon.json
│   ├── forest_dawn.json
│   └── ...
├── history.json            " Generation history
└── state.json             " Plugin state
```

---

#### **Feature 4: Auto-Prompt Configuration**

**Description**: Configurable automatic theme suggestions

**Configuration Options**:
```lua
require('hexwitch').setup({
  prompt_frequency = "manual",     -- "manual" | "daily" | "weekly" | "never"
  prompt_on_startup = false,       -- Show prompt on first launch of day/week
  prompt_timing = "immediate",     -- "immediate" | "delayed"
  prompt_delay_seconds = 180,      -- If "delayed", wait this long
  show_welcome = true,             -- First-time setup wizard
})
```

**Frequency Modes**:

| Mode | Behavior | Persistence |
|------|----------|-------------|
| `manual` | No auto-prompts, user initiates all actions | N/A |
| `daily` | Prompt once per calendar day on first launch | Save last prompt date |
| `weekly` | Prompt once per 7 days | Save last prompt date |
| `never` | No prompts, even first-time welcome | Flag set permanently |

---

#### **Feature 5: AI Provider Configuration**

**Description**: Support multiple AI backends

**Configuration**:
```lua
require('hexwitch').setup({
  ai_provider = "openai",         -- "openai" | "openrouter" | "anthropic" | "ollama" | "custom"
  api_key = nil,                   -- Or use env var HEXWITCH_API_KEY
  model = "gpt-4",
  timeout = 30,                    -- Request timeout in seconds
  fallback_provider = "openrouter", -- Fallback if primary fails
})
```

**Supported Providers**:
- **OpenAI**: GPT models, API key required
- **OpenRouter**: Multiple model access, API key required  
- **Anthropic Claude**: Premium quality, API key required
- **Ollama**: Local models, no API key, works offline
- **Custom**: User-defined endpoint

---

### **2.2 Optional Features (v2.0+)**

#### **Feature 6: Theme Preview Mode**
- Preview theme in split window before applying
- Side-by-side comparison with current theme

#### **Feature 7: Smart Timing**
- Delay prompts until user settles in (after first save, etc.)
- Don't interrupt during active typing

#### **Feature 8: Theme Sharing**
- Export theme as shareable JSON
- Import community themes
- Optional: Cloud sync (encrypted)

#### **Feature 9: Treesitter Integration**
- Apply colors to treesitter highlight groups
- Language-specific syntax customization

#### **Feature 10: Accessibility Mode**
- WCAG contrast checking
- Colorblind-friendly palette generation
- High contrast mode toggle

---

## **3. User Experience & Flows**

### **3.1 First-Time User Experience (FTUE)**

#### **Step 1: Installation**

User installs via package manager:
```lua
-- lazy.nvim
{
  'yourusername/hexwitch.nvim',
  dependencies = { 'nvim-lua/plenary.nvim' },
  config = function()
    require('hexwitch').setup({
      ai_provider = "anthropic",
      api_key = os.getenv("ANTHROPIC_API_KEY"),
    })
  end
}
```

#### **Step 2: First Launch Welcome**

On first Neovim launch after installation:

```
┌─────────────────────────────────────────────────────────────┐
│  ✨ Welcome to hexwitch.nvim!                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Cast the perfect colorscheme spell with AI                 │
│                                                             │
│  Before we begin, a quick question:                         │
│                                                             │
│  How often would you like theme suggestions?                │
│                                                             │
│  ● Manual only - I'll use :Hexwitch when I want (Default)   │
│  ○ Daily - Fresh theme each morning                         │
│  ○ Weekly - New theme every week                            │
│  ○ Never - Just commands, no prompts                        │
│                                                             │
│  (Change anytime with :HexwitchConfig)                      │
│                                                             │
│  [Continue]  [Skip Setup]                                   │
└─────────────────────────────────────────────────────────────┘
```

**User Actions**:
- Selects frequency preference
- Clicks "Continue"

**System Actions**:
- Saves preference to `~/.local/share/nvim/hexwitch/state.json`
- Sets `show_welcome = false` to not show again

#### **Step 3: First Theme Generation Prompt**

```
┌─────────────────────────────────────────────────────────────┐
│  🔮 Cast Your First Theme                                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Describe the vibe you want:                                │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ [Click examples below or type your own...]            │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  💡 Try these:                                              │
│  • cozy coffee shop with warm browns and cream             │
│  • cyberpunk neon with deep purples and electric blue      │
│  • forest at dawn - soft greens and golden light           │
│  • minimal monochrome for focused coding                   │
│  • retro terminal green on black                           │
│                                                             │
│  [?] Tips for great themes                                  │
│                                                             │
│  [Generate Theme]  [Maybe Later]                            │
└─────────────────────────────────────────────────────────────┘
```

**Clicking an example**:
- Fills input field with example text
- User can edit before generating

**Clicking [?] Tips**:
- Shows inline help overlay

**User Actions**:
- Types or selects description
- Clicks "Generate Theme"

**System Actions**:
- Shows loading state
- Calls AI API
- Applies generated theme
- Shows success feedback

---

### **3.2 Core User Flow: Manual Theme Generation**

#### **Flow Diagram**:
```
User types :Hexwitch
    ↓
[Input Prompt UI]
    ↓
User enters description
    ↓
User clicks "Generate"
    ↓
[Loading State: "Brewing colors... ✨"]
    ↓
AI API Call (3-5 seconds)
    ↓
[Success] → Apply theme → [Feedback UI]
    OR
[Error] → [Error UI with retry options]
```

#### **Step 1: Command Entry**
```vim
:Hexwitch
```

#### **Step 2: Input Prompt**

```
┌─────────────────────────────────────────────────────────────┐
│  🔮 hexwitch.nvim                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Describe your theme:                                       │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ sunset over ocean with warm oranges and deep blues_   │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  💡 Examples • 📋 Recent: "cyberpunk neon..." • [?] Help   │
│                                                             │
│  [Generate Theme]  [Random]  [Cancel]                       │
└─────────────────────────────────────────────────────────────┘
```

**Interactions**:
- **Text input**: Standard text editing (Ctrl+w to delete word, etc.)
- **Examples link**: Shows dropdown with 5 quick examples
- **Recent link**: Shows last 3 used prompts
- **Help link**: Shows inline tips overlay
- **Random button**: Generates theme with prompt "surprise me with creative colors"

#### **Step 3: Loading State**

```
┌─────────────────────────────────────────────────────────────┐
│  🔮 hexwitch.nvim                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                  ✨ Brewing colors... ✨                    │
│                                                             │
│              [████████░░░░░░░░░░░░░░] 40%                  │
│                                                             │
│          (Talking to AI, takes 3-5 seconds)                 │
│                                                             │
│                    [Cancel Request]                         │
└─────────────────────────────────────────────────────────────┘
```

**Features**:
- Animated progress bar (fake progress for UX, based on timeout)
- Cancel button (aborts API request)
- Estimated time display

#### **Step 4a: Success State**

Theme applies immediately, then shows feedback:

```
┌─────────────────────────────────────────────────────────────┐
│  ✓ Theme Applied: "Ocean Sunset"                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  How does it feel?                                          │
│                                                             │
│  [❤️ Love it!]  [🔄 Tweak it]  [🎲 Try Again]  [↩ Undo]    │
│                                                             │
│  [💾 Save Theme]  [Share]  [Close]                          │
└─────────────────────────────────────────────────────────────┘
```

**Auto-dismiss**: Closes after 5 seconds unless user interacts

**Button Actions**:
- **Love it**: Saves to history, closes UI, shows subtle "Saved ✓" notification
- **Tweak it**: Opens refinement UI (see section 3.3)
- **Try Again**: Keeps same prompt, regenerates with variation
- **Undo**: Reverts to previous theme
- **Save Theme**: Opens save dialog with name input
- **Share**: Exports theme JSON to clipboard
- **Close**: Just closes UI

#### **Step 4b: Error State**

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Theme Generation Failed                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Couldn't reach the AI service                              │
│                                                             │
│  Possible reasons:                                          │
│  • No internet connection                                   │
│  • API key invalid or quota exceeded                        │
│  • Service temporarily down                                 │
│                                                             │
│  What would you like to do?                                 │
│                                                             │
│  [Try Again]  [Use Fallback (Ollama)]  [Check Settings]    │
│                                                             │
│  [Close]                                                     │
└─────────────────────────────────────────────────────────────┘
```

**Button Actions**:
- **Try Again**: Retries same request
- **Use Fallback**: Switches to Ollama (local) if configured
- **Check Settings**: Opens config file or shows current API key status
- **Close**: Dismisses error

---

### **3.3 Refinement Flow**

#### **Trigger**: User clicks "🔄 Tweak it" from success feedback

```
┌─────────────────────────────────────────────────────────────┐
│  🔧 Refine Theme: "Ocean Sunset"                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Quick adjustments:                                         │
│                                                             │
│  Contrast:    [Increase]  [Decrease]                        │
│  Temperature: [Warmer]    [Cooler]                          │
│  Saturation:  [More Vibrant]  [More Muted]                  │
│  Brightness:  [Lighter]   [Darker]                          │
│                                                             │
│  Or describe specific changes:                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ make the background darker and comments less gray_    │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  [Apply Changes]  [Reset to Original]  [Cancel]            │
└─────────────────────────────────────────────────────────────┘
```

**Quick Adjustment Behavior**:
- Clicking button immediately applies change (no API call for quick adjustments)
- Uses algorithmic color transformations (HSL adjustments)
- Multiple clicks stack (e.g., "Warmer" twice = +20 hue shift)

**Custom Changes Behavior**:
- Sends current theme + change description to AI
- Shows loading state
- Applies refined theme

**User Can**:
- Make multiple quick adjustments before committing
- Preview changes in real-time (optional v2.0 feature)
- Reset to original theme
- Chain refinements (refine the refined theme)

---

### **3.4 Auto-Prompt Flow (Daily/Weekly Mode)**

#### **Trigger**: User opens Neovim, 24+ hours since last prompt (daily mode)

**Timing Options**:

**Immediate (Default)**:
Shows 2 seconds after Neovim fully loads

**Delayed**:
Shows after user has been coding for 3 minutes (configurable)

#### **Prompt UI**:

**Morning (6am-12pm)**:
```
┌─────────────────────────────────────────────────────────────┐
│  ☀️ Fresh theme for a fresh day?                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Describe today's coding vibe...                            │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                                                       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  Quick picks:                                               │
│  • "energizing citrus colors"                              │
│  • "calm morning blue"                                     │
│                                                             │
│  [Generate]  [Keep Current]  [Skip Today]  [Settings]      │
└─────────────────────────────────────────────────────────────┘
```

**Evening (6pm-12am)**:
```
┌─────────────────────────────────────────────────────────────┐
│  🌙 Evening theme for relaxed coding?                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Describe tonight's vibe...                                 │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                                                       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  Quick picks:                                               │
│  • "cozy warm dim colors"                                  │
│  • "deep twilight blues"                                   │
│                                                             │
│  [Generate]  [Keep Current]  [Skip Today]  [Settings]      │
└─────────────────────────────────────────────────────────────┘
```

**Button Actions**:
- **Generate**: Proceeds with theme generation
- **Keep Current**: Closes prompt, updates timestamp (won't show again today)
- **Skip Today**: Closes prompt, updates timestamp, doesn't affect preference
- **Settings**: Opens frequency settings, allows changing to manual/weekly

#### **Adaptive Behavior: Dismissal Fatigue Detection**

If user clicks "Keep Current" or "Skip Today" **5 times consecutively**:

```
┌─────────────────────────────────────────────────────────────┐
│  💭 We've noticed you've skipped the last 5 prompts...      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Would you prefer to switch to manual mode?                 │
│                                                             │
│  You can always use :Hexwitch when you want a new theme.    │
│                                                             │
│  [Yes, Switch to Manual]  [No, Keep Daily]  [Try Weekly]   │
└─────────────────────────────────────────────────────────────┘
```

**User Selection Updates Config**

---

### **3.5 Theme Management Flow**

#### **Saving a Theme**

**Trigger**: User clicks "💾 Save Theme" or types `:HexwitchSave`

```
┌─────────────────────────────────────────────────────────────┐
│  💾 Save Theme                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Theme name:                                                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ ocean_sunset_                                         │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  Original prompt: "sunset over ocean with warm oranges..." │
│                                                             │
│  [Save]  [Cancel]                                           │
└─────────────────────────────────────────────────────────────┘
```

**Validation**:
- Name must be alphanumeric + underscores
- Cannot be empty
- If name exists, prompt to overwrite

**Storage Format** (`~/.local/share/nvim/hexwitch/themes/ocean_sunset.json`):
```json
{
  "name": "ocean_sunset",
  "prompt": "sunset over ocean with warm oranges and deep blues",
  "generated_at": "2025-10-21T09:15:30Z",
  "colors": {
    "background": "#1a2332",
    "foreground": "#e8d5b7",
    "cursor": "#ff6b35",
    "accents": [
      "#ff6b35",
      "#f7931e",
      "#fbb040",
      "#4a90e2",
      "#357abd",
      "#2e5c8a",
      "#1d3557",
      "#a8dadc"
    ]
  },
  "highlight_groups": {
    "Normal": { "fg": "#e8d5b7", "bg": "#1a2332" },
    "Comment": { "fg": "#6c7a89", "italic": true },
    "Function": { "fg": "#4a90e2" },
    "String": { "fg": "#fbb040" },
    // ... more highlight groups
  },
  "metadata": {
    "ai_provider": "anthropic",
    "model": "claude-sonnet-4-5-20250929",
    "generation_time_ms": 3240
  }
}
```

#### **Loading a Theme**

**Command**: `:HexwitchLoad ocean_sunset`

**Or browse with**: `:HexwitchList`

```
┌─────────────────────────────────────────────────────────────┐
│  📚 Saved Themes                                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ● ocean_sunset          [Preview] [Load] [Delete]          │
│    "sunset over ocean with warm oranges..."                │
│    Created: Oct 21, 2025                                    │
│                                                             │
│  ● cyberpunk_neon        [Preview] [Load] [Delete]          │
│    "cyberpunk neon with deep purples..."                   │
│    Created: Oct 20, 2025                                    │
│                                                             │
│  ● forest_dawn           [Preview] [Load] [Delete]          │
│    "forest at dawn - soft greens..."                       │
│    Created: Oct 18, 2025                                    │
│                                                             │
│  [Close]                                                     │
└─────────────────────────────────────────────────────────────┘
```

**Button Actions**:
- **Preview**: Shows color palette in floating window
- **Load**: Applies theme immediately
- **Delete**: Confirms deletion, removes file

---

### **3.6 History & Undo Flow**

#### **Viewing History**

**Command**: `:HexwitchHistory`

```
┌─────────────────────────────────────────────────────────────┐
│  📜 Generation History (Last 20)                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. "sunset over ocean..." - 5 min ago      [Load] [Save]  │
│  2. "cyberpunk neon..." - 2 hours ago       [Load] [Save]  │
│  3. "forest at dawn..." - Yesterday         [Load] [Save]  │
│  4. "minimal monochrome..." - 2 days ago    [Load] [Save]  │
│                                                             │
│  [Clear History]  [Close]                                   │
└─────────────────────────────────────────────────────────────┘
```

**Storage**: `~/.local/share/nvim/hexwitch/history.json` (keeps last 50 generations)

#### **Undo/Redo**

**Commands**:
```vim
:HexwitchUndo     " Revert to previous theme
:HexwitchRedo     " Reapply undone theme
```

**Undo Stack**:
- Maintains stack of last 10 applied themes in current session
- Persists across sessions in state file
- Shows notification: "↩ Reverted to previous theme"

---

## **4. Technical Specifications**

### **4.1 Architecture**

```
hexwitch.nvim/
├── lua/
│   └── hexwitch/
│       ├── init.lua              # Plugin entry point, setup()
│       ├── config.lua            # Configuration management
│       ├── ui/
│       │   ├── input.lua         # Input prompt floating window
│       │   ├── feedback.lua      # Success/error/loading states
│       │   ├── refinement.lua    # Refinement UI
│       │   ├── list.lua          # Theme list browser
│       │   └── components.lua    # Reusable UI components
│       ├── ai/
│       │   ├── providers/
│       │   │   ├── anthropic.lua # Claude integration
│       │   │   ├── openai.lua    # OpenAI integration
│       │   │   ├── ollama.lua    # Ollama integration
│       │   │   └── custom.lua    # Custom endpoint
│       │   ├── client.lua        # HTTP client wrapper
│       │   └── prompt.lua        # Prompt engineering
│       ├── theme/
│       │   ├── generator.lua     # Theme generation logic
│       │   ├── applicator.lua    # Apply colors to Neovim
│       │   ├── parser.lua        # Parse AI responses
│       │   ├── validator.lua     # Validate color schemes
│       │   └── transformer.lua   # Quick adjustments (HSL)
│       ├── storage/
│       │   ├── themes.lua        # Save/load themes
│       │   ├── history.lua       # Generation history
│       │   └── state.lua         # Plugin state management
│       ├── commands.lua          # Vim commands registration
│       ├── autoprompt.lua        # Auto-prompt logic
│       └── utils.lua             # Utility functions
├── plugin/
│   └── hexwitch.vim              # Vim commands definition
├── doc/
│   └── hexwitch.txt              # Help documentation
└── README.md
```

### **4.2 Data Models**

#### **Theme Object**
```lua
{
  name = "ocean_sunset",
  prompt = "sunset over ocean...",
  generated_at = "2025-10-21T09:15:30Z",
  colors = {
    background = "#1a2332",
    foreground = "#e8d5b7",
    cursor = "#ff6b35",
    accents = { "#ff6b35", "#f7931e", ... }  -- 8 colors
  },
  highlight_groups = {
    Normal = { fg = "#e8d5b7", bg = "#1a2332" },
    Comment = { fg = "#6c7a89", italic = true },
    -- ... ~50 core highlight groups
  },
  metadata = {
    ai_provider = "anthropic",
    model = "claude-sonnet-4-5-20250929",
    generation_time_ms = 3240
  }
}
```

#### **State Object** (`~/.local/share/nvim/hexwitch/state.json`)
```json
{
  "version": "1.0.0",
  "config": {
    "prompt_frequency": "manual",
    "show_welcome": false,
    "last_prompt_date": "2025-10-21",
    "dismissal_count": 0
  },
  "current_theme": { /* theme object */ },
  "undo_stack": [ /* array of theme objects */ ],
  "redo_stack": [ /* array of theme objects */ ],
  "stats": {
    "themes_generated": 47,
    "themes_saved": 12,
    "favorite_provider": "anthropic"
  }
}
```

#### **History Object** (`~/.local/share/nvim/hexwitch/history.json`)
```json
{
  "entries": [
    {
      "timestamp": "2025-10-21T09:15:30Z",
      "prompt": "sunset over ocean...",
      "theme": { /* theme object */ },
      "saved": true,
      "saved_name": "ocean_sunset"
    },
    // ... up to 50 entries
  ]
}
```

### **4.3 AI Integration**

#### **Request Format (Anthropic Claude)**

```lua
-- HTTP POST to https://api.anthropic.com/v1/messages
{
  model = "claude-sonnet-4-5-20250929",
  max_tokens = 1024,
  messages = {
    {
      role = "user",
      content = [[
You are a professional colorscheme designer for code editors.

Generate a Neovim theme based on this description:
"sunset over ocean with warm oranges and deep blues"

Requirements:
1. Background and foreground must have WCAG AA contrast (4.5:1 minimum)
2. Generate exactly 8 distinct accent colors for syntax highlighting
3. Colors should evoke the described mood/aesthetic
4. Ensure readability for long coding sessions

Return ONLY valid JSON with this exact structure:
{
  "name": "descriptive_theme_name",
  "background": "#hexcolor",
  "foreground": "#hexcolor",
  "cursor": "#hexcolor",
  "accents": ["#hex1", "#hex2", "#hex3", "#hex4", "#hex5", "#hex6", "#hex7", "#hex8"]
}

No explanations, just JSON.
]]
    }
  }
}
```

#### **Response Parsing**

```lua
-- Expected response
{
  "id": "msg_...",
  "content": [
    {
      "type": "text",
      "text": '{"name":"ocean_sunset","background":"#1a2332",...}'
    }
  ]
}

-- Parser extracts JSON, validates structure, ensures all required fields
```

#### **Error Handling**

```lua
-- Possible errors
{
  "API_KEY_INVALID",      -- 401 response
  "RATE_LIMIT_EXCEEDED",  -- 429 response
  "SERVICE_UNAVAILABLE",  -- 503 response
  "TIMEOUT",              -- No response in 30s
  "INVALID_JSON",         -- AI returned malformed JSON
  "MISSING_COLORS",       -- JSON missing required fields
  "INVALID_HEX",          -- Color not valid hex format
  "LOW_CONTRAST"          -- Generated colors fail WCAG check
}

-- Each error has specific user-facing message and recovery action
```

### **4.4 Color Application Logic**

#### **Core Highlight Groups (50+ groups)**

```lua
local highlight_groups = {
  -- Editor UI
  Normal = { fg = colors.foreground, bg = colors.background },
  NormalFloat = { fg = colors.foreground, bg = lighten(colors.background, 5) },
  FloatBorder = { fg = colors.accents[1] },
  Cursor = { fg = colors.background, bg = colors.cursor },
  CursorLine = { bg = lighten(colors.background, 3) },
  ```lua
  CursorLineNr = { fg = colors.accents[1], bold = true },
  LineNr = { fg = desaturate(colors.foreground, 60) },
  Visual = { bg = colors.accents[4], fg = colors.background },
  VisualNOS = { bg = colors.accents[4], fg = colors.background },
  Search = { bg = colors.accents[2], fg = colors.background },
  IncSearch = { bg = colors.accents[1], fg = colors.background },
  
  -- Status & Tab lines
  StatusLine = { fg = colors.foreground, bg = lighten(colors.background, 10) },
  StatusLineNC = { fg = desaturate(colors.foreground, 40), bg = lighten(colors.background, 5) },
  TabLine = { fg = colors.foreground, bg = lighten(colors.background, 5) },
  TabLineSel = { fg = colors.background, bg = colors.accents[1], bold = true },
  TabLineFill = { bg = colors.background },
  
  -- Syntax highlighting
  Comment = { fg = desaturate(colors.foreground, 50), italic = true },
  Constant = { fg = colors.accents[1] },
  String = { fg = colors.accents[2] },
  Character = { fg = colors.accents[2] },
  Number = { fg = colors.accents[3] },
  Boolean = { fg = colors.accents[3] },
  Float = { fg = colors.accents[3] },
  
  Identifier = { fg = colors.accents[4] },
  Function = { fg = colors.accents[5], bold = true },
  
  Statement = { fg = colors.accents[6] },
  Conditional = { fg = colors.accents[6] },
  Repeat = { fg = colors.accents[6] },
  Label = { fg = colors.accents[6] },
  Operator = { fg = colors.foreground },
  Keyword = { fg = colors.accents[6], bold = true },
  Exception = { fg = colors.accents[6] },
  
  PreProc = { fg = colors.accents[7] },
  Include = { fg = colors.accents[7] },
  Define = { fg = colors.accents[7] },
  Macro = { fg = colors.accents[7] },
  PreCondit = { fg = colors.accents[7] },
  
  Type = { fg = colors.accents[8] },
  StorageClass = { fg = colors.accents[8] },
  Structure = { fg = colors.accents[8] },
  Typedef = { fg = colors.accents[8] },
  
  Special = { fg = colors.accents[1] },
  SpecialChar = { fg = colors.accents[1] },
  Tag = { fg = colors.accents[5] },
  Delimiter = { fg = colors.foreground },
  SpecialComment = { fg = colors.accents[2], italic = true },
  Debug = { fg = colors.accents[1] },
  
  -- Messages & diagnostics
  Error = { fg = colors.accents[1], bold = true },
  ErrorMsg = { fg = colors.accents[1], bold = true },
  WarningMsg = { fg = colors.accents[2] },
  MoreMsg = { fg = colors.accents[5] },
  Question = { fg = colors.accents[5] },
  
  -- Diffs
  DiffAdd = { bg = blend(colors.accents[5], colors.background, 0.2) },
  DiffChange = { bg = blend(colors.accents[2], colors.background, 0.2) },
  DiffDelete = { bg = blend(colors.accents[1], colors.background, 0.2) },
  DiffText = { bg = blend(colors.accents[2], colors.background, 0.4), bold = true },
  
  -- Popup menu
  Pmenu = { fg = colors.foreground, bg = lighten(colors.background, 8) },
  PmenuSel = { fg = colors.background, bg = colors.accents[1], bold = true },
  PmenuSbar = { bg = lighten(colors.background, 15) },
  PmenuThumb = { bg = colors.accents[1] },
  
  -- Spell checking
  SpellBad = { undercurl = true, sp = colors.accents[1] },
  SpellCap = { undercurl = true, sp = colors.accents[2] },
  SpellLocal = { undercurl = true, sp = colors.accents[5] },
  SpellRare = { undercurl = true, sp = colors.accents[8] },
  
  -- LSP semantic tokens (v2.0)
  ["@variable"] = { fg = colors.foreground },
  ["@variable.builtin"] = { fg = colors.accents[3] },
  ["@variable.parameter"] = { fg = colors.accents[4] },
  ["@variable.member"] = { fg = colors.accents[4] },
  
  ["@constant"] = { fg = colors.accents[1] },
  ["@constant.builtin"] = { fg = colors.accents[1], bold = true },
  ["@constant.macro"] = { fg = colors.accents[7] },
  
  ["@string"] = { fg = colors.accents[2] },
  ["@string.escape"] = { fg = colors.accents[3] },
  ["@string.regexp"] = { fg = colors.accents[2], italic = true },
  
  ["@character"] = { fg = colors.accents[2] },
  ["@number"] = { fg = colors.accents[3] },
  ["@boolean"] = { fg = colors.accents[3] },
  
  ["@function"] = { fg = colors.accents[5], bold = true },
  ["@function.builtin"] = { fg = colors.accents[5] },
  ["@function.macro"] = { fg = colors.accents[7] },
  ["@function.method"] = { fg = colors.accents[5] },
  
  ["@keyword"] = { fg = colors.accents[6], bold = true },
  ["@keyword.function"] = { fg = colors.accents[6] },
  ["@keyword.operator"] = { fg = colors.accents[6] },
  ["@keyword.return"] = { fg = colors.accents[6], bold = true },
  
  ["@type"] = { fg = colors.accents[8] },
  ["@type.builtin"] = { fg = colors.accents[8], bold = true },
  
  ["@property"] = { fg = colors.accents[4] },
  ["@attribute"] = { fg = colors.accents[7] },
  
  ["@comment"] = { fg = desaturate(colors.foreground, 50), italic = true },
  ["@comment.error"] = { fg = colors.accents[1], bold = true },
  ["@comment.warning"] = { fg = colors.accents[2], bold = true },
  ["@comment.note"] = { fg = colors.accents[5], bold = true },
  ["@comment.todo"] = { fg = colors.accents[2], bold = true },
  
  -- Plugin support
  -- GitSigns
  GitSignsAdd = { fg = colors.accents[5] },
  GitSignsChange = { fg = colors.accents[2] },
  GitSignsDelete = { fg = colors.accents[1] },
  
  -- Telescope
  TelescopeBorder = { fg = colors.accents[1] },
  TelescopePromptBorder = { fg = colors.accents[1] },
  TelescopeResultsBorder = { fg = colors.accents[4] },
  TelescopePreviewBorder = { fg = colors.accents[5] },
  TelescopeSelection = { fg = colors.background, bg = colors.accents[1], bold = true },
  TelescopeMatching = { fg = colors.accents[2], bold = true },
  
  -- NvimTree
  NvimTreeFolderName = { fg = colors.accents[5] },
  NvimTreeOpenedFolderName = { fg = colors.accents[5], bold = true },
  NvimTreeRootFolder = { fg = colors.accents[1], bold = true },
  NvimTreeGitDirty = { fg = colors.accents[2] },
  NvimTreeGitNew = { fg = colors.accents[5] },
  NvimTreeGitDeleted = { fg = colors.accents[1] },
  
  -- Diagnostic (LSP)
  DiagnosticError = { fg = colors.accents[1] },
  DiagnosticWarn = { fg = colors.accents[2] },
  DiagnosticInfo = { fg = colors.accents[5] },
  DiagnosticHint = { fg = colors.accents[4] },
  DiagnosticUnderlineError = { undercurl = true, sp = colors.accents[1] },
  DiagnosticUnderlineWarn = { undercurl = true, sp = colors.accents[2] },
  DiagnosticUnderlineInfo = { undercurl = true, sp = colors.accents[5] },
  DiagnosticUnderlineHint = { undercurl = true, sp = colors.accents[4] },
}
```

#### **Color Utility Functions**

```lua
-- Lighten/darken colors
function lighten(hex, percent)
  local h, s, l = hex_to_hsl(hex)
  l = math.min(100, l + percent)
  return hsl_to_hex(h, s, l)
end

function darken(hex, percent)
  local h, s, l = hex_to_hsl(hex)
  l = math.max(0, l - percent)
  return hsl_to_hex(h, s, l)
end

-- Adjust saturation
function saturate(hex, percent)
  local h, s, l = hex_to_hsl(hex)
  s = math.min(100, s + percent)
  return hsl_to_hex(h, s, l)
end

function desaturate(hex, percent)
  local h, s, l = hex_to_hsl(hex)
  s = math.max(0, s - percent)
  return hsl_to_hex(h, s, l)
end

-- Blend two colors
function blend(hex1, hex2, ratio)
  local r1, g1, b1 = hex_to_rgb(hex1)
  local r2, g2, b2 = hex_to_rgb(hex2)
  local r = r1 * ratio + r2 * (1 - ratio)
  local g = g1 * ratio + g2 * (1 - ratio)
  local b = b1 * ratio + b2 * (1 - ratio)
  return rgb_to_hex(r, g, b)
end

-- WCAG contrast ratio
function contrast_ratio(hex1, hex2)
  local l1 = relative_luminance(hex1)
  local l2 = relative_luminance(hex2)
  local lighter = math.max(l1, l2)
  local darker = math.min(l1, l2)
  return (lighter + 0.05) / (darker + 0.05)
end

function validate_contrast(foreground, background)
  local ratio = contrast_ratio(foreground, background)
  return ratio >= 4.5  -- WCAG AA standard
end
```

### **4.5 Configuration API**

#### **Full Configuration Schema**

```lua
require('hexwitch').setup({
  -- Auto-prompt settings
  prompt_frequency = "manual",      -- "manual" | "daily" | "weekly" | "never"
  prompt_on_startup = false,        -- Show on first launch of period
  prompt_timing = "immediate",      -- "immediate" | "delayed"
  prompt_delay_seconds = 180,       -- Delay before showing (if "delayed")
  contextual_prompts = false,       -- Time-of-day aware copy (v2.0)
  show_welcome = true,              -- First-time setup wizard
  
  -- AI provider settings
  ai_provider = "anthropic",        -- "anthropic" | "openai" | "ollama" | "custom"
  api_key = nil,                    -- Or use env var HEXWITCH_API_KEY
  model = "claude-sonnet-4-5-20250929",
  timeout = 30,                     -- Request timeout in seconds
  fallback_provider = nil,          -- Fallback if primary fails
  
  -- Custom endpoint (for "custom" provider)
  custom_endpoint = {
    url = "http://localhost:11434/api/generate",
    headers = {},
    request_format = function(prompt) 
      return { prompt = prompt }
    end,
    response_parser = function(response)
      return vim.json.decode(response.body)
    end
  },
  
  -- Theme settings
  contrast_threshold = 4.5,         -- Minimum WCAG contrast ratio
  accent_count = 8,                 -- Number of accent colors
  apply_treesitter = true,          -- Apply to treesitter groups (v2.0)
  apply_lsp_semantic = true,        -- Apply to LSP semantic tokens (v2.0)
  
  -- Storage settings
  storage_path = vim.fn.stdpath('data') .. '/hexwitch',
  max_history = 50,                 -- Maximum history entries
  auto_save_history = true,         -- Automatically save to history
  
  -- UI settings
  ui = {
    border = "rounded",             -- "none" | "single" | "double" | "rounded"
    width_ratio = 0.6,              -- Floating window width (% of screen)
    height_ratio = 0.4,             -- Floating window height (% of screen)
    icons = true,                   -- Show emoji/icons
    animations = false,             -- Animated progress bars (v2.0)
    preview_mode = false,           -- Show preview before applying (v2.0)
  },
  
  -- Keymaps (inside floating windows)
  keymaps = {
    close = "<Esc>",
    confirm = "<CR>",
    cancel = "<C-c>",
    next = "<Tab>",
    prev = "<S-Tab>",
  },
  
  -- Callbacks (for custom integrations)
  on_theme_applied = function(theme)
    -- Called after theme is applied
  end,
  on_theme_saved = function(theme, name)
    -- Called after theme is saved
  end,
  on_error = function(error_type, message)
    -- Called on errors
  end,
})
```

---

## **5. Edge Cases & Error Handling**

### **5.1 Network & API Errors**

#### **Case 1: No Internet Connection**

**Detection**: HTTP request times out or returns connection error

**User Experience**:
```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  No Internet Connection                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Couldn't reach the AI service.                             │
│                                                             │
│  Options:                                                   │
│  • Check your internet connection                           │
│  • Use local Ollama (offline mode)                          │
│  • Try again when online                                    │
│                                                             │
│  [Setup Ollama]  [Try Again]  [Cancel]                      │
└─────────────────────────────────────────────────────────────┘
```

**System Behavior**:
- If `fallback_provider = "ollama"` configured → automatically retry with Ollama
- Log error to `~/.local/share/nvim/hexwitch/error.log`
- Don't update last prompt timestamp (allow retry)

---

#### **Case 2: Invalid/Missing API Key**

**Detection**: 401 Unauthorized response

**User Experience**:
```
┌─────────────────────────────────────────────────────────────┐
│  🔑 API Key Issue                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Your API key is missing or invalid.                        │
│                                                             │
│  Current provider: Anthropic Claude                         │
│  API key: Not set                                           │
│                                                             │
│  To fix:                                                    │
│  1. Get API key from: https://console.anthropic.com         │
│  2. Set environment variable: ANTHROPIC_API_KEY             │
│     or add to config: api_key = "sk-ant-..."               │
│  3. Restart Neovim                                          │
│                                                             │
│  [Open Config]  [Documentation]  [Close]                    │
└─────────────────────────────────────────────────────────────┘
```

**System Behavior**:
- Show detailed setup instructions
- Link to provider documentation
- Offer to open config file for editing

---

#### **Case 3: Rate Limit Exceeded**

**Detection**: 429 Too Many Requests response

**User Experience**:
```
┌─────────────────────────────────────────────────────────────┐
│  ⏱️  Rate Limit Reached                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  You've hit the API rate limit.                             │
│                                                             │
│  Rate limit resets in: 45 seconds                           │
│                                                             │
│  Options:                                                   │
│  • Wait and try again                                       │
│  • Use local Ollama (no rate limits)                        │
│  • Upgrade your API plan                                    │
│                                                             │
│  [Wait & Retry]  [Use Ollama]  [Close]                      │
└─────────────────────────────────────────────────────────────┘
```

**System Behavior**:
- Parse `Retry-After` header if present
- Show countdown timer
- Automatically retry when limit resets (if user chooses)

---

#### **Case 4: AI Service Down (503)**

**Detection**: 503 Service Unavailable

**User Experience**:
```
┌─────────────────────────────────────────────────────────────┐
│  🔧 Service Temporarily Down                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  The AI service is experiencing issues.                     │
│                                                             │
│  This usually resolves within a few minutes.                │
│                                                             │
│  [Try Again]  [Check Status]  [Use Fallback]  [Close]       │
└─────────────────────────────────────────────────────────────┘
```

**System Behavior**:
- "Check Status" opens provider status page in browser
- "Use Fallback" switches to configured fallback provider
- Retry with exponential backoff if user chooses

---

### **5.2 AI Response Errors**

#### **Case 5: Invalid JSON Response**

**Detection**: AI returns non-JSON or malformed JSON

**Example Bad Response**:
```
Here's a nice theme for you:
{"name": "ocean_sunset", "background": "#1a2332"...
[Missing closing brace]
```

**System Behavior**:
- Attempt to fix common JSON errors (missing brackets, trailing commas)
- If unfixable, show error:

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Unexpected Response                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  The AI returned an invalid response.                       │
│                                                             │
│  This is usually temporary. Try again?                      │
│                                                             │
│  [Try Again]  [Report Issue]  [Close]                       │
└─────────────────────────────────────────────────────────────┘
```

- Log full response for debugging
- "Report Issue" opens GitHub issue template with anonymized response

---

#### **Case 6: Missing Required Colors**

**Detection**: JSON valid but missing fields

**Example**:
```json
{
  "name": "ocean_sunset",
  "background": "#1a2332",
  "foreground": "#e8d5b7"
  // Missing: cursor, accents
}
```

**System Behavior**:
- Attempt to generate missing colors algorithmically:
  - `cursor`: Use brightest accent or complementary color
  - `accents`: Generate from existing colors using color theory
- Show warning:

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Theme Applied with Adjustments                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Some colors were missing and auto-generated.               │
│                                                             │
│  The theme may not match your exact expectations.           │
│                                                             │
│  [Looks Good]  [Regenerate]  [Refine]                       │
└─────────────────────────────────────────────────────────────┘
```

---

#### **Case 7: Low Contrast Colors**

**Detection**: Contrast ratio between background/foreground < 4.5:1

**System Behavior**:
- Automatically adjust foreground lightness to meet WCAG AA
- Show notification:

```
✓ Theme adjusted for readability (contrast improved)
```

- If adjustment fails, reject theme:

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Poor Contrast Detected                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  This theme has very low contrast and may be hard to read.  │
│                                                             │
│  Contrast ratio: 2.1:1 (4.5:1 minimum recommended)          │
│                                                             │
│  [Try Again]  [Apply Anyway]  [Cancel]                      │
└─────────────────────────────────────────────────────────────┘
```

---

### **5.3 File System Errors**

#### **Case 8: Storage Directory Not Writable**

**Detection**: Cannot create `~/.local/share/nvim/hexwitch/`

**User Experience**:
```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Storage Error                                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Cannot write to storage directory:                         │
│  ~/.local/share/nvim/hexwitch/                              │
│                                                             │
│  Permissions issue? Try:                                    │
│  mkdir -p ~/.local/share/nvim/hexwitch                      │
│  chmod 755 ~/.local/share/nvim/hexwitch                     │
│                                                             │
│  Themes will work but won't be saved.                       │
│                                                             │
│  [Continue Anyway]  [Close]                                 │
└─────────────────────────────────────────────────────────────┘
```

**System Behavior**:
- Continue working with in-memory themes only
- Disable save/history features
- Log error for troubleshooting

---

#### **Case 9: Corrupted State File**

**Detection**: state.json exists but is invalid JSON

**System Behavior**:
- Backup corrupted file: `state.json.backup.20251021_091530`
- Create fresh state file
- Show notification:

```
⚠️  Recovered from corrupted state file (backup saved)
```

- Don't interrupt user workflow

---

### **5.4 Concurrent Access**

#### **Case 10: Multiple Neovim Instances**

**Scenario**: User has 3 Neovim windows open, all with hexwitch

**Problem**: Race conditions on shared files (state.json, history.json)

**Solution**: File locking mechanism

```lua
-- Acquire lock before writing
local lock_file = storage_path .. '/hexwitch.lock'

function write_with_lock(filepath, data)
  local lock = acquire_lock(lock_file, timeout = 2)
  if not lock then
    -- Another instance is writing, queue write
    schedule_write(filepath, data)
    return
  end
  
  -- Write safely
  write_file(filepath, data)
  release_lock(lock)
end
```

**Behavior**:
- Only one instance can write at a time
- Others wait or queue writes
- If lock held > 5 seconds, assume stale and override

---

### **5.5 User Input Edge Cases**

#### **Case 11: Empty Prompt**

**User Action**: Clicks "Generate" with empty input field

**System Behavior**:
- Don't send API request
- Shake input field (visual feedback)
- Show inline error: "⚠️ Please describe your theme"

---

#### **Case 12: Extremely Long Prompt**

**User Input**: "I want a theme that looks like..." (2000+ characters)

**System Behavior**:
- Truncate to 500 characters
- Show warning: "⚠️ Prompt truncated (max 500 chars)"
- Still process request

---

#### **Case 13: Special Characters in Theme Name**

**User Input**: Tries to save as `my-theme/v1.0!`

**System Behavior**:
- Sanitize name: `my_theme_v1_0`
- Show: "Saved as: my_theme_v1_0 (special characters removed)"

**Validation Rules**:
- Alphanumeric + underscores only
- Max 50 characters
- Cannot start with number
- Cannot be empty

---

#### **Case 14: Theme Name Collision**

**User Action**: Saves theme with name that already exists

**System Behavior**:
```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Theme Already Exists                                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  "ocean_sunset" already exists.                             │
│                                                             │
│  [Overwrite]  [Save As New]  [Cancel]                       │
└─────────────────────────────────────────────────────────────┘
```

**If "Save As New"**:
```
┌─────────────────────────────────────────────────────────────┐
│  💾 Save Theme                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Theme name:                                                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ ocean_sunset_2_                                       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  [Save]  [Cancel]                                           │
└─────────────────────────────────────────────────────────────┘
```

---

### **5.6 Plugin Conflicts**

#### **Case 15: Conflicting Colorscheme Plugins**

**Problem**: User has auto-loading colorscheme plugin that conflicts

**Detection**: Check if other colorscheme was applied within 100ms of hexwitch theme

**System Behavior**:
```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Theme Conflict Detected                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Another plugin is overriding hexwitch themes.              │
│                                                             │
│  Possible conflicts:                                        │
│  • colorscheme set in init.lua                             │
│  • Auto-loading theme plugins                              │
│                                                             │
│  Disable conflicting plugins for hexwitch to work properly. │
│                                                             │
│  [Show Help]  [Don't Show Again]  [Close]                   │
└─────────────────────────────────────────────────────────────┘
```

---

#### **Case 16: Missing Dependencies**

**Problem**: User doesn't have `plenary.nvim` installed

**Detection**: Check in setup()

**System Behavior**:
```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️  Missing Dependency                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  hexwitch requires: plenary.nvim                            │
│                                                             │
│  Install with your package manager:                         │
│                                                             │
│  -- lazy.nvim                                               │
│  { 'nvim-lua/plenary.nvim' }                                │
│                                                             │
│  [Documentation]  [Close]                                   │
└─────────────────────────────────────────────────────────────┘
```

**Fallback**: Disable plugin gracefully, don't crash

---

### **5.7 Auto-Prompt Edge Cases**

#### **Case 17: User Changes Timezone**

**Problem**: User travels, changes timezone

**Current timestamp**: `2025-10-21 09:00:00 PST`
**Last prompt**: `2025-10-21 08:00:00 EST`

**System Behavior**:
- Always use UTC internally
- Convert to local time for display only
- Avoid duplicate prompts

---

#### **Case 18: System Clock Changed**

**Problem**: User changes system time backwards

**Detection**: Last prompt timestamp is in the future

**System Behavior**:
- Reset last prompt timestamp to current time
- Log warning: "Clock change detected, reset prompt schedule"
- Don't spam prompts

---

#### **Case 19: First Use After Long Hiatus**

**Scenario**: User installs plugin, uses it, then doesn't open Neovim for 30 days

**System Behavior**:
- Don't show 30 accumulated daily prompts
- Show single prompt: "Welcome back! Been a while - want a fresh theme?"
- Reset schedule normally

---

### **5.8 Refinement Edge Cases**

#### **Case 20: Refining with Contradictory Instructions**

**User Input**: "make it darker" → User clicks "Lighter" button

**System Behavior**:
- Last action wins
- Show: "Making theme lighter..."
- Clear previous instruction

---

#### **Case 21: Too Many Refinements**

**Scenario**: User refines same theme 10+ times

**System Behavior**:
- After 5 refinements, suggest:

```
💡 Tip: Might be easier to start fresh?

[Continue Refining]  [Start Over]
```

- Don't prevent continued refinement, just suggest

---

### **5.9 History Edge Cases**

#### **Case 22: History File Too Large**

**Problem**: history.json exceeds 10MB (thousands of entries)

**System Behavior**:
- Keep only last 50 entries (configurable via `max_history`)
- Archive old entries: `history_archive_20251021.json`
- Show: "Archived old history (kept last 50 entries)"

---

#### **Case 23: Deleted Theme Still in History**

**Problem**: User deletes saved theme, but it's in history

**System Behavior**:
- History shows: `"ocean_sunset" (deleted)`
- "Load" button still works (loads from history data)
- "Save" button offers to restore it

---

## **6. Commands Reference**

### **6.1 Core Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:Hexwitch` | Open interactive prompt | `:
`:Hexwitch` |
| `:Hexwitch <description>` | Generate theme directly | `:Hexwitch cozy autumn forest` |
| `:HexwitchQuick` | Generate variation of last theme | `:HexwitchQuick` |
| `:HexwitchRandom` | Generate random theme | `:HexwitchRandom` |
| `:HexwitchRefine` | Open refinement UI | `:HexwitchRefine` |
| `:HexwitchRefine <changes>` | Refine with description | `:HexwitchRefine darker background` |

### **6.2 Theme Management Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchSave <name>` | Save current theme | `:HexwitchSave ocean_sunset` |
| `:HexwitchLoad <name>` | Load saved theme | `:HexwitchLoad ocean_sunset` |
| `:HexwitchList` | Browse saved themes | `:HexwitchList` |
| `:HexwitchDelete <name>` | Delete saved theme | `:HexwitchDelete ocean_sunset` |
| `:HexwitchExport <name>` | Export theme as JSON | `:HexwitchExport ocean_sunset` |
| `:HexwitchImport <path>` | Import theme from JSON | `:HexwitchImport ~/theme.json` |

### **6.3 History Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchHistory` | View generation history | `:HexwitchHistory` |
| `:HexwitchUndo` | Revert to previous theme | `:HexwitchUndo` |
| `:HexwitchRedo` | Reapply undone theme | `:HexwitchRedo` |
| `:HexwitchClearHistory` | Clear all history | `:HexwitchClearHistory` |

### **6.4 Configuration Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchConfig` | Open config UI | `:HexwitchConfig` |
| `:HexwitchSetFrequency <mode>` | Set prompt frequency | `:HexwitchSetFrequency daily` |
| `:HexwitchSetProvider <provider>` | Change AI provider | `:HexwitchSetProvider ollama` |
| `:HexwitchStatus` | Show plugin status | `:HexwitchStatus` |
| `:HexwitchDocs` | Open documentation | `:HexwitchDocs` |

### **6.5 Utility Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchInspect` | Show current theme details | `:HexwitchInspect` |
| `:HexwitchContrast` | Check contrast ratios | `:HexwitchContrast` |
| `:HexwitchDebug` | Show debug information | `:HexwitchDebug` |
| `:HexwitchVersion` | Show plugin version | `:HexwitchVersion` |

---

## **7. Data Flow Diagrams**

### **7.1 Theme Generation Flow**

```
┌─────────────┐
│   User      │
│ :Hexwitch   │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│   UI Layer          │
│ - Show input prompt │
│ - Collect user desc │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Prompt Builder     │
│ - Build AI prompt   │
│ - Add context       │
│ - Add requirements  │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│   AI Client         │
│ - Select provider   │
│ - Make HTTP request │
│ - Handle timeout    │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Response Parser    │
│ - Extract JSON      │
│ - Validate schema   │
│ - Fix common errors │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Theme Validator    │
│ - Check contrast    │
│ - Verify hex colors │
│ - Fill missing vals │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Theme Generator    │
│ - Map to hl groups  │
│ - Generate variants │
│ - Apply algorithms  │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Theme Applicator   │
│ - Set highlight grps│
│ - Apply immediately │
│ - Update UI state   │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Storage Manager    │
│ - Save to history   │
│ - Update undo stack │
│ - Update state file │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│   UI Feedback       │
│ - Show success      │
│ - Offer actions     │
└─────────────────────┘
```

### **7.2 Auto-Prompt Flow**

```
┌─────────────────┐
│ Neovim Startup  │
└────────┬────────┘
         │
         ▼
┌───────────────────────┐
│ hexwitch autoload     │
│ - Load state.json     │
│ - Check frequency cfg │
└────────┬──────────────┘
         │
         ▼
    ┌────────────┐
    │ Frequency? │
    └──┬──┬──┬───┘
       │  │  │
  manual daily weekly
       │  │  │
       │  ▼  ▼
       │ ┌──────────────────┐
       │ │ Check timestamp  │
       │ │ - Load last_date │
       │ │ - Compare to now │
       │ └────┬─────────────┘
       │      │
       │      ▼
       │  ┌────────────┐
       │  │ Time diff? │
       │  └──┬────┬────┘
       │     │    │
       │   Yes   No → Exit
       │     │
       │     ▼
       │ ┌──────────────────┐
       │ │ Check timing cfg │
       │ └────┬─────────────┘
       │      │
       │      ▼
       │  ┌────────────┐
       │  │ Immediate? │
       │  └──┬────┬────┘
       │     │    │
       │   Yes   No
       │     │    │
       │     │    ▼
       │     │ ┌──────────────┐
       │     │ │ Schedule     │
       │     │ │ delayed show │
       │     │ └──────────────┘
       │     │
       │     ▼
       │ ┌──────────────────┐
       │ │ Show prompt UI   │
       │ └────┬─────────────┘
       │      │
       │      ▼
       │  ┌─────────────┐
       │  │ User action │
       │  └──┬──┬───┬───┘
       │     │  │   │
       │  Generate Skip Settings
       │     │  │   │
       │     ▼  ▼   ▼
       │  [Follow standard flow]
       │     │
       │     ▼
       │ ┌──────────────────┐
       │ │ Update timestamp │
       │ │ Save to state    │
       │ └──────────────────┘
       │
       └─→ [Exit, no prompt]
```

### **7.3 Refinement Flow**

```
┌─────────────────┐
│ User clicks     │
│ "🔄 Tweak it"   │
└────────┬────────┘
         │
         ▼
┌─────────────────────┐
│ Load current theme  │
│ - Get colors        │
│ - Get metadata      │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Show refinement UI  │
│ - Quick adjustments │
│ - Custom input      │
└────────┬────────────┘
         │
         ▼
    ┌────────────┐
    │ User choice│
    └──┬────┬────┘
       │    │
   Quick  Custom
   button  text
       │    │
       ▼    ▼
   ┌──────┐ ┌──────────────┐
   │ HSL  │ │ Build prompt │
   │ Math │ │ with changes │
   └──┬───┘ └──┬───────────┘
      │        │
      │        ▼
      │    ┌──────────────┐
      │    │ Call AI API  │
      │    └──┬───────────┘
      │       │
      │       ▼
      │    ┌──────────────┐
      │    │ Parse result │
      │    └──┬───────────┘
      │       │
      └───────┴──────┐
                     │
                     ▼
            ┌────────────────┐
            │ Apply new theme│
            └────────┬───────┘
                     │
                     ▼
            ┌────────────────┐
            │ Update UI      │
            │ - Can refine   │
            │   again        │
            └────────────────┘
```

---

## **8. Success Metrics & Analytics**

### **8.1 User Engagement Metrics**

**Track (locally, privacy-preserving)**:

```lua
-- In state.json
{
  "stats": {
    "themes_generated": 47,
    "themes_saved": 12,
    "themes_refined": 8,
    "average_refinements_per_theme": 1.7,
    "most_used_prompts": [
      "cyberpunk neon",
      "forest dawn",
      "minimal dark"
    ],
    "favorite_provider": "anthropic",
    "daily_prompt_dismissals": 2,
    "weekly_prompt_dismissals": 0,
    "avg_generation_time_ms": 3240
  }
}
```

**Purpose**: Help users understand their usage patterns

**Display**: `:HexwitchStats` command shows:

```
┌─────────────────────────────────────────────────────────────┐
│  📊 Your hexwitch Stats                                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Themes generated: 47                                       │
│  Themes saved: 12                                           │
│  Average refinements: 1.7 per theme                         │
│                                                             │
│  Most used vibes:                                           │
│  1. cyberpunk neon (8 times)                               │
│  2. forest dawn (5 times)                                  │
│  3. minimal dark (4 times)                                 │
│                                                             │
│  Favorite provider: Anthropic                               │
│  Average generation time: 3.2 seconds                       │
│                                                             │
│  [Reset Stats]  [Close]                                     │
└─────────────────────────────────────────────────────────────┘
```

### **8.2 Quality Metrics**

**Track**:
- How often users refine themes (indicator of first-try quality)
- How often users undo/discard themes
- How often saved themes are reused
- Contrast ratio distribution

**Use Cases**:
- Improve AI prompts based on refinement patterns
- Detect if certain prompt types fail more often
- Optimize default settings

---

## **9. Security & Privacy**

### **9.1 API Key Handling**

**Best Practices**:

1. **Never log API keys**
```lua
-- Bad
print("API request with key: " .. api_key)

-- Good
print("API request sent")
```

2. **Prefer environment variables**
```lua
-- Priority order
api_key = config.api_key           -- 1. Explicit config
       or os.getenv("ANTHROPIC_API_KEY")  -- 2. Env var
       or os.getenv("HEXWITCH_API_KEY")   -- 3. Generic env
       or nil                              -- 4. Prompt user
```

3. **Warn on insecure storage**
```lua
if config.api_key then
  vim.notify(
    "⚠️  API key in config file. Consider using environment variable instead.",
    vim.log.levels.WARN
  )
end
```

### **9.2 Prompt Privacy**

**User prompts contain potentially sensitive info**:
- Project names ("theme for my-secret-project")
- Personal preferences
- Timestamps

**Privacy measures**:
1. **Never send telemetry** - All data stays local
2. **Clear option in history**:
```vim
:HexwitchClearHistory  " Removes all saved prompts
```
3. **Anonymize in error reports**:
```lua
-- When user clicks "Report Issue"
function anonymize_error_report(error_data)
  return {
    error_type = error_data.type,
    error_message = error_data.message,
    plugin_version = VERSION,
    nvim_version = vim.version(),
    provider = error_data.provider,
    -- Do NOT include: prompts, API keys, usernames, paths
  }
end
```

### **9.3 AI Provider Communication**

**Sent to AI**:
- User's theme description
- Previous theme colors (for refinement)
- No personal identifiers

**Not sent**:
- Neovim version
- Operating system
- File paths
- Username
- Other plugins installed

---

## **10. Documentation Requirements**

### **10.1 README.md Structure**

```markdown
# hexwitch.nvim 🔮

Cast the perfect colorscheme spell with AI

[Demo GIF]

## ✨ Features

- Generate themes from natural language
- Refine with quick adjustments or descriptions
- Save and manage your favorite themes
- Auto-prompt for fresh daily/weekly themes
- Multiple AI providers (Claude, OpenAI, Ollama)

## 📦 Installation

### lazy.nvim
[code block]

### packer.nvim
[code block]

## ⚙️ Configuration

[Full config example]

## 🚀 Quick Start

[Step-by-step first use]

## 📚 Commands

[Table of all commands]

## 🎨 Examples

[Screenshots of themes with prompts]

## 🤝 Contributing

## 📄 License

## 🙏 Acknowledgments
```

### **10.2 Help Documentation (:help hexwitch)**

```vim
*hexwitch.txt*  AI-powered theme generator for Neovim

CONTENTS                                                *hexwitch-contents*

1. Introduction .................... |hexwitch-introduction|
2. Requirements .................... |hexwitch-requirements|
3. Installation .................... |hexwitch-installation|
4. Configuration ................... |hexwitch-configuration|
5. Commands ........................ |hexwitch-commands|
6. Functions ....................... |hexwitch-functions|
7. Mappings ........................ |hexwitch-mappings|
8. Troubleshooting ................. |hexwitch-troubleshooting|
9. FAQ ............................. |hexwitch-faq|
10. Changelog ...................... |hexwitch-changelog|

==============================================================================
1. INTRODUCTION                                     *hexwitch-introduction*

hexwitch.nvim is an AI-powered colorscheme generator that allows you to
create and customize Neovim themes using natural language descriptions.

Instead of browsing hundreds of themes or manually adjusting colors, simply
describe the aesthetic you want and hexwitch will generate it.

Examples: >
    :Hexwitch cyberpunk neon with deep purples
    :Hexwitch cozy coffee shop vibes
    :Hexwitch minimal monochrome for focused coding
<

==============================================================================
2. REQUIREMENTS                                     *hexwitch-requirements*

- Neovim >= 0.9.0
- plenary.nvim
- curl (for HTTP requests)
- API key for AI provider (or local Ollama installation)

==============================================================================
[... continue with detailed docs ...]
```

### **10.3 Example Gallery**

**Create `GALLERY.md` with**:
- Screenshots of generated themes
- Prompts used to create them
- Export JSON for each theme
- Community submissions

---

## **11. Testing Strategy**

### **11.1 Unit Tests**

**Test Coverage Areas**:

```lua
-- lua/hexwitch/tests/

-- Color utilities
describe("color_utils", function()
  it("converts hex to HSL correctly", function()
    assert.equals({ 240, 100, 50 }, hex_to_hsl("#0000ff"))
  end)
  
  it("calculates contrast ratio", function()
    assert.equals(21, contrast_ratio("#ffffff", "#000000"))
  end)
  
  it("validates WCAG AA compliance", function()
    assert.is_true(validate_contrast("#ffffff", "#000000"))
    assert.is_false(validate_contrast("#ffffff", "#eeeeee"))
  end)
end)

-- Theme parsing
describe("theme_parser", function()
  it("parses valid AI response", function()
    local json = '{"name":"test","background":"#000",...}'
    local theme = parse_ai_response(json)
    assert.equals("test", theme.name)
  end)
  
  it("handles missing accent colors", function()
    local json = '{"name":"test","background":"#000","foreground":"#fff"}'
    local theme = parse_ai_response(json)
    assert.equals(8, #theme.colors.accents)
  end)
  
  it("rejects invalid hex colors", function()
    local json = '{"background":"not-a-color"}'
    assert.has_error(function() parse_ai_response(json) end)
  end)
end)

-- Prompt frequency
describe("autoprompt", function()
  it("detects daily interval correctly", function()
    local last = "2025-10-20"
    local now = "2025-10-21"
    assert.is_true(should_prompt("daily", last, now))
  end)
  
  it("respects weekly interval", function()
    local last = "2025-10-20"
    local now = "2025-10-22"
    assert.is_false(should_prompt("weekly", last, now))
  end)
  
  it("never prompts on manual mode", function()
    assert.is_false(should_prompt("manual", "2020-01-01", "2025-10-21"))
  end)
end)

-- Storage
describe("storage", function()
  it("saves theme correctly", function()
    local theme = create_test_theme()
    save_theme(theme, "test_theme")
    local loaded = load_theme("test_theme")
    assert.same(theme, loaded)
  end)
  
  it("handles corrupted files gracefully", function()
    write_file("state.json", "invalid json{{{")
    assert.has_no_error(function() load_state() end)
  end)
end)
```

### **11.2 Integration Tests**

```lua
-- Test full workflows
describe("full_generation_flow", function()
  it("generates theme from prompt", function()
    -- Mock AI API
    mock_api_response({
      name = "test_theme",
      background = "#1a1a1a",
      foreground = "#ffffff",
      -- ...
    })
    
    -- Run command
    vim.cmd("Hexwitch test prompt")
    
    -- Verify theme applied
    local normal = vim.api.nvim_get_hl_by_name("Normal", true)
    assert.equals(0x1a1a1a, normal.background)
  end)
end)

describe("refinement_flow", function()
  it("refines existing theme", function()
    -- Apply base theme
    apply_theme(base_theme)
    
    -- Refine
    vim.cmd("HexwitchRefine darker background")
    
    -- Verify change
    local new_bg = get_current_background()
    assert.is_true(is_darker(new_bg, base_theme.background))
  end)
end)
```

### **11.3 Manual Testing Checklist**

**Before Each Release**:

- [ ] Test on Neovim 0.9, 0.10, nightly
- [ ] Test on Linux, macOS, Windows
- [ ] Test with each AI provider (Anthropic, OpenAI, Ollama)
- [ ] Test error scenarios (no internet, invalid key, etc.)
- [ ] Test with empty config (defaults work)
- [ ] Test auto-prompt in all modes (daily, weekly, manual)
- [ ] Test theme save/load/delete
- [ ] Test undo/redo
- [ ] Test refinement (quick adjustments + custom)
- [ ] Test with multiple Neovim instances
- [ ] Test help documentation (:help hexwitch)
- [ ] Verify no API keys in logs
- [ ] Test plugin conflicts (with other colorscheme plugins)

---

## **12. Release & Versioning**

### **12.1 Semantic Versioning**

**Format**: `MAJOR.MINOR.PATCH`

**Examples**:
- `1.0.0` - Initial release
- `1.1.0` - Add new feature (theme preview mode)
- `1.0.1` - Bug fix (fix contrast calculation)
- `2.0.0` - Breaking change (change config structure)

### **12.2 Release Checklist**

**Pre-release**:
- [ ] All tests passing
- [ ] Update CHANGELOG.md
- [ ] Update version in `lua/hexwitch/init.lua`
- [ ] Update documentation
- [ ] Test on supported Neovim versions
- [ ] Create git tag

**Release**:
- [ ] Push to GitHub with tag
- [ ] Create GitHub release with notes
- [ ] Announce in Neovim Discord/Reddit
- [ ] Submit to awesome-neovim list

**Post-release**:
- [ ] Monitor issues
- [ ] Respond to feedback
- [ ] Plan next version

### **12.3 Changelog Format**

```markdown
# Changelog

All notable changes to hexwitch.nvim will be documented in this file.

## [1.0.0] - 2025-10-21

### Added
- Initial release
- AI theme generation from natural language
- Support for Anthropic, OpenAI, Ollama
- Theme refinement with quick adjustments
- Theme save/load system
- Auto-prompt (daily/weekly modes)
- Undo/redo stack
- Comprehensive error handling

### Changed
- N/A (initial release)

### Fixed
- N/A (initial release)

## [1.1.0] - 2025-11-15 (Planned)

### Added
- Theme preview mode
- Treesitter highlight groups
- Theme sharing/export
- Accessibility contrast checker

### Changed
- Improved AI prompts for better color generation
- Faster theme application

### Fixed
- Issue #12: Fix crash on empty prompt
- Issue #15: Handle timezone changes correctly
```

---

## **13. Future Enhancements (v2.0+)**

### **13.1 Advanced Features**

**Theme Preview Mode**:
```
┌─────────────────────────────────────────────────────────────┐
│  🔮 Theme Preview                         [Apply] [Discard] │
├───────────────────────┬─────────────────────────────────────┤
│                       │                                     │
│  Current Theme        │  New Theme: "Ocean Sunset"          │
│                       │                                     │
│  [Code preview        │  [Code preview with                 │
│   with current        │   new theme colors]                 │
│   colors]             │                                     │
│                       │                                     │
│                       │                                     │
└───────────────────────┴─────────────────────────────────────┘
```

**Smart Timing**:
- Detect when user is idle (no typing for 30s)
- Detect after first `:w` save
- Avoid showing during active editing

**Contextual Intelligence**:
- Detect current file type, suggest appropriate theme
- Morning: "Energizing theme for your Python project?"
- Late night: "Easy-on-eyes theme for JavaScript?"

**Theme Collections**:
- Save groups of themes
- "Work themes", "Fun themes", "Presentation themes"
- Quick switch between collections

**Collaborative Features**:
- Share themes with URLs (optional cloud sync)
- Import community themes
- Vote on favorite themes

### **13.2 AI Enhancements**

**Multi-step Generation**:
```
Step 1: Generate base palette
Step 2: User adjusts palette
Step 3: AI applies to highlight groups
Step 4: User fine-tunes specific groups
```

**Learning from Preferences**:
- Track which generated themes user saves
- Analyze patterns (prefer warm colors, high contrast, etc.)
- Improve future generations based on preferences

**Mood Detection**:
- Analyze user's coding patterns
- Late night? Suggest dimmer themes
- High commit rate? Suggest energizing themes

### **13.3 Integration Features**

**Sync Across Machines**:
- Optional encrypted cloud sync
- Store themes in Git repository
- Sync with dotfiles manager

**IDE Integration**:
- Generate matching themes for:
  - Terminal emulators
  - tmux status bar
  - Browser DevTools
  - Markdown previewer

**Time-based Auto-switching**:
```lua
{
  auto_switch = {
    enabled = true,
    schedule = {
      ["06:00-12:00"] = "morning_bright",
      ["12:00-18:00"] = "afternoon_neutral",
      ["18:00-23:00"] = "evening_warm",
      ["23:00-06:00"] = "night_dark"
    }
  }
}
```

---

## **14. Support & Community**

### **14.1 Support Channels**

**GitHub Issues**: For bugs, feature requests
**GitHub Discussions**: For questions, showcasing themes
**Discord**: Real-time help (if community grows)

### **14.2 Contributing Guidelines**

**Welcome Contributions**:
- Bug fixes
- New AI provider integrations
- Documentation improvements
- Theme examples
- Translations

**Contribution Process**:
1. Fork repository
2. Create feature branch
3. Make changes with tests
4. Submit PR with description
5. Respond to review feedback

### **14.3 Code of Conduct**

- Be respectful and inclusive
- Focus on constructive feedback
- Welcome newcomers
- Credit contributors

---

## **15. Marketing & Launch Strategy**

### **15.1 Pre-launch**

- [ ] Build MVP with core features
- [ ] Create demo video/GIF
- [ ] Write comprehensive README
- [ ] Test with 5-10 beta users
- [ ] Gather feedback, iterate

### **15.2 Launch**

**Where to Announce**:
- [ ] Reddit: /r/neovim, /r/vim
- [ ] Neovim Discourse
- [ ] Hacker News (Show HN)
- [ ] Twitter/X with #neovim hashtag
- [ ] awesome-neovim list
- [ ] This Week in Neovim newsletter

**Launch Post Template**:
```markdown
🔮 Introducing hexwitch.nvim - Cast the Perfect Colorscheme

I built hexwitch because I was tired of:
- Spending hours browsing colorscheme galleries
- Almost finding the perfect theme but...
- Manually tweaking colors in config files

With hexwitch, just describe what you want:
"cyberpunk neon with deep purples" 
"cozy coffee shop vibes"
"minimal monochrome for focused coding"

And it generates a complete Neovim theme using AI.

[Demo GIF]

Features:
✨ Generate from natural language
🎨 Refine with quick adjustments
💾 Save your favorites
🔄 Daily/weekly fresh themes
🤖 Multiple AI providers

GitHub: [link]
Would love your feedback!
```

### **15.3 Post-launch**

- Respond to all feedback within 24h
- Fix critical bugs immediately
- Gather feature requests
- Build community showcase
- Create tutorial videos
- Write blog posts about interesting use cases

---

## **16. Appendix**

### **16.1 Color Theory Reference**

**Color Spaces**:
- **RGB**: Red, Green, Blue (0-255 each)
- **Hex**: #RRGGBB (00-FF each)
- **HSL**: Hue (0-360°), Saturation (0-100%), Lightness (0-100%)

**HSL Advantages for Theme Generation**:
- Hue: Change color family while preserving saturation/lightness
- Saturation: Make colors more vibrant/muted
- Lightness: Make colors lighter/darker

**Common Transformations**:
```lua
-- Increase warmth: Shift hue toward orange/red (-20° to 0°)
-- Decrease warmth: Shift hue toward blue (+180° to +200°)
-- Increase contrast: Darken background, lighten foreground
-- Decrease contrast: Move both toward middle gray
```

### **16.2 AI Prompt Engineering Tips**

**Good AI Prompts Include**:
1. **Specific mood/feeling**: "energizing", "calm", "focused"
2. **Color references**: "warm browns", "deep blues", "neon accents"
3. **Real-world analogies**: "sunset", "forest", "coffee shop"
4. **Use case**: "for long coding sessions", "for presentations"

**Poor Prompts**:
- Too vague: "nice theme"
- Just single color: "blue"
- Contradictory: "bright dark theme"

**Example Good Prompts**:
- "Solarized-inspired but with purple accents for Python"
- "High contrast dark theme for readability, blue and orange"
- "Retro 80s terminal aesthetic with neon green and magenta"
- "Minimal light theme with subtle pastels for writing docs"

### **16.3 Accessibility Guidelines**

**WCAG Contrast Standards**:
- **AA**: 4.5:1 for normal text (minimum for hexwitch)
- **AAA**: 7:1 for normal text (ideal)
- **Large text**: 3:1 minimum

**Color Blindness Considerations**:
- **Protanopia**: Red-blind (7% of males)
- **Deuteranopia**: Green-blind (1% of males)
- **Tritanopia**: Blue-blind (rare)

**Hexwitch Approach**:
- Always validate contrast ratios
- Offer "accessibility mode" that generates themes tested for color blindness
- Use tools like Coblis to simulate color blindness

### **16.4 Glossary**

- **Accent color**: Secondary color used for syntax highlighting
- **Colorscheme**: Complete set of colors for editor UI and syntax
- **Contrast ratio**: Mathematical measure of color difference (1:1 to 21:1)
- **Highlight group**: Neovim's syntax element (Comment, Function, etc.)
- **HSL**: Hue, Saturation, Lightness color model
- **Prompt**: User's natural language description of desired theme
- **Provider**: AI service (Anthropic, OpenAI, Ollama)
- **Refinement**: Iterative improvement of generated theme
- **Treesitter**: Modern Neoim syntax parser for better highlighting

---

## **17. Complete Configuration Example**

### **17.1 Minimal Configuration**

```lua
-- For users who just want it to work
require('hexwitch').setup({
  ai_provider = "anthropic",
  api_key = os.getenv("ANTHROPIC_API_KEY"),
})
```

### **17.2 Recommended Configuration**

```lua
-- Balanced setup with common preferences
require('hexwitch').setup({
  -- AI Settings
  ai_provider = "anthropic",
  api_key = os.getenv("ANTHROPIC_API_KEY"),
  model = "claude-sonnet-4-5-20250929",
  timeout = 30,
  fallback_provider = "ollama",  -- Fallback to local if API fails
  
  -- Auto-prompt Settings
  prompt_frequency = "weekly",   -- Once a week is nice
  prompt_on_startup = true,      -- Show on first launch
  prompt_timing = "delayed",     -- Wait until settled in
  prompt_delay_seconds = 120,    -- 2 minutes after startup
  
  -- Theme Settings
  contrast_threshold = 4.5,      -- WCAG AA minimum
  accent_count = 8,
  
  -- UI Settings
  ui = {
    border = "rounded",
    width_ratio = 0.6,
    height_ratio = 0.4,
    icons = true,
  },
})
```

### **17.3 Power User Configuration**

```lua
-- Advanced setup with all features
require('hexwitch').setup({
  -- AI Settings
  ai_provider = "anthropic",
  api_key = os.getenv("ANTHROPIC_API_KEY"),
  model = "claude-sonnet-4-5-20250929",
  timeout = 30,
  fallback_provider = "ollama",
  
  -- Auto-prompt Settings
  prompt_frequency = "daily",
  prompt_on_startup = true,
  prompt_timing = "delayed",
  prompt_delay_seconds = 180,
  contextual_prompts = true,     -- Time-of-day aware (v2.0)
  show_welcome = false,          -- Already configured
  
  -- Theme Settings
  contrast_threshold = 7.0,      -- WCAG AAA for maximum readability
  accent_count = 8,
  apply_treesitter = true,       -- Full treesitter support
  apply_lsp_semantic = true,     -- LSP semantic tokens
  
  -- Storage Settings
  storage_path = vim.fn.stdpath('data') .. '/hexwitch',
  max_history = 100,             -- Keep more history
  auto_save_history = true,
  
  -- UI Settings
  ui = {
    border = "double",
    width_ratio = 0.7,
    height_ratio = 0.5,
    icons = true,
    animations = true,           -- Animated progress (v2.0)
    preview_mode = true,          -- Preview before applying (v2.0)
  },
  
  -- Custom Keymaps
  keymaps = {
    close = { "<Esc>", "q" },
    confirm = { "<CR>", "<C-y>" },
    cancel = { "<C-c>", "<C-q>" },
    next = "<Tab>",
    prev = "<S-Tab>",
  },
  
  -- Callbacks for custom integrations
  on_theme_applied = function(theme)
    -- Notify tmux to match colors
    vim.fn.system('tmux source-file ~/.tmux.conf')
    
    -- Update terminal emulator (if supported)
    update_terminal_colors(theme.colors)
    
    -- Log for analytics
    print("Applied theme: " .. theme.name)
  end,
  
  on_theme_saved = function(theme, name)
    -- Auto-export to dotfiles repo
    local export_path = vim.fn.expand('~/dotfiles/nvim/themes/')
    vim.fn.system(string.format(
      'cp %s/themes/%s.json %s',
      vim.fn.stdpath('data') .. '/hexwitch',
      name,
      export_path
    ))
  end,
  
  on_error = function(error_type, message)
    -- Custom error logging
    local log = io.open(vim.fn.expand('~/hexwitch_errors.log'), 'a')
    log:write(string.format('[%s] %s: %s\n', 
      os.date('%Y-%m-%d %H:%M:%S'),
      error_type,
      message
    ))
    log:close()
  end,
})

-- Custom commands for workflow
vim.api.nvim_create_user_command('HexwitchMorning', function()
  vim.cmd('Hexwitch energizing theme with citrus colors for productive morning')
end, {})

vim.api.nvim_create_user_command('HexwitchEvening', function()
  vim.cmd('Hexwitch calm warm theme for relaxed evening coding')
end, {})

vim.api.nvim_create_user_command('HexwitchPresentation', function()
  vim.cmd('Hexwitch high contrast light theme for presentations')
end, {})

-- Keybindings
vim.keymap.set('n', '<leader>ht', ':Hexwitch<CR>', { desc = 'Generate theme' })
vim.keymap.set('n', '<leader>hr', ':HexwitchRefine<CR>', { desc = 'Refine theme' })
vim.keymap.set('n', '<leader>hs', ':HexwitchSave ', { desc = 'Save theme' })
vim.keymap.set('n', '<leader>hl', ':HexwitchList<CR>', { desc = 'List themes' })
vim.keymap.set('n', '<leader>hu', ':HexwitchUndo<CR>', { desc = 'Undo theme' })
```

### **17.4 Offline/Local-Only Configuration**

```lua
-- For users who prefer no external API calls
require('hexwitch').setup({
  ai_provider = "ollama",
  custom_endpoint = {
    url = "http://localhost:11434/api/generate",
    model = "llama3.2",  -- or any model you have installed
  },
  
  prompt_frequency = "manual",  -- Only when explicitly called
  
  -- Everything else stays local
  storage_path = vim.fn.stdpath('data') .. '/hexwitch',
})
```

---

## **18. Troubleshooting Guide**

### **18.1 Common Issues**

#### **Issue: "Module 'plenary' not found"**

**Cause**: Missing dependency

**Solution**:
```lua
-- Add to your plugin manager config
-- lazy.nvim
{
  'yourusername/hexwitch.nvim',
  dependencies = { 'nvim-lua/plenary.nvim' },
}

-- packer.nvim
use {
  'yourusername/hexwitch.nvim',
  requires = { 'nvim-lua/plenary.nvim' },
}
```

---

#### **Issue: "API key invalid or missing"**

**Cause**: API key not configured

**Solution**:
```bash
# Set environment variable (in ~/.bashrc or ~/.zshrc)
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Or in config
require('hexwitch').setup({
  api_key = "sk-ant-api03-...",  -- Not recommended
})

# Or use a secrets manager
require('hexwitch').setup({
  api_key = require('my_secrets').anthropic_key,
})
```

---

#### **Issue: "Theme generation takes too long"**

**Cause**: Slow network or API response

**Solution**:
```lua
-- Increase timeout
require('hexwitch').setup({
  timeout = 60,  -- Wait longer
  
  -- Or use local provider
  ai_provider = "ollama",
  fallback_provider = "ollama",
})
```

---

#### **Issue: "Colors look wrong/washed out"**

**Cause**: Terminal not supporting true color

**Solution**:
```lua
-- Add to your Neovim config
vim.opt.termguicolors = true

-- Check terminal support
-- Run in terminal: echo $TERM
-- Should be: xterm-256color, screen-256color, or tmux-256color

-- In ~/.tmux.conf (if using tmux)
set -g default-terminal "screen-256color"
set -ga terminal-overrides ",xterm-256color:Tc"
```

---

#### **Issue: "Theme keeps getting overridden"**

**Cause**: Conflicting colorscheme plugin

**Solution**:
```lua
-- Remove or disable other colorscheme auto-loaders
-- Comment out lines like:
-- vim.cmd('colorscheme gruvbox')
-- require('tokyonight').load()

-- Or load hexwitch theme after everything else
vim.api.nvim_create_autocmd("VimEnter", {
  callback = function()
    -- Ensure hexwitch theme stays applied
    vim.defer_fn(function()
      require('hexwitch').reapply_current_theme()
    end, 100)
  end
})
```

---

#### **Issue: "Floating window doesn't appear"**

**Cause**: UI rendering issue or keybinding conflict

**Solution**:
```lua
-- Check if command works directly
:lua require('hexwitch.ui.input').show()

-- Check for errors
:messages

-- Try different border style
require('hexwitch').setup({
  ui = {
    border = "single",  -- Try different borders
  }
})

-- Check for conflicting plugins
-- Temporarily disable other UI plugins to test
```

---

#### **Issue: "Saved themes not loading"**

**Cause**: File permission or path issue

**Solution**:
```bash
# Check directory exists and is writable
ls -la ~/.local/share/nvim/hexwitch/
chmod 755 ~/.local/share/nvim/hexwitch/

# Check file contents
cat ~/.local/share/nvim/hexwitch/themes/my_theme.json

# Verify JSON is valid
python3 -m json.tool ~/.local/share/nvim/hexwitch/themes/my_theme.json
```

---

#### **Issue: "Auto-prompt shows every time"**

**Cause**: State file not being saved

**Solution**:
```lua
-- Check state file
:lua print(vim.inspect(require('hexwitch.storage.state').load()))

-- Manually reset
:HexwitchSetFrequency manual

-- Check permissions
chmod 644 ~/.local/share/nvim/hexwitch/state.json
```

---

### **18.2 Debug Mode**

**Enable verbose logging**:

```lua
require('hexwitch').setup({
  debug = true,  -- Enable debug logging
})

-- View logs
:messages

-- Or write to file
require('hexwitch').setup({
  debug = true,
  log_file = vim.fn.expand('~/hexwitch_debug.log'),
})
```

**What gets logged in debug mode**:
- API requests and responses (with API key redacted)
- Theme generation steps
- File I/O operations
- Error stack traces
- Performance metrics

---

### **18.3 Getting Help**

**Before opening an issue**:
1. Check `:HexwitchDebug` output
2. Review `:messages` for errors
3. Try with minimal config
4. Search existing issues

**When opening an issue, include**:
```markdown
**Bug Description**
Clear description of what's wrong

**To Reproduce**
Steps to reproduce the behavior

**Expected Behavior**
What you expected to happen

**Environment**
- OS: [e.g., macOS 14.0]
- Neovim version: [e.g., 0.10.0]
- hexwitch version: [e.g., 1.0.0]
- AI provider: [e.g., Anthropic]

**Configuration**
```lua
-- Your hexwitch config
```

**Debug Output**
```
-- Output from :HexwitchDebug
```

**Screenshots/Logs**
If applicable
```

---

## **19. Performance Considerations**

### **19.1 Optimization Targets**

**Theme Application Speed**:
- **Target**: < 50ms to apply theme
- **Bottleneck**: Setting 50+ highlight groups
- **Optimization**: Batch `nvim_set_hl()` calls

```lua
-- Slow: Individual calls
for name, hl in pairs(highlight_groups) do
  vim.api.nvim_set_hl(0, name, hl)
end

-- Fast: Batch with defer_fn
vim.schedule(function()
  for name, hl in pairs(highlight_groups) do
    vim.api.nvim_set_hl(0, name, hl)
  end
end)
```

**API Request Time**:
- **Target**: < 5s for theme generation
- **Typical**: 2-4s with Claude
- **Fallback**: Show "Taking longer than usual..." after 5s

**File I/O**:
- **Target**: < 10ms for save/load operations
- **Optimization**: Async file operations with plenary

```lua
-- Use plenary's async I/O
local async = require('plenary.async')
local file = require('plenary.file')

async.run(function()
  file.write_async(filepath, data, function()
    print("Saved!")
  end)
end)
```

**Memory Usage**:
- **Target**: < 5MB for plugin
- **Monitor**: Theme history size
- **Limit**: Keep only last 50 themes

### **19.2 Caching Strategy**

**Cache API Responses** (optional, v2.0):
```lua
-- Cache themes for repeated prompts
local cache = {}

function generate_theme_cached(prompt)
  local cache_key = vim.fn.sha256(prompt)
  
  if cache[cache_key] then
    return cache[cache_key]
  end
  
  local theme = call_ai_api(prompt)
  cache[cache_key] = theme
  return theme
end
```

**Considerations**:
- Cache size limit (max 20 entries)
- Cache invalidation (clear after 7 days)
- User control (`:HexwitchClearCache`)

---

## **20. Final Checklist**

### **20.1 Pre-Release Checklist**

**Code Quality**:
- [ ] All functions documented
- [ ] No hardcoded paths
- [ ] No exposed API keys
- [ ] Error handling on all external calls
- [ ] Async operations don't block UI

**Documentation**:
- [ ] README complete with examples
- [ ] Help documentation (`:help hexwitch`)
- [ ] All commands documented
- [ ] Configuration options explained
- [ ] Troubleshooting guide included

**Testing**:
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Manual testing on 3+ systems
- [ ] Tested with all supported providers
- [ ] Tested with minimal config

**User Experience**:
- [ ] First-time experience smooth
- [ ] Errors are user-friendly
- [ ] Loading states clear
- [ ] Success feedback satisfying
- [ ] No surprising behavior

**Performance**:
- [ ] Theme application < 50ms
- [ ] No blocking operations
- [ ] Startup time < 10ms
- [ ] Memory usage reasonable

**Accessibility**:
- [ ] All themes meet WCAG AA contrast
- [ ] UI navigable with keyboard only
- [ ] Screen reader friendly (text-only fallbacks)

**Security**:
- [ ] API keys never logged
- [ ] No telemetry without consent
- [ ] Safe handling of user input
- [ ] No arbitrary code execution

---

## **21. Success Criteria**

### **21.1 Definition of Success**

**Metrics** (after 3 months):
- ⭐ **GitHub Stars**: 500+ (indicates interest)
- 📥 **Installations**: 2,000+ unique users
- 🐛 **Issue Resolution**: < 7 day average response time
- ⚡ **Performance**: 95% of generations < 5 seconds
- 😊 **User Satisfaction**: Positive feedback ratio > 80%

**Qualitative Goals**:
- Featured in "awesome-neovim" list
- Mentioned in "This Week in Neovim"
- Community contributes themes to gallery
- At least 5 community PRs merged
- Used in popular Neovim configs

### **21.2 User Stories Validation**

**Can users successfully**:
- ✅ Install and configure in < 5 minutes
- ✅ Generate first theme with no issues
- ✅ Understand what makes good prompts
- ✅ Refine themes to their liking
- ✅ Save and reuse favorite themes
- ✅ Troubleshoot common issues themselves
- ✅ Customize to their workflow

---

## **22. Conclusion**

### **22.1 Product Summary**

**hexwitch.nvim** is a comprehensive AI-powered theme generator that transforms how Neovim users interact with colorschemes. By combining:

- 🎨 **Natural language input** - Describe what you want
- 🤖 **AI generation** - Get professional results
- ⚡ **Instant application** - See changes immediately
- 🔄 **Easy refinement** - Iterate until perfect
- 💾 **Persistent storage** - Keep your favorites
- 🔮 **Flexible automation** - Fresh themes when you want them

The product solves the real problem of colorscheme fatigue and the "almost perfect" theme dilemma.

### **22.2 Core Principles**

1. **User control first** - Never interrupt, always respectful
2. **Quality over quantity** - One great theme > many mediocre ones
3. **Simplicity with depth** - Easy to start, powerful when needed
4. **Privacy by default** - Local-first, no tracking
5. **Accessibility always** - Readable themes for everyone

### **22.3 Development Philosophy**

- **Ship fast, iterate faster** - v1.0 focuses on core features
- **Listen to users** - Community feedback shapes roadmap
- **Maintain quality** - No features at expense of reliability
- **Stay focused** - Do one thing exceptionally well

### **22.4 Next Steps**

**Immediate** (next 2 weeks):
1. Build MVP with core features
2. Alpha test with 5 users
3. Fix critical issues
4. Prepare launch materials

**Short-term** (1-3 months):
1. Public launch
2. Gather feedback
3. Rapid iteration on issues
4. Build community

**Long-term** (6-12 months):
1. v2.0 with advanced features
2. Ecosystem integrations
3. Theme marketplace (maybe)
4. Continue improving AI prompts

---

## **23. Appendix: File Structure Reference**

### **23.1 Complete Directory Tree**

```
hexwitch.nvim/
├── lua/
│   └── hexwitch/
│       ├── init.lua                 # Plugin entry, setup()
│       ├── config.lua              # Config validation & defaults
│       │
│       ├── ui/
│       │   ├── init.lua            # UI module loader
│       │   ├── input.lua           # Input prompt window
│       │   ├── feedback.lua        # Success/error messages
│       │   ├── refinement.lua      # Refinement interface
│       │   ├── list.lua            # Theme browser
│       │   ├── welcome.lua         # First-time wizard
│       │   └── components/
│       │       ├── window.lua      # Base floating window
│       │       ├── button.lua      # Button component
│       │       ├── input_field.lua # Text input component
│       │       └── progress.lua    # Progress bar component
│       │
│       ├── ai/
│       │   ├── init.lua            # AI module loader
│       │   ├── client.lua          # HTTP client wrapper
│       │   ├── prompt.lua          # Prompt engineering
│       │   └── providers/
│       │       ├── anthropic.lua   # Claude integration
│       │       ├── openai.lua      # OpenAI integration
│       │       ├── ollama.lua      # Ollama integration
│       │       └── custom.lua      # Custom endpoint support
│       │
│       ├── theme/
│       │   ├── init.lua            # Theme module loader
│       │   ├── generator.lua       # Core generation logic
│       │   ├── applicator.lua      # Apply to Neovim
│       │   ├── parser.lua          # Parse AI responses
│       │   ├── validator.lua       # Validate themes
│       │   ├── transformer.lua     # Quick adjustments (HSL)
│       │   └── highlight_groups.lua # Highlight group definitions
│       │
│       ├── storage/
│       │   ├── init.lua            # Storage module loader
│       │   ├── themes.lua          # Theme save/load/delete
│       │   ├── history.lua         # Generation history
│       │   ├── state.lua           # Plugin state management
│       │   └── lock.lua            # File locking for concurrency
│       │
│       ├── autoprompt.lua          # Auto-prompt logic
│       ├── commands.lua            # Vim command registration
│       ├── keymaps.lua             # Default keymaps
│       └── utils/
│           ├── colors.lua          # Color conversion & manipulation
│           ├── contrast.lua        # WCAG contrast calculations
│           ├── logger.lua          # Logging utilities
│           └── async.lua           # Async helpers
│
├── plugin/
│   └── hexwitch.vim                # Vim commands definition
│
├── doc/
│   └── hexwitch.txt                # Vim help documentation
│
├── tests/
│   ├── unit/
│   │   ├── colors_spec.lua
│   │   ├── parser_spec.lua
│   │   ├── validator_spec.lua
│   │   └── autoprompt_spec.lua
│   └── integration/
│       ├── generation_spec.lua
│       ├── refinement_spec.lua
│       └── storage_spec.lua
│
├── examples/
│   ├── themes/                     # Example generated themes
│   │   ├── cyberpunk_neon.json
│   │   ├── forest_dawn.json
│   │   └── ocean_sunset.json
│   └── configs/                    # Example configurations
│       ├── minimal.lua
│       ├── recommended.lua
│       └── poweruser.lua
│
├── .github/
│   ├── workflows/
│   │   ├── test.yml               # CI/CD testing
│   │   └── release.yml            # Automated releases
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
│
├── README.md
├── CHANGELOG.md
├── LICENSE
├── CONTRIBUTING.md
└── GALLERY.md                      # Theme showcase
```

---

This completes the comprehensive PRD for **hexwitch.nvim**! 🔮

**Summary of what we've covered**:
1. ✅ Product vision & user flows
2. ✅ Complete feature specifications
3. ✅ All edge cases & error handling
4. ✅ Configuration examples
5. ✅ Technical architecture
6. ✅ Testing strategy
7. ✅ Documentation requirements
8. ✅ Performance considerations
9. ✅ Security & privacy measures
10. ✅ Launch & marketing strategy
11. ✅ Future enhancements roadmap

**The PRD is production-ready!** You can now:
- Hand this to developers for implementation
- Use it as a reference during development
- Share with potential contributors
- Reference for user documentation

Want me to dive deeper into any specific section, or shall we move on to discussing implementation details?