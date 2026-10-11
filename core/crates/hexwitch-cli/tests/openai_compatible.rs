//! Real HTTP transport to loopback mocks; no external APIs or real credentials.
use std::io::{Read, Write};
use std::process::Command;

fn serve(response: String) -> (String, std::thread::JoinHandle<(String, serde_json::Value)>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut raw = vec![];
        let mut block = [0; 4096];
        let (head, body) = loop {
            let n = stream.read(&mut block).unwrap();
            assert!(n > 0);
            raw.extend_from_slice(&block[..n]);
            if let Some(end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8(raw[..end].to_vec()).unwrap();
                let length = head
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse::<usize>().unwrap())
                    })
                    .unwrap();
                if raw.len() >= end + 4 + length {
                    break (
                        head,
                        serde_json::from_slice(&raw[end + 4..end + 4 + length]).unwrap(),
                    );
                }
            }
        };
        assert!(head.contains("Authorization: Bearer mock-compatible-key"));
        stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response}",response.len()).as_bytes()).unwrap();
        (head, body)
    });
    (format!("http://{address}/custom/v1/"), worker)
}

#[test]
fn custom_base_url_key_environment_and_output_modes_reach_chat_completions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval");
    let model_response = std::fs::read_to_string(root.join("llm/clarify-blue.json")).unwrap();
    for mode in ["json_schema", "json_object", "none"] {
        let envelope=serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":model_response}}]}).to_string();
        let (base, worker) = serve(envelope);
        let output = Command::new(env!("CARGO_BIN_EXE_hexwitch"))
            .args([
                "--machine",
                "interpret",
                root.join("themes/nord.json").to_str().unwrap(),
                "Make that blue quieter.",
                "--v1",
                "--base-url",
                &base,
                "--api-key-env",
                "HEXWITCH_TEST_COMPAT_KEY",
                "--model",
                "my/custom-model",
                "--response-format",
                mode,
            ])
            .env("HEXWITCH_TEST_COMPAT_KEY", "mock-compatible-key")
            .env_remove("OPENAI_API_KEY")
            .env_remove("OPENAI_BASE_URL")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("mock-compatible-key"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("HTTP API connection is unencrypted"));
        assert!(!stderr.contains("mock-compatible-key"));
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "needs_clarification");
        let (head, payload) = worker.join().unwrap();
        assert!(head.starts_with("POST /custom/v1/chat/completions HTTP/"));
        assert_eq!(payload["model"], "my/custom-model");
        assert!(
            payload.get("temperature").is_none(),
            "model-specific unsupported sampling parameter injected"
        );
        match mode {
            "json_schema" => {
                assert_eq!(payload["response_format"]["type"], "json_schema");
                let variants = payload["response_format"]["json_schema"]["schema"]["properties"]
                    ["actions"]["items"]["anyOf"]
                    .as_array()
                    .unwrap();
                assert!(variants
                    .iter()
                    .all(|v| v["properties"]["op"]["enum"][0] != "set_color"));
                let boolean = variants
                    .iter()
                    .find(|v| v["properties"]["op"]["enum"][0] == "set_bool")
                    .unwrap();
                assert_eq!(boolean["properties"]["scope"]["type"], "null");
            }
            "json_object" => assert_eq!(payload["response_format"]["type"], "json_object"),
            _ => assert!(payload.get("response_format").is_none()),
        }
    }
}
#[test]
fn base_url_environment_is_supported_and_missing_key_errors_name_only_variable() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval");
    let response=serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":std::fs::read_to_string(root.join("llm/clarify-blue.json")).unwrap()}}]}).to_string();
    let (base, worker) = serve(response);
    let success = Command::new(env!("CARGO_BIN_EXE_hexwitch"))
        .args([
            "--machine",
            "interpret",
            root.join("themes/nord.json").to_str().unwrap(),
            "Which blue?",
            "--v1",
            "--api-key-env",
            "HEXWITCH_ENV_TEST_KEY",
        ])
        .env("OPENAI_BASE_URL", base)
        .env("HEXWITCH_ENV_TEST_KEY", "mock-compatible-key")
        .output()
        .unwrap();
    assert!(success.status.success());
    worker.join().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_hexwitch"))
        .args([
            "--machine",
            "interpret",
            root.join("themes/nord.json").to_str().unwrap(),
            "test",
            "--v1",
            "--api-key-env",
            "HEXWITCH_MISSING_TEST_KEY",
        ])
        .env("OPENAI_BASE_URL", "https://gateway.example/v1")
        .env_remove("HEXWITCH_MISSING_TEST_KEY")
        .output()
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!output.status.success());
    assert!(response["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("HEXWITCH_MISSING_TEST_KEY"));
}
