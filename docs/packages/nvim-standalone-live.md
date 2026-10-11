# Standalone Neovim live verification

## Reproduction and fixes

Tested the real release executable in `nvim --clean --headless`, with only the
Hexwitch runtime directory, shipped Gruvbox excerpt and synthetic requests.
No personal Neovim configuration/code/image was transmitted. The live tests used
the user-selected OpenAI-compatible HTTP gateway and `FAST_SMALLER` model; the
credential was inherited from its named environment variable and not printed.

Failures reproduced (not dismissed as config problems):

1. Model attached `scope=request` to `set_bool`. Local validation correctly
   refused irrelevant fields. Fixed provider schema to use per-operation anyOf
   branches with unused fields forced to null; used fields are non-null.
2. Model copied a theme inventory color into `set_color` for readability. Local
   trusted-anchor validation refused it. Fixed request-specific schema to remove
   set_color unless opaque color anchors were supplied, then constrain its enum.
3. Natural-language contrast response omitted persistent=false. Fixed operation
   branch to require a concrete boolean and prompt guidance to distinguish false
   from null. No fields are silently discarded or guessed during validation.
4. Ordinary :Hexwitch used legacy interpretation even while testing v1. Added
   setup interpretation='v1', used by the standalone launcher and user's local
   plugin spec. Legacy :HexwitchReplay explicitly retains the legacy protocol.

## Verified flows

After fixes, two consecutive extended live runs passed with ordinary prompts:

- "No italics anywhere."
- "Keep the background color and make comments easier to read."
- Ambiguous blue request requiring clarification.

Each run checked actual emitted highlight properties, reference loading,
candidate review/no auto-apply, application, preserved base and unrelated syntax
colors, comment contrast >=4.5 against the unchanged background, clarification
without mutation/candidate, undo/redo and restoration of original highlights.
Previous intermediate failures and successful bounded rechecks were used for
reproduction; this is not a broad model benchmark or proof of all provider behavior.

Offline Rust/Neovim tests exercise schema restrictions, strict rejection of the
live invalid scope fixture, preserved legacy replay and standalone initialization.
`adapters/nvim/tests/live.lua` is opt-in and not run by CI.

## Interactive playground (no installed plugins/user config)

```sh
OPENAI_BASE_URL='http://your-gateway' \
HEXWITCH_API_KEY_ENV='YOUR_EXISTING_KEY_ENV' \
HEXWITCH_MODEL='your-model' \
sh scripts/nvim-standalone.sh
```

The key itself must already be set in that environment. The launcher does not
read a credential file or print it. It sets NVIM_APPNAME=hexwitch-standalone to
keep artifacts/state separate, uses --clean, loads a bundled reference and sample
Lua buffer, and selects interpretation=v1. Ordinary :Hexwitch is now the tested
property-aware path. No auto-build or downloads occur.

## Automated live smoke (explicit opt-in)

```sh
HEXWITCH_LIVE_BASE_URL='https://api.example/v1' \
HEXWITCH_LIVE_KEY_ENV='YOUR_KEY_ENV' \
HEXWITCH_LIVE_MODEL='your-model' \
HEXWITCH_LIVE_EXTENDED=1 \
HEXWITCH_LIVE_ARTIFACTS='/path/to/private/test-artifacts' \
nvim --clean --headless -l adapters/nvim/tests/live.lua
```

Extended smoke makes three real requests and may incur provider charges. Without
extended mode it makes one request. It refuses missing keys; it never falls back
to replay. HTTP warns because credentials/prompts are unencrypted.

## Limits

Verified locally with Neovim 0.12.5; CI's 0.10 baseline has offline tests, not these
live calls. Other models/providers and json_object/none modes still have strict
validation but may need separate live conformance. No automatic retry on model
schema failure. Full session persistence/cancellation, rendered overlay conformance
and installed-theme capture are not implemented. Success on the bundled workflow
is not a promise that every natural-language aesthetic request is expressible.
