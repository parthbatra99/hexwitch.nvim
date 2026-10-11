//! Standalone launcher initialization against this build; no user configuration.
use std::process::{Command, Stdio};
#[test]
fn standalone_launcher_initializes_without_plugins_or_personal_config() {
    if !Command::new("nvim")
        .arg("--version")
        .output()
        .is_ok_and(|r| r.status.success())
    {
        assert!(std::env::var("CI").is_err());
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let mut child=Command::new("sh").arg(root.join("scripts/nvim-standalone.sh")).args(["--headless","-c", "lua assert(vim.wait(15000, function() return require('hexwitch.session').active() ~= nil end, 20)); assert(require('hexwitch.config').get().interpretation == 'v1'); print('PASS standalone initialization')", "-c","qa!"]).env("HEXWITCH_BINARY",env!("CARGO_BIN_EXE_hexwitch")).env_remove("OPENAI_API_KEY").env_remove("SLINGRING_API_KEY").stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("standalone initialization timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PASS standalone initialization")
            || String::from_utf8_lossy(&output.stdout).contains("PASS standalone initialization")
    );
}
