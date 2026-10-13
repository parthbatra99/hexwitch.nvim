# hexwitch

An editor-independent theme engine and local browser playground. Start from a
reference, make a deliberate change, inspect its property diff and validation,
then accept or reject it. Rust computes the colors; a language model proposes
intent, not a replacement palette.

**Current scope: engine, experiments, and evaluations only.** Editor adapters,
colorscheme emitters, editor installation and release packaging have been removed.
Theme-file import remains a reference-input format, not an editor integration.

## Run the playground

Requires a Rust toolchain and a browser. Live interpretation also requires `curl`.
No JavaScript build, plugins, or personal editor configuration are needed.

```sh
cargo run --manifest-path core/Cargo.toml -p hexwitch-cli -- playground
```

Open **http://127.0.0.1:7331**. Use `--port 0` for an available port; the host prints
the URL. The host is loopback-only, keeps one experiment in memory, and stops with
Ctrl-C. Export before stopping if you want to resume.

### Connect a model

Nudges, checks and history work offline. To describe changes in plain words,
open **Connect a model** (from the welcome card, the model chip, or
**More → Providers & models**) and follow the three steps:

1. **Pick a provider:** OpenAI, OpenRouter, a server on this machine (Ollama,
   LM Studio), or any OpenAI-compatible API root such as a team gateway.
2. **Add your key:** name an environment variable the engine can read
   (recommended; the page tells you whether it is set), or paste a key that the
   engine keeps in memory until it stops. Local servers need no key.
3. **Build your model list:** the engine fetches the provider's catalog (no prompt
   data is sent); pick the models you use and star a default. They appear under
   the model chip next to the composer.

Providers are saved to `~/.config/hexwitch/providers.json` (or
`$XDG_CONFIG_HOME`, `$HEXWITCH_PROVIDERS_FILE`, `--providers-file`) with key
**variable names only**; pasted keys are never written to disk, sent to the
browser, or exported. `/chat/completions` and `/models` are appended to the API
root; no `/v1` is added. HTTP is supported but unencrypted. Nothing is sent until
you pick a model and press send, and the composer always names the host it sends
to. Automatic retries are disabled.

Launch flags still work and appear as a read-only provider, selected on start:

```sh
cargo run --manifest-path core/Cargo.toml -p hexwitch-cli -- playground \
  --base-url http://slingring.procol.tech \
  --api-key-env SLINGRING_API_KEY \
  --model FAST_SMALLER
```

## What you can test

The workspace has two halves: the theme preview on the left, the session on the
right.

- **Talk:** describe a change in the composer. The model chip picks the provider
  and model, or **Offline**, which shows the exact prompt instead of sending it.
  Clarifying questions appear in the session; answer them in the composer.
- **Nudge:** click any token, swatch or the background to open a popover with
  lighter/darker, more vivid/muted, warmer/cooler, a 4.5:1 readability floor,
  an exact color, typography, underline, opacity and locks. Optionally include
  semantic rules. Nudges stack onto the pending change until you keep it.
- **Review:** changed tokens are outlined; toggle Before/After or hold Space.
  The review card lists property changes and compares validation against the
  current theme, so pre-existing failures are not blamed on the edit. Keep
  needs a second look ("Keep anyway") only when a check got worse.
- **History:** every kept change is a step in the session; **Go back** jumps to
  any step, and ⌘Z / ⇧⌘Z step through them.
- **Watching a request:** while a model works, the session shows a live card
  with the elapsed time (confirmed by the engine every second), what it is
  waiting on, and a **Stop** button that really kills the request. Failures
  become cards that say what went wrong (bad key, unknown model, timeout,
  unreachable host...) with **Try again**, **Change model** and **Providers**.
- **References:** the built-in Eval ink/paper states plus all 23 official
  [Omarchy](https://github.com/basecamp/omarchy) themes in a searchable picker;
  import IR or theme JSON/JSONC. Preview TypeScript, Rust, Python or Go
  under normal, selection, search, added-line and floating scenes.
- **More menu:** save/open sessions (export/restore including pending
  candidates and questions), contrast checker, providers, and developer tools:
  engine inspector with explicit IntentSpecs, replay of saved model JSON, and the
  74-case regression matrix with three paired checks.

Browser rendering is a portable role visualization, **not editor conformance**.
Model output is not automatically proven to preserve intent; a schema-valid wrong
edit can still become a reviewable candidate. Check the diff before accepting.

## Engine boundaries

```text
reference → IR → intent → deterministic revision → validation → candidate review
                         ↑                                  ↓
                 optional language model              accept / reject
```

| Path | Purpose |
|---|---|
| `core/crates/hexwitch-core/` | Color math, IR, import, semantic resolution, intent, validation, experiment sessions, evaluators |
| `core/crates/hexwitch-cli/` | Scriptable engine operations, compatible API transport, loopback playground host |
| `playground/` | Browser controls/preview and opt-in browser smoke test |
| `core/eval/` | Authored references, semantic contracts, replay runs, mutations, fixture verification |
| `docs/` | Status, playground internals, versioned engine and experiment contracts |

Legacy CLI edit/replay formats remain for regression compatibility. The playground
uses property-aware intent v1. Neither mode emits editor artifacts.

## Verify

```sh
cargo fmt --manifest-path core/Cargo.toml --all --check
cargo clippy --manifest-path core/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path core/Cargo.toml --all --locked
python3 core/eval/verify-fixtures.py
python3 -m unittest discover -s core/eval -p 'test_*.py'
```

Optional real-browser checks use Playwright; see [playground verification and
limitations](docs/playground.md). Live testing is opt-in and separate
from offline CI. The configured small model has demonstrated valid-but-wrong
saturation/opacity interpretations; this is an open quality issue, regardless
of canned replay scores.

[Status](docs/status.md) ·
[IR/intent contract](docs/contracts/ir-v1.md) ·
[Experiment contract](docs/contracts/experiment-v1.md) ·
[Evaluation methodology](core/eval/README.md)
