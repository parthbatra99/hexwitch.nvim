# Playground internals

The README covers starting the playground and what it can do. This page covers
how the host works, how it is tested, and what it cannot do yet.

The browser calls the Rust engine through a loopback host built into the CLI.
Rust owns accepted history, revision checks, candidate computation and
validation. The browser only draws: it does not tokenize source with an editor
grammar, and its preview says nothing about how an editor would render the theme.
Static files are compiled into the binary, so rebuild after editing `playground/`.

## Providers

The host keeps a provider registry in `~/.config/hexwitch/providers.json`.
`$XDG_CONFIG_HOME`, `$HEXWITCH_PROVIDERS_FILE` or `--providers-file` override the
location. The file is written owner-only and stores environment variable names,
never key values. Pasted keys stay in host memory for the run and never reach the
file, the browser or exports. Launch flags become a read-only provider that is
selected on start. Model catalogs are fetched server-side without prompt data,
and local servers are called without an Authorization header.

Transport rules are in the [IR contract](contracts/ir-v1.md#api-connection).

## Live requests

`GET /api/live` reports phase, model, host and elapsed time.
`POST /api/live/cancel` kills the in-flight `curl`, which ends the request with
`provider cancelled`. The page polls once a second while waiting and shows
progress toward the 2-minute limit. Provider errors become failure cards with a
retry button. HTTP 400/422, 404 and 5xx map to `bad_request`, `not_found` and
`server_error`.

## Omarchy themes

`playground/themes/omarchy.json` bundles Omarchy's 23 official themes, pinned to
commit `e71a816` (MIT, notice in `playground/themes/OMARCHY-LICENSE`). Each one
is the theme's `colors.toml` rendered through Omarchy's
`default/themed/vscode-theme.json.tpl`, cut down to the workbench keys the engine
imports, the semantic token colors and one rule per generic syntax scope. Dropping
language-specific rules keeps them from overriding base roles. Regenerate from
files copied at that revision:

```sh
python3 playground/themes/build-omarchy.py --colors <dir> --template <tpl> --revision <sha>
```

## Tests

Offline host tests run real HTTP against a mock provider. They cover consent,
origin rejection, revision guards, export and restore, and the provider registry:
environment checks, catalog listing, save/select/remove, persistence across a
restart, keyless local calls, read-only launch providers, and that no key value
shows up in the file, the bootstrap payload or an export. CI makes no live calls.

The optional headless Chrome smoke covers nudges, compare, keep/undo/redo,
semantic italics, the readability floor, the matrix, theme switching, replay
clarification, the offline prompt and the connect-a-model flow against an
in-process mock. It was rewritten for the redesigned UI and has not been run
against it yet.

Install Playwright outside the repository and point `NODE_PATH` at its
`node_modules`. The smoke opens an isolated headless context, never an existing
browser profile:

```sh
NODE_PATH=/path/to/node_modules \
HEXWITCH_CHROME='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
node playground/smoke.cjs
```

`HEXWITCH_BIN` selects a binary and `HEXWITCH_SCREENSHOT` sets the screenshot
path; screenshots default to the ignored `experiments/` directory.

The live variant sends only the script's bundled references and prompts, and
runs only when asked for:

```sh
OPENAI_BASE_URL=http://slingring.procol.tech \
HEXWITCH_API_KEY_ENV=SLINGRING_API_KEY HEXWITCH_MODEL=FAST_SMALLER \
NODE_PATH=/path/to/node_modules \
HEXWITCH_CHROME='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
HEXWITCH_FAILURE_BUNDLE=experiments/live-failure.json \
node playground/smoke.cjs --live
```

It saves a failure bundle when the path is set and is expected to fail on the
saturation issue below. Review exported prompts and responses before sharing
them. Nothing retries automatically.

## Live model runs

Runs used `http://slingring.procol.tech`, `FAST_SMALLER` and the server-side
`SLINGRING_API_KEY`, with bundled synthetic state only. Across several runs the
model got these right:

- no italics across lexical and semantic styles, with no recoloring
- comments raised to contrast 4.5 or more, background color kept
- a darker background with opacity and other properties untouched
- a question for an ambiguous "blue", then a second provider call after the answer

The full live smoke did not pass. Answers to the saturation clarification
changed the wrong property: decoration opacity, foreground opacity set to zero,
or a keyword background that does not exist, where foreground chroma was wanted.
One run failed with an HTTP transport error. These are development runs, not an
accuracy score.

The provider schema now has per-operation property enums and numeric ranges, with
opacity and integer blend in separate branches, and the prompt separates
saturation from opacity. That narrows what the model can send; it does not make
the answer correct. Reconstructed probes check that semantic evaluation flags
saturation-as-opacity, and the live smoke still asserts the correct property
instead of accepting an opacity edit.

## Limitations

- One in-memory experiment per host, shared by all tabs. No autosave or crash recovery.
- Pasted keys are lost when the host stops, and providers using them show "key
  missing" until the key is pasted again. Environment keys are read at start, so
  a newly exported variable needs a restart.
- Progress is reported per phase (waiting, checking). Responses are not streamed.
- Preview snippets are per-language token sketches, not parsed code.
- A partial candidate can contain refusals. Accepting it is not a quality pass.
- Restore accepts at most 8 MiB, which a long exported history can exceed.
- Only the pinned import fixtures carry enforced license and revision metadata.
