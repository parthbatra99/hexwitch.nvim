# Hexwitch Neovim adapter

A thin client for the Rust engine. No Telescope or Plenary dependencies.
Requires **Neovim 0.10+** and a matching `hexwitch` binary. Live interpretation
also requires curl and an API key in Neovim's launch environment. All live
calls use OpenAI-compatible Chat Completions; base URL and model are configurable. Offline replay and explicit edit specs need neither key.

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
      base_url = "https://api.openai.com/v1",
      api_key_env = "OPENAI_API_KEY", -- name only, never paste the key here
      model = "gpt-4o-mini",
      response_format = "json_schema", -- "json_object" or "none" if unsupported
    })
  end,
}
```

If `cmd` is omitted, the wrapper checks PATH then this checkout's release/debug
binary. It never downloads or builds automatically. An explicit missing cmd is
reported rather than silently replaced. `~` in cmd paths is expanded. For a release
binary run `cargo build --release --manifest-path core/Cargo.toml -p hexwitch-cli`.

Or add `adapters/nvim` to runtimepath directly and run
`runtime plugin/hexwitch.lua`. Use `:checkhealth hexwitch` to check the binary.
Prebuilt release binaries are not set up yet; building/installing the engine
is currently a separate step. No automatic downloads or installs occur.

## OpenAI-compatible live APIs

Use an API root, not a full completion endpoint; the engine appends
`/chat/completions`. For example:

```lua
require("hexwitch").setup({
  base_url = vim.env.OPENAI_BASE_URL or "https://api.openai.com/v1",
  api_key_env = "OPENAI_API_KEY",
  model = vim.env.HEXWITCH_MODEL or "gpt-4o-mini",
  response_format = "json_schema",
  auto_apply = false,
})
```

Set the named key environment variable before launching Neovim. For another API,
change base_url (e.g. `https://openrouter.ai/api/v1`) and use that provider's model
identifier and key. `OPENAI_BASE_URL` is supported when base_url is omitted.
Both HTTP and HTTPS API roots are accepted. HTTP is unencrypted: API keys and
prompts can be intercepted or modified in transit. Prefer HTTPS; use HTTP only
when you deliberately trust the connection (for example, an internal gateway).
The CLI and health check warn about HTTP. URL credentials, query strings and
fragments are still rejected.

Response formats are explicit: `json_schema` for strict structured output,
`json_object` for JSON-mode servers, `none` when the server only supports ordinary
Chat Completions. Local schema/semantic validation remains strict in every mode;
there are no automatic retry/fallback requests or credential-file discovery.
`:checkhealth hexwitch` shows resolved binary/API root and whether the key is set,
never its value. Legacy provider aliases remain available for older configs.

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

## Property-aware interpretation (opt-in)

`:HexwitchV1 no italics anywhere` uses the v1 interpretation/report protocol.
Unlike legacy replay it can name individual fg/bg/opacity/typography properties,
show clarification questions without creating a candidate, and block required
constraint rollback. It still requires an explicitly configured API key for
live calls; no network call happens for saved replay:

```lua
require("hexwitch").refine("Keep the background color, but remove its transparency.", {
  v1 = true,
  from_llm = "/repo/core/eval/llm/property-opaque-background.json",
})
```

`opts.context` can point to reviewed prior requests/actions. Clarification is
shown in notifications and `:HexwitchStatus`; supply a corrected request to
continue. The v1 path is not yet the default. Persistent recovery, cancellation
and complete rendered-scene application gates remain unfinished.

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
