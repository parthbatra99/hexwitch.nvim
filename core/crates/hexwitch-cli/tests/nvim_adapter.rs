//! Run the Lua adapter suite against this build of the real engine.
//! No API keys, external plugins, user config, or provider calls.
use std::process::Command;

#[test]
fn thin_adapter_drives_the_real_engine() {
    if !Command::new("nvim")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
    {
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
    let mut child = Command::new("nvim")
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
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("start isolated Neovim");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("headless adapter suite exceeded 60s");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "adapter tests failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
