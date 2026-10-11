# Engine development baseline

Current scope: Rust engine, evaluations, compatible API transport and local browser
playground. Editor clients, emitters, Lua dependencies, editor test runners and
editor-install CI are removed. Prior adapter implementations remain available in
Git history, not the active tree.

## Environment

- Stable Rust toolchain with rustfmt/clippy.
- Python 3 for fixture metadata and authored replay rebuilds.
- Browser for the loopback playground; no frontend build dependencies.
- `curl` for explicitly requested compatible API calls.
- Optional Node + Playwright for browser smoke tests. These are not runtime deps.

## Verification

```sh
cargo test --manifest-path core/Cargo.toml --all --locked
cargo fmt --manifest-path core/Cargo.toml --all --check
cargo clippy --manifest-path core/Cargo.toml --all-targets --locked -- -D warnings
python3 core/eval/verify-fixtures.py
python3 -m unittest discover -s core/eval -p 'test_*.py'
```

Tests must run without any installed editor or personal editor configuration.
External-provider tests are separate opt-in observations, never prerequisites for
offline CI. Canned replay validates contracts/evaluators, not model accuracy.
