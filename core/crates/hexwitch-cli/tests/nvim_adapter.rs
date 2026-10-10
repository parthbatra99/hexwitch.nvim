//! Run the Lua adapter suite against this build of the real engine.
//! No API keys, external plugins, user config, or provider calls.
use std::process::Command;

#[test]
fn thin_adapter_drives_the_real_engine() {
    if Command::new("nvim").arg("--version").output().is_err() {
        assert!(
            std::env::var("CI").is_err(),
            "Neovim must be installed in CI"
        );
        eprintln!("skipping adapter tests: nvim not on PATH");
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let adapter = root.join("adapters/nvim");
    let output = Command::new("nvim")
        .args([
            "--headless",
            "--clean",
            "-c",
            "lua dofile(vim.env.HEXWITCH_TEST_RUNNER)",
        ])
        .env("HEXWITCH_TEST_RUNNER", adapter.join("tests/run.lua"))
        .env("HEXWITCH_TEST_ROOT", root)
        .env("HEXWITCH_BINARY", env!("CARGO_BIN_EXE_hexwitch"))
        .env_remove("OPENAI_API_KEY")
        .env_remove("OPENROUTER_API_KEY")
        .output()
        .expect("start isolated Neovim");
    assert!(
        output.status.success(),
        "adapter tests failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
