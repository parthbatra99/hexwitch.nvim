# Reproducible prototype baseline

## Scope

P0 checkpoints the existing thin adapter and preserves `adapters/nvim-legacy/`.
Neither the legacy plugin nor its history is removed. `FUTURE_PLAN.md` remains
the planning baseline; package evidence belongs in `docs/IMPLEMENTATION_STATUS.md`.

## Local environment verified

- macOS, Rust stable (workspace `core/rust-toolchain.toml`).
- Neovim 0.10+ required for the thin client (`vim.system`, `nvim_get_hl`).
- Engine requires no network for regression tests. Provider calls are opt-in.
- Legacy tests additionally need Plenary and Telescope; the CI job retains
  its relocated working directory and minimal init.

## Commands

```sh
cargo fmt --manifest-path core/Cargo.toml --all --check
cargo clippy --manifest-path core/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path core/Cargo.toml --all --locked
```

Initial checkpoint: 84 tests pass, including the real-binary thin-adapter test
and real headless-Neovim emitted-highlight checks. Formatting and clippy pass.
These are functionality checks, not evidence of aesthetic quality, faithful
complete-theme import, live-provider reliability, or comprehensive readability.
Legacy unit tests are a separate CI suite; do not imply they ran locally without
its optional dependencies.

## Integration ownership

The primary session owns shared contracts, `main.rs`, `lib.rs`, manifests,
shared schemas, and integration. Delegated packages use named branches and
worktrees, explicit allowed paths, focused tests, failure fixtures, limitations,
and compatibility notes. Contracts merge before their consumers.
