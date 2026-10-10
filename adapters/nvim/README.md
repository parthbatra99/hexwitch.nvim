# Hexwitch Neovim adapter

A thin client for the Rust engine. No Telescope or Plenary dependencies.
Requires **Neovim 0.10+** and a matching `hexwitch` binary. Live interpretation
also requires curl and `OPENAI_API_KEY` or `OPENROUTER_API_KEY` in Neovim's
launch environment. Offline replay and explicit edit specs need neither key.

## Local installation

From the repository:

```sh
cargo build --manifest-path core/Cargo.toml -p hexwitch-cli
```

Configure the adapter directory as a local plugin (not the monorepo root):

```lua
-- lazy.nvim
{
  dir = "/absolute/path/to/hexwitch.nvim/adapters/nvim",
  name = "hexwitch.nvim",
  config = function()
    require("hexwitch").setup({
      cmd = "/absolute/path/to/hexwitch.nvim/core/target/debug/hexwitch",
      auto_apply = false, -- review candidates first (default)
      report = "short",  -- "full" shows the full engine report
      -- provider = "openrouter",
      -- model = "openai/gpt-4o-mini",
    })
  end,
}
```

Or add `adapters/nvim` to runtimepath directly and run
`runtime plugin/hexwitch.lua`. Use `:checkhealth hexwitch` to check the binary.
Prebuilt release binaries are not set up yet; building/installing the engine
is currently a separate step. No automatic downloads or installs occur.

## Workflow

1. `:HexwitchReference /path/to/theme.json` loads a reference. The engine
   accepts IR JSON or its supported subset of VS Code JSON. The adapter copies
   the document into its session storage so later edits do not mutate it.
2. `:Hexwitch darker background, readable comments, keep the syntax colors`
   asks the engine to interpret the request and prepare a candidate. With no
   arguments, `:Hexwitch` opens `vim.ui.input`.
3. `:HexwitchStatus` shows the interpretation notes, exact EditSpec, revision
   report, version history and artifact path. `:HexwitchAnalyze` shows the
   candidate's contrast ladder and accent budget.
4. `:HexwitchApply` accepts the pending candidate; `:HexwitchDiscard` rejects
   it without changing the current theme. `:HexwitchApply 2` can revisit a
   saved version.
5. `:HexwitchUndo` / `:HexwitchRedo` reapply previous artifacts. Undoing the
   first reference restores the original editor highlights and terminal
   palette. A revision after undo starts a new branch; saved artifacts remain.

This is **review before apply**, not a split-screen visual preview. Applying
an artifact changes your open editor immediately. Live colorscheme loading
fires Neovim highlights directly; configured plugin-specific colors and
ColorScheme autocmd behavior are not yet modeled by the engine.

For testing without a provider:

```vim
:HexwitchReference /repo/core/eval/themes/gruvbox-dark-medium.json
:HexwitchReplay /repo/core/eval/llm/darker-readable-comments.json
:HexwitchStatus
:HexwitchApply
:HexwitchUndo
```

`:HexwitchSpec /path/to/spec.json` prepares a revision from an explicit
EditSpec, skipping interpretation but not revision or emission.

## Safety and current limits

- Engine processes use argument arrays, never shell command strings.
- The adapter does not read or store API keys; the child engine inherits the
  process environment. Provider credential handling is the CLI's responsibility.
- Only one mutation request runs at a time. Pending candidates must be accepted
  or discarded before another refinement.
- Failed subprocesses leave the current theme/history unchanged. If loading
  an emitted artifact errors, its partial highlight changes are rolled back.
- Refused or impossible edits always require manual acceptance, even with
  `auto_apply = true`; the machine-readable revision report controls this.
- Artifacts live in `stdpath('data')/hexwitch/sessions/<session>/`. Documents,
  specs and generated Lua remain on disk; history navigation is session-local
  and is not restored automatically after restarting Neovim.
- JSON-only references today; installed-theme capture, wallpaper inputs,
  complete scope fidelity, full scene validation and terminal degradation
  checks remain future engine work.

## Tests

`cargo test --manifest-path core/Cargo.toml --all --locked` runs the Lua suite
in a clean headless Neovim against the binary built by Cargo. It removes
provider keys from that subprocess environment and replays saved responses.
It requires no Lua test framework or external plugin.
