# First-wave package handoff

Package: P1 contracts plus adaptation regression foundations
Objective: establish real v1 consumers and visible failure reporting before new adapters
Dependencies: P0 checkpoint fce4841
Allowed paths: integration-owned core contracts/CLI and focused adapter fixes
Contract version: IR/intent/protocol 1; legacy edit spec 0 accepted explicitly
Required positive tests: independent opacity/color locks, generated lock origin,
sequential unlock, typography reset, IR roundtrip, protocol operations, JSONC/include,
bidirectional quantized solver, real Neovim loading, TCP mocked provider
Required failure tests: invalid numeric/selector/version/state, malformed import,
include cycle/escape, transactional final requirements, unknown scene ordering,
provider refusal/truncation/rate limit, artifact symbolic link and collision
Explicitly out of scope: image/synthesis/distinct candidates, production adapter
conformance, live model preference evaluation, prebuilt release artifacts
Acceptance gate: full offline tests + formatting/clippy; no phase gate implied

## Compatibility changes

- Explicit IR version 1 required; use import --migrate-v0 for old IR.
- Legacy EditSpec contrast now modifies only foreground and rechecks final floors.
- Invalid ratios (>21) are schema/input errors, not solver infeasibility.
- Arbitrary missing selectors are errors, no longer silent no-ops.
- Model unknown fields/missing actions/irrelevant non-null parameters rejected.
- Provider selection required; no automatic credential discovery; no automatic
  schema/network retry. Provider must support json_schema output (discovered).
- Atomic writes are per artifact, private mode at creation; multi-file rollback
  is not implemented. Machine domain outcomes can be partial without process error.
- Neovim alpha flattening against base is explicitly reported as a loss and is
  not a claim of faithful context-sensitive transparency. Typography false/clear
  lowers as explicit flags; corrected Neovim dotted/dashed attribute names.
- Legacy adapter preserved, no unrelated deletion. It still uses its old protocol;
  full v1 property/session UI adoption is pending.

## Evidence

104 tests before final boundary additions; final count is recorded in status ledger.
Includes real local Neovim 0.12.5 roundtrips and TCP mock curl transport.
No personal data or live network provider evaluation was sent.

## Limitations

Detailed package status is in docs/IMPLEMENTATION_STATUS.md. In particular,
representative dotted scope resolution is not a complete TextMate evaluator;
semantic source rules remain preserved/reported but not applied. Image consent,
ICC/orientation and limits are not implemented. Human labels need independent
review. Release OS matrix, action SHA pinning and dependency/license inventory
remain gates, not completed capabilities.
