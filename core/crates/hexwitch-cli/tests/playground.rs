use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
struct Host {
    child: Child,
    address: String,
    token: String,
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn scratch_file() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "hexwitch-providers-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
impl Host {
    fn new(extra: &[&str]) -> Self {
        let file = scratch_file();
        let host = Self::with_file(&file, extra);
        let _ = std::fs::remove_file(file);
        host
    }
    fn with_file(file: &std::path::Path, extra: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_hexwitch"))
            .args(["playground", "--port", "0", "--providers-file"])
            .arg(file)
            .args(extra)
            .env("HEXWITCH_PLAYGROUND_TEST_KEY", "mock-playground-key")
            .env_remove("OPENAI_BASE_URL")
            .env_remove("HEXWITCH_UNSET_TEST_KEY")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let address = line
            .trim()
            .strip_prefix("Hexwitch engine playground: http://")
            .expect("host prints address")
            .to_string();
        let mut host = Self {
            child,
            address,
            token: String::new(),
        };
        let (_, bootstrap) = host.http("GET", "/api/bootstrap", None, None);
        host.token = serde_json::from_str::<Value>(&bootstrap).unwrap()["token"]
            .as_str()
            .unwrap()
            .to_string();
        host
    }
    fn bootstrap(&self) -> Value {
        self.get_json("/api/bootstrap")
    }
    fn get_json(&self, path: &str) -> Value {
        let (_, body) = self.http("GET", path, None, None);
        serde_json::from_str(&body).unwrap()
    }
    fn http(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
        origin: Option<&str>,
    ) -> (String, String) {
        let body = body.map(|v| v.to_string()).unwrap_or_default();
        let mut stream = TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(20)))
            .unwrap();
        let origin = origin
            .map(str::to_string)
            .unwrap_or_else(|| format!("http://{}", self.address));
        write!(stream,"{method} {path} HTTP/1.1\r\nHost: {}\r\nOrigin: {origin}\r\nX-Hexwitch-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",self.address,self.token,body.len()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (head, body) = response.split_once("\r\n\r\n").unwrap();
        (head.to_string(), body.to_string())
    }
    fn post(&self, path: &str, body: Value) -> Value {
        let (head, body) = self.http("POST", path, Some(body), None);
        assert!(head.starts_with("HTTP/1.1 200"), "{head}");
        serde_json::from_str(&body).unwrap()
    }
}
#[test]
fn host_assets_security_review_history_export_and_matrix() {
    let host = Host::new(&[]);
    let (head, html) = host.http("GET", "/", None, None);
    assert!(head.contains("Content-Security-Policy"));
    assert!(html.contains("/app.js"));
    assert!(html.contains("/favicon.svg"));
    let (head, svg) = host.http("GET", "/favicon.svg", None, None);
    assert!(head.contains("image/svg+xml"));
    assert!(svg.starts_with("<svg"));
    let (_, js) = host.http("GET", "/app.js", None, None);
    assert!(js.contains("'use strict'"));
    let (head, _) = host.http(
        "POST",
        "/api/reject",
        Some(json!({"revision":0})),
        Some("https://evil.example"),
    );
    assert!(head.starts_with("HTTP/1.1 403"));
    let no_consent = host.post(
        "/api/prompt",
        json!({"revision":0,"mode":"live","request":"Darker background"}),
    );
    assert_eq!(no_consent["ok"], false);
    let inspected = host.post(
        "/api/prompt",
        json!({"revision":0,"mode":"inspect","request":"Make the background darker"}),
    );
    assert_eq!(inspected["ok"], true);
    assert!(inspected["result"]["system"]
        .as_str()
        .unwrap()
        .contains("preserving bg_opacity"));
    let proposed=host.post("/api/revise",json!({"revision":0,"spec":{"version":1,"actions":[{"op":"lightness","target":{"roles":["surface.base"],"property":"bg"},"delta":-0.02}]}}));
    assert_eq!(proposed["ok"], true);
    assert_eq!(
        proposed["result"]["current"]["styles"]["surface.base"]["bg"],
        "#151c25"
    );
    assert_eq!(proposed["result"]["changes"].as_array().unwrap().len(), 1);
    assert_eq!(host.post("/api/accept", json!({"revision":1}))["ok"], false);
    assert_eq!(
        host.post("/api/accept", json!({"revision":0,"allow_unverified":true}))["ok"],
        false
    );
    let exported = host.post("/api/export", json!({}));
    assert!(exported["result"]["candidate"].is_object());
    assert_eq!(
        exported["result"]["theme_hashes"][0]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert_eq!(host.post("/api/reject", json!({"revision":1}))["ok"], true);
    assert_eq!(
        host.post(
            "/api/restore",
            json!({"revision":2,"bundle":exported["result"]})
        )["ok"],
        true
    );
    assert_eq!(
        host.post("/api/accept", json!({"revision":3,"allow_unverified":true}))["ok"],
        true
    );
    assert_eq!(
        host.post("/api/undo", json!({"revision":4}))["result"]["current"]["styles"]
            ["surface.base"]["bg"],
        "#151c25"
    );
    assert_eq!(host.post("/api/redo", json!({"revision":5}))["ok"], true);
    let matrix = host.post("/api/evaluate", json!({}));
    assert_eq!(matrix["ok"], true);
    assert_eq!(matrix["result"]["passed"], 74);
    assert_eq!(matrix["result"]["pair_failures"], 0);
    let contrast = host.post(
        "/api/color",
        json!({"foreground":"#ffffff","background":"#000000"}),
    );
    assert_eq!(contrast["result"]["ratio"], 21.0);
}
#[test]
fn invalid_interpretation_retains_raw_result_without_a_candidate() {
    let host = Host::new(&[]);
    let model = json!({"version":1,"decision":"ready","notes":"Invalid delta probe.","questions":[],"actions":[{"op":"chroma","roles":["syntax.keyword"],"property":"fg","include_semantic":false,"scope":null,"color":null,"number":-2.0,"boolean":null,"underline":null,"state":null,"backgrounds":null,"min":null,"direction":null,"persistent":null}]});
    let result = host.post("/api/prompt", json!({"revision":0,"mode":"replay","request":"Make keyword foreground slightly less saturated.","response":model.to_string()}));
    assert_eq!(result["ok"], true);
    assert_eq!(result["result"]["interpretation"]["outcome"], "error");
    assert!(result["result"]["state"]["candidate"].is_null());
    assert_eq!(
        result["result"]["state"]["last_result"]["model_response"],
        model
    );
    assert_eq!(result["result"]["state"]["revision"], 1);
    let bundle = host.post("/api/export", json!({}));
    assert!(bundle["result"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["kind"] == "interpretation_error"));
}
#[test]
fn wrong_host_missing_token_and_oversized_body_are_rejected() {
    let host = Host::new(&[]);
    for (headers, status) in [
        ("Host: evil.example\r\nContent-Length: 0".to_string(), "403"),
        (format!("Host: {}\r\nOrigin: http://{}\r\nContent-Type: application/json\r\nContent-Length: 0", host.address, host.address), "403"),
        (format!("Host: {}\r\nOrigin: http://{}\r\nX-Hexwitch-Token: {}\r\nContent-Type: application/json\r\nContent-Length: 9000000", host.address, host.address, host.token), "413"),
    ] {
        let mut stream = TcpStream::connect(&host.address).unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
        write!(stream,"POST /api/reject HTTP/1.1\r\n{headers}\r\n\r\n").unwrap();
        let mut reply = String::new(); stream.read_to_string(&mut reply).unwrap();
        assert!(reply.starts_with(&format!("HTTP/1.1 {status}")));
    }
}
fn read_request(stream: &mut TcpStream) -> (String, Value) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let mut data = vec![];
    let mut block = [0; 4096];
    loop {
        let n = stream.read(&mut block).unwrap();
        assert!(n > 0);
        data.extend_from_slice(&block[..n]);
        if let Some(end) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8(data[..end].to_vec()).unwrap();
            let length = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|n| n.trim().parse::<usize>().unwrap())
                })
                .unwrap();
            if data.len() >= end + 4 + length {
                return (
                    head,
                    serde_json::from_slice(&data[end + 4..end + 4 + length]).unwrap(),
                );
            }
        }
    }
}
#[test]
fn live_host_calls_real_transport_and_resumes_clarification_without_credential_leak() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let worker = std::thread::spawn(move || {
        let mut requests = vec![];
        for response in [
            json!({"version":1,"decision":"needs_clarification","notes":"Choose a target.","questions":["Comments #112233 or keywords?"],"actions":[]}),
            json!({"version":1,"decision":"ready","notes":"Reduce comment chroma only.","questions":[],"actions":[{"op":"chroma","roles":["syntax.comment"],"property":"fg","include_semantic":false,"scope":null,"color":null,"number":-0.02,"boolean":null,"underline":null,"state":null,"backgrounds":null,"min":null,"direction":null,"persistent":null}]}),
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let (head, payload) = read_request(&mut stream);
            assert!(head.starts_with("POST /chat/completions HTTP/"));
            assert!(head.contains("Authorization: Bearer mock-playground-key"));
            requests.push(payload);
            let body=json!({"choices":[{"finish_reason":"stop","message":{"content":response.to_string()}}]}).to_string();
            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
        requests
    });
    let host = Host::new(&[
        "--base-url",
        &base,
        "--api-key-env",
        "HEXWITCH_PLAYGROUND_TEST_KEY",
        "--model",
        "test-model",
    ]);
    let first = host.post(
        "/api/prompt",
        json!({"revision":0,"mode":"live","request":"Make blue less intense","consent":true}),
    );
    assert_eq!(first["ok"], true);
    assert!(first["result"]["state"]["clarification"].is_object());
    let second = host.post(
        "/api/prompt",
        json!({"revision":1,"mode":"live","answer":"Comments","consent":true}),
    );
    assert_eq!(second["ok"], true);
    assert!(second["result"]["state"]["candidate"].is_object());
    let requests = worker.join().unwrap();
    let user = &requests[1]["messages"][1]["content"].as_str().unwrap();
    assert!(user.contains("User answer: Comments"));
    assert!(!user.contains("#112233"));
    assert!(requests[1]["messages"][0]["content"]
        .as_str()
        .unwrap()
        .contains("Comments #112233 or keywords?"));
    let exported = host.post("/api/export", json!({}));
    assert!(!exported.to_string().contains("mock-playground-key"));
    assert!(exported["result"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["kind"] == "provider_request"));
}
fn serve_mock(
    listener: TcpListener,
    count: usize,
) -> std::thread::JoinHandle<Vec<(String, Value)>> {
    std::thread::spawn(move || {
        let mut seen = vec![];
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            let (head, payload) = read_any_request(&mut stream);
            let body = if head.starts_with("GET ") {
                json!({"data":[{"id":"m-b"},{"id":"m-a"}]})
            } else {
                let reply = json!({"version":1,"decision":"ready","notes":"Darken comments slightly.","questions":[],"actions":[{"op":"lightness","roles":["syntax.comment"],"property":"fg","include_semantic":false,"scope":null,"color":null,"number":-0.02,"boolean":null,"underline":null,"state":null,"backgrounds":null,"min":null,"direction":null,"persistent":null}]});
                json!({"choices":[{"finish_reason":"stop","message":{"content":reply.to_string()}}]})
            }
            .to_string();
            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            seen.push((head, payload));
        }
        seen
    })
}
fn read_any_request(stream: &mut TcpStream) -> (String, Value) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let mut data = vec![];
    let mut block = [0; 4096];
    loop {
        let n = stream.read(&mut block).unwrap();
        assert!(n > 0);
        data.extend_from_slice(&block[..n]);
        if let Some(end) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8(data[..end].to_vec()).unwrap();
            let length = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|n| n.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            if data.len() >= end + 4 + length {
                let payload = if length == 0 {
                    Value::Null
                } else {
                    serde_json::from_slice(&data[end + 4..end + 4 + length]).unwrap()
                };
                return (head, payload);
            }
        }
    }
}
#[test]
fn saved_providers_route_requests_persist_and_never_store_keys() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let mock = serve_mock(listener, 4);
    let file = scratch_file();
    let host = Host::with_file(&file, &[]);
    assert_eq!(host.bootstrap()["providers"]["providers"], json!([]));
    assert!(host.bootstrap()["providers"]["selected"].is_null());
    let env = host.post(
        "/api/providers/env",
        json!({"name":"HEXWITCH_PLAYGROUND_TEST_KEY"}),
    );
    assert_eq!(env["result"]["present"], true);
    let missing = host.post(
        "/api/providers/env",
        json!({"name":"HEXWITCH_UNSET_TEST_KEY"}),
    );
    assert_eq!(missing["result"]["present"], false);
    assert_eq!(
        host.post("/api/providers/env", json!({"name":"sk-live-123"}))["ok"],
        false
    );
    let draft = json!({"kind":"custom","name":"Mock","base_url":base,"key_env":"HEXWITCH_PLAYGROUND_TEST_KEY"});
    let listed = host.post("/api/providers/test", draft.clone());
    assert_eq!(listed["result"]["models"], json!(["m-a", "m-b"]));
    let mut save = draft.clone();
    save["models"] = json!(["m-a", "m-b"]);
    save["default_model"] = json!("m-b");
    let saved = host.post("/api/providers/save", save);
    assert_eq!(saved["result"]["id"], "mock");
    assert_eq!(
        saved["result"]["providers"]["selected"],
        json!({"provider":"mock","model":"m-b"})
    );
    let mut bad = draft.clone();
    bad["key_env"] = json!("sk-live-123");
    assert_eq!(host.post("/api/providers/save", bad)["ok"], false);
    let live = host.post(
        "/api/prompt",
        json!({"revision":0,"mode":"live","request":"Darker comments","consent":true,"provider":"mock","model":"m-a"}),
    );
    assert_eq!(live["ok"], true, "{live}");
    assert!(live["result"]["state"]["candidate"].is_object());
    let pasted = host.post(
        "/api/providers/save",
        json!({"kind":"custom","name":"Pasted","base_url":base,"key":"pasted-secret-123","models":["m-a"]}),
    );
    assert_eq!(pasted["result"]["id"], "pasted");
    assert_eq!(
        host.post("/api/providers/test", json!({"id":"pasted"}))["ok"],
        true
    );
    let local = host.post(
        "/api/providers/test",
        json!({"kind":"local","base_url":base}),
    );
    assert_eq!(local["ok"], true, "{local}");
    let selected = host.post(
        "/api/providers/select",
        json!({"provider":"mock","model":"m-a"}),
    );
    assert_eq!(selected["result"]["selected"]["model"], "m-a");
    let stored = std::fs::read_to_string(&file).unwrap();
    assert!(stored.contains("HEXWITCH_PLAYGROUND_TEST_KEY"));
    for leak in [
        stored.clone(),
        host.bootstrap().to_string(),
        host.post("/api/export", json!({})).to_string(),
    ] {
        assert!(!leak.contains("pasted-secret-123"));
        assert!(!leak.contains("mock-playground-key"));
    }
    let seen = mock.join().unwrap();
    assert!(seen[0].0.starts_with("GET /models "));
    assert!(seen[0]
        .0
        .contains("Authorization: Bearer mock-playground-key"));
    assert!(seen[1].0.starts_with("POST /chat/completions "));
    assert_eq!(seen[1].1["model"], "m-a");
    assert!(seen[2]
        .0
        .contains("Authorization: Bearer pasted-secret-123"));
    assert!(!seen[3].0.to_ascii_lowercase().contains("authorization"));
    drop(host);
    let restarted = Host::with_file(&file, &[]);
    let providers = restarted.bootstrap()["providers"].clone();
    assert_eq!(
        providers["selected"],
        json!({"provider":"mock","model":"m-a"})
    );
    let list = providers["providers"].as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["key_ready"], true);
    assert_eq!(list[1]["key_source"], "session");
    assert_eq!(list[1]["key_ready"], false);
    let removed = restarted.post("/api/providers/remove", json!({"id":"pasted"}));
    assert_eq!(removed["result"]["providers"].as_array().unwrap().len(), 1);
    assert!(!std::fs::read_to_string(&file).unwrap().contains("Pasted"));
    drop(restarted);
    let _ = std::fs::remove_file(file);
}
#[test]
fn launch_flags_become_a_read_only_selected_provider() {
    let host = Host::new(&[
        "--base-url",
        "http://127.0.0.1:9/v1",
        "--api-key-env",
        "HEXWITCH_PLAYGROUND_TEST_KEY",
        "--model",
        "launch-model",
    ]);
    let providers = host.bootstrap()["providers"].clone();
    assert_eq!(
        providers["selected"],
        json!({"provider":"launch","model":"launch-model"})
    );
    assert_eq!(providers["providers"][0]["launch"], true);
    assert_eq!(providers["providers"][0]["key_ready"], true);
    let edit = host.post(
        "/api/providers/save",
        json!({"id":"launch","kind":"custom","base_url":"http://127.0.0.1:9/v1","key_env":"HEXWITCH_PLAYGROUND_TEST_KEY"}),
    );
    assert_eq!(edit["ok"], false);
    assert_eq!(
        host.post("/api/providers/remove", json!({"id":"launch"}))["ok"],
        false
    );
}
#[test]
fn live_calls_report_progress_and_can_be_cancelled() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let hang = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_any_request(&mut stream);
        std::thread::sleep(std::time::Duration::from_secs(4));
    });
    let host = std::sync::Arc::new(Host::new(&[
        "--base-url",
        &base,
        "--api-key-env",
        "HEXWITCH_PLAYGROUND_TEST_KEY",
        "--model",
        "slow-model",
    ]));
    assert_eq!(host.get_json("/api/live")["active"], false);
    let caller = {
        let host = host.clone();
        std::thread::spawn(move || {
            host.post(
                "/api/prompt",
                json!({"revision":0,"mode":"live","request":"Darker comments","consent":true}),
            )
        })
    };
    let started = std::time::Instant::now();
    let status = loop {
        let status = host.get_json("/api/live");
        if status["active"] == true {
            break status;
        }
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    assert_eq!(status["phase"], "waiting");
    assert_eq!(status["model"], "slow-model");
    assert!(status["host"].as_str().unwrap().starts_with("127.0.0.1:"));
    let busy = host.post(
        "/api/prompt",
        json!({"revision":0,"mode":"live","request":"Again","consent":true}),
    );
    assert!(busy["error"].as_str().unwrap().contains("in progress"));
    let mut cancelled = false;
    while !cancelled {
        cancelled = host.post("/api/live/cancel", json!({}))["result"]["cancelled"] == true;
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }
    let result = caller.join().unwrap();
    assert_eq!(result["ok"], false);
    assert!(result["error"]
        .as_str()
        .unwrap()
        .contains("provider cancelled"));
    assert!(started.elapsed() < std::time::Duration::from_secs(3));
    assert_eq!(host.get_json("/api/live")["active"], false);
    assert_eq!(
        host.post("/api/live/cancel", json!({}))["result"]["cancelled"],
        false
    );
    let reference = host.post(
        "/api/reference",
        json!({"revision":0,"name":"omarchy-tokyo-night"}),
    );
    assert_eq!(reference["ok"], true, "{reference}");
    assert_eq!(
        reference["result"]["current"]["meta"]["name"],
        "Tokyo Night (Omarchy)"
    );
    let themes = host.bootstrap()["themes"]["groups"].clone();
    assert_eq!(themes[1]["name"], "Omarchy");
    assert!(themes[1]["themes"].as_array().unwrap().len() >= 23);
    hang.join().unwrap();
}
