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
      ai_provider = "openai",
      api_key = os.getenv("OPENAI_API_KEY"),
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
│  [Try Again]  [Use Fallback (OpenRouter)]  [Check Settings] │
│                                                             │
│  [Close]                                                     │
└─────────────────────────────────────────────────────────────┘
```

**Button Actions**:
- **Try Again**: Retries same request
- **Use Fallback**: Switches to OpenRouter if configured
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
    "ai_provider": "openai",
    "model": "gpt-4",
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
│       │   │   ├── openai.lua    # OpenAI integration
│       │   │   ├── openrouter.lua # OpenRouter integration
│       │   │   ├── anthropic.lua # Claude integration
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
    ai_provider = "openai",
    model = "gpt-4",
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
    "favorite_provider": "openai"
  }
}
```

### **4.3 AI Integration**

#### **Request Format (OpenAI GPT-4)**

```lua
-- HTTP POST to https://api.openai.com/v1/chat/completions
{
  model = "gpt-4",
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

#### **Request Format (OpenRouter)**

```lua
-- HTTP POST to https://openrouter.ai/api/v1/chat/completions
{
  model = "anthropic/claude-3.5-sonnet",
  max_tokens = 1024,
  messages = {
    {
      role = "user",
      content = [[Same prompt as OpenAI]]
    }
  },
  headers = {
    ["HTTP-Referer"] = "https://github.com/yourusername/hexwitch.nvim",
    ["X-Title"] = "hexwitch.nvim"
  }
}
```

#### **Response Parsing**

```lua
-- Expected response from OpenAI
{
  "choices": [
    {
      "message": {
        "content": '{"name":"ocean_sunset","background":"#1a2332",...}'
      }
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

### **4.4 Configuration API**

#### **Full Configuration Schema**

```lua
require('hexwitch').setup({
  -- Auto-prompt settings
  prompt_frequency = "manual",      -- "manual" | "daily" | "weekly" | "never"
  prompt_on_startup = false,        -- Show on first launch of period
  prompt_timing = "immediate",      -- "immediate" | "delayed"
  prompt_delay_seconds = 180,       -- Delay before showing (if "delayed")
  show_welcome = true,              -- First-time setup wizard
  
  -- AI provider settings
  ai_provider = "openai",           -- "openai" | "openrouter" | "anthropic" | "ollama" | "custom"
  api_key = nil,                    -- Or use env var HEXWITCH_API_KEY
  model = "gpt-4",
  timeout = 30,                     -- Request timeout in seconds
  fallback_provider = "openrouter", -- Fallback if primary fails
  
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

## **5. Commands Reference**

### **5.1 Core Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:Hexwitch` | Open interactive prompt | `:Hexwitch` |
| `:Hexwitch <description>` | Generate theme directly | `:Hexwitch cozy autumn forest` |
| `:HexwitchQuick` | Generate variation of last theme | `:HexwitchQuick` |
| `:HexwitchRandom` | Generate random theme | `:HexwitchRandom` |
| `:HexwitchRefine` | Open refinement UI | `:HexwitchRefine` |
| `:HexwitchRefine <changes>` | Refine with description | `:HexwitchRefine darker background` |

### **5.2 Theme Management Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchSave <name>` | Save current theme | `:HexwitchSave ocean_sunset` |
| `:HexwitchLoad <name>` | Load saved theme | `:HexwitchLoad ocean_sunset` |
| `:HexwitchList` | Browse saved themes | `:HexwitchList` |
| `:HexwitchDelete <name>` | Delete saved theme | `:HexwitchDelete ocean_sunset` |
| `:HexwitchExport <name>` | Export theme as JSON | `:HexwitchExport ocean_sunset` |
| `:HexwitchImport <path>` | Import theme from JSON | `:HexwitchImport ~/theme.json` |

### **5.3 History Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchHistory` | View generation history | `:HexwitchHistory` |
| `:HexwitchUndo` | Revert to previous theme | `:HexwitchUndo` |
| `:HexwitchRedo` | Reapply undone theme | `:HexwitchRedo` |
| `:HexwitchClearHistory` | Clear all history | `:HexwitchClearHistory` |

### **5.4 Configuration Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchConfig` | Open config UI | `:HexwitchConfig` |
| `:HexwitchSetFrequency <mode>` | Set prompt frequency | `:HexwitchSetFrequency daily` |
| `:HexwitchSetProvider <provider>` | Change AI provider | `:HexwitchSetProvider openrouter` |
| `:HexwitchStatus` | Show plugin status | `:HexwitchStatus` |
| `:HexwitchDocs` | Open documentation | `:HexwitchDocs` |

### **5.5 Utility Commands**

| Command | Description | Example |
|---------|-------------|---------|
| `:HexwitchInspect` | Show current theme details | `:HexwitchInspect` |
| `:HexwitchContrast` | Check contrast ratios | `:HexwitchContrast` |
| `:HexwitchDebug` | Show debug information | `:HexwitchDebug` |
| `:HexwitchVersion` | Show plugin version | `:HexwitchVersion` |

---

## **6. Success Criteria**

### **6.1 Definition of Success**

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

### **6.2 User Stories Validation**

**Can users successfully**:
- ✅ Install and configure in < 5 minutes
- ✅ Generate first theme with no issues
- ✅ Understand what makes good prompts
- ✅ Refine themes to their liking
- ✅ Save and reuse favorite themes
- ✅ Troubleshoot common issues themselves
- ✅ Customize to their workflow

---

## **7. Conclusion**

### **7.1 Product Summary**

**hexwitch.nvim** is a comprehensive AI-powered theme generator that transforms how Neovim users interact with colorschemes. By combining:

- 🎨 **Natural language input** - Describe what you want
- 🤖 **AI generation** - Get professional results
- ⚡ **Instant application** - See changes immediately
- 🔄 **Easy refinement** - Iterate until perfect
- 💾 **Persistent storage** - Keep your favorites
- 🔮 **Flexible automation** - Fresh themes when you want them

The product solves the real problem of colorscheme fatigue and the "almost perfect" theme dilemma.

### **7.2 Core Principles**

1. **User control first** - Never interrupt, always respectful
2. **Quality over quantity** - One great theme > many mediocre ones
3. **Simplicity with depth** - Easy to start, powerful when needed
4. **Privacy by default** - Local-first, no tracking
5. **Accessibility always** - Readable themes for everyone

### **7.3 Development Philosophy**

- **Ship fast, iterate faster** - v1.0 focuses on core features
- **Listen to users** - Community feedback shapes roadmap
- **Maintain quality** - No features at expense of reliability
- **Stay focused** - Do one thing exceptionally well

### **7.4 Next Steps**

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
