# Local engine playground

## Delivered scope

The browser controls call the actual Rust engine through a loopback host in the CLI.
No editor adapter, emitter, Lua plugin, editor launcher or editor-specific test is
left in the active tree. Theme JSON remains a reference-input path; personal editor
configuration is untouched. Repository naming/remote URL is unchanged.

Workspace: theme preview on the left, session on the right. One composer with a
model chip (or Offline, which captures the exact prompt); click-to-nudge popovers on
tokens, swatches and the background (lightness, chroma, warmer/cooler hue,
readability floor, exact color, typography, underline, opacity, locks, semantic
inclusion) that stack onto the pending candidate; Before/After compare and
hold-Space peek with changed tokens outlined; a review card that compares
validation against the current theme so pre-existing failures are not blamed on
the edit; history as session steps with Go back, plus ⌘Z/⇧⌘Z; theme picker with
bundled references and import; a More menu for save/open session, contrast
checker, providers, engine inspector with explicit IntentSpecs, offline replay
and the regression matrix.

Providers: a guided three-step connect flow (provider, key, model list) and a
Providers & models page. The host keeps a provider registry saved to
`~/.config/hexwitch/providers.json` (`$XDG_CONFIG_HOME`,
`$HEXWITCH_PROVIDERS_FILE` or `--providers-file` override it), written owner-only
with key **variable names only**. Pasted keys live in host memory for the run and
never reach the file, the browser or exports. Launch flags become a read-only,
initially selected provider. Model catalogs are fetched server-side without prompt
data; local servers are called without an Authorization header.

Live requests: the host exposes `GET /api/live` (phase, model, host, elapsed time)
and `POST /api/live/cancel`, which kills the in-flight `curl` so the request ends
promptly with `provider cancelled`. The page polls once a second while waiting,
shows a timer and progress toward the 2-minute limit, says when it cannot reach
the engine, and turns provider errors into explained failure cards with retry.
HTTP 400/422, 404 and 5xx now map to `bad_request`, `not_found` and
`server_error` instead of a generic `http` code.

Themes: `playground/themes/omarchy.json` bundles Omarchy's 23 official themes,
pinned to commit `e71a816`, MIT (notice in `playground/themes/OMARCHY-LICENSE`).
Each is a curated excerpt: the theme's `colors.toml` rendered through Omarchy's
own `default/themed/vscode-theme.json.tpl`, keeping the workbench keys the engine
imports, the semantic token colors, and one rule per generic syntax scope, so
language-specific rules cannot override base roles. Regenerate with
`python3 playground/themes/build-omarchy.py --colors <dir> --template <tpl>
--revision <sha>` from files copied at that revision. They are previews of
Omarchy's palettes, not proof of editor rendering.

Rust owns accepted history, revision checks, candidate computation and validation.
The browser is a portable visualization; it does not tokenize source through an
editor grammar or prove editor/runtime conformance.

## Verification

- Offline engine and loopback-host tests pass without installed editors.
- Python fixture/hash and authored rebuild tests pass.
- Host tests exercise real compatible HTTP transport to a mock, consent, origin
  rejection, revision guards, export and restoration, plus the provider registry:
  environment checks, catalog listing, save/select/remove, persistence across a
  restart, keyless local calls, read-only launch providers, and that neither
  environment nor pasted keys appear in the file, bootstrap or exports. No live
  API occurs in CI.
- The optional headless Chrome smoke covers nudges and stacking, compare,
  keep/undo/redo, semantic italics, the readability floor, matrix, theme switch,
  replay clarification, the offline prompt and the full connect-a-model flow
  against an in-process mock provider. It was rewritten for the redesigned UI and
  has not yet been run against it.

### Real provider observations

Bounded browser runs used `http://slingring.procol.tech`, `FAST_SMALLER`, and the
server-side `SLINGRING_API_KEY`, with bundled synthetic state only. Key values were
not printed or written to the browser/export. Multiple runs correctly produced:

- no italics across lexical and semantic styles, without recoloring;
- readable comments at calculated contrast >=4.5, with background color preserved;
- darker background without touching opacity or other properties;
- questions for ambiguous blue and an actual second provider call after an answer.

**The full live smoke did not pass.** Saturation clarification answers repeatedly
produced incorrect properties: decoration opacity, zero foreground opacity, or an
absent keyword background instead of foreground chroma. One run failed with an
HTTP transport error. These are observed development runs, not an accuracy score.

The provider schema was tightened with operation-specific property enums and
numeric ranges, including separate integer blend and opacity branches. Prompt
guidance explicitly distinguishes saturation from opacity and defaults to
preservation. These do not prove semantic correctness. Invalid/absent targets are
rejected locally; wrong-but-valid actions still require review. Reconstructed
negative probes assert that semantic evaluation catches saturation-as-opacity.
The live smoke retains the original correctness assertion rather than accepting
an opacity edit as success.

## Run

```sh
cargo run --manifest-path core/Cargo.toml -p hexwitch-cli -- playground \
  --base-url http://slingring.procol.tech \
  --api-key-env SLINGRING_API_KEY --model FAST_SMALLER
```

Open the printed `http://127.0.0.1:7331` URL. A bare `playground` starts fully usable
offline; connect providers from the page. Launch flags are optional. `--port 0` chooses an available port. Static files are compiled into the
binary; rebuild after editing `playground/`. Stop with Ctrl-C; manually export to
persist a session.

### Optional browser smoke

Install Playwright outside this repository and supply its node_modules directory
through `NODE_PATH`. Use installed Chrome with an isolated headless context, or
Playwright's browser installation. No existing browser profile is opened.

```sh
NODE_PATH=/path/to/node_modules \
HEXWITCH_CHROME='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
node playground/smoke.cjs
```

Use `HEXWITCH_BIN` to select a binary and `HEXWITCH_SCREENSHOT` for screenshot path.
Default screenshots go to ignored `experiments/`. Live testing sends only the
script's bundled references/synthetic prompts and must be requested explicitly:

```sh
OPENAI_BASE_URL=http://slingring.procol.tech \
HEXWITCH_API_KEY_ENV=SLINGRING_API_KEY HEXWITCH_MODEL=FAST_SMALLER \
NODE_PATH=/path/to/node_modules \
HEXWITCH_CHROME='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
HEXWITCH_FAILURE_BUNDLE=experiments/live-failure.json \
node playground/smoke.cjs --live
```

This command saves a failure bundle if its path is configured and can fail on the
known model-quality issue; that is intentional, not a skipped assertion. Review
exported prompts/responses before sharing. There are no automatic paid retries.

## Limitations

- One in-memory experiment per host; tabs share state. No autosave/crash recovery.
- Pasted keys are forgotten when the host stops; providers that use them show
  "key missing" until the key is pasted again. Environment-variable keys are read
  from the host's environment at start, so exporting a new variable needs a restart.
- Progress is phase-level (waiting, checking); providers are not streamed.
- Preview snippets are token sketches per language, not parsed code; the lexical
  role preview and calculated validation scenes remain fixed.
- Partial candidates may contain refusals. User acknowledgement is not a quality pass.
- 8 MiB inbound bundle limit; very large exported histories may exceed restoration.
- Curated theme excerpts have qualified provenance; only pinned import fixtures
  have enforced upstream license/revision metadata. No release eligibility claim.
- Canned matrix scores establish contract/evaluator behavior, not model accuracy,
  fresh blind holdout performance, aesthetics or productivity.

See [experiment contract](../contracts/experiment-v1.md) and
[future engine work](../../FUTURE_PLAN.md).
