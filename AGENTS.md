# Repository rules

## Build artifacts

- After running Rust builds, checks, tests, or browser tests that build Rust, clean local build artifacts before finishing the task, even if verification fails.
- Run from the repository root: `cargo clean --manifest-path core/Cargo.toml --target-dir core/target`.
- Stop any processes started for the task before cleaning. If a user-requested server must keep running, defer cleanup and report it.
- Clean any alternate target directory created for the task too. Do not delete shared Cargo caches, toolchains, installed binaries, or unrelated projects' artifacts.
- Do not commit build outputs or leave repository-local caches and binaries behind.

## Source code

- Do not add source or configuration comments or docstrings, including Rust documentation comments, Python docstrings, or JavaScript, CSS, and HTML comments.
- Use clear names and small functions instead. Keep prose documentation in Markdown files.
- Preserve executable directives, required license notices, generated upstream files, and comment syntax inside runtime strings or test fixtures when it is data rather than source commentary.
- Preserve user-facing CLI help as explicit Clap `help`/`about` attributes, not documentation comments.
