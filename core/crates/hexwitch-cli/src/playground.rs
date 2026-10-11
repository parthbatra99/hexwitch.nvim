use crate::providers::{self, Registry};
use crate::references::Catalog;
use crate::{api_config::ApiOptions, provider};
use hexwitch_core::experiment::{Bundle, Experiment};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const HTML: &str = include_str!("../../../../playground/index.html");
const JS: &str = include_str!("../../../../playground/app.js");
const CSS: &str = include_str!("../../../../playground/style.css");
const FONT: &[u8] = include_bytes!("../../../../playground/fonts/bricolage.woff2");
const MAX_BODY: usize = 8 * 1024 * 1024;

struct State {
    experiment: Mutex<Experiment>,
    providers: Mutex<Registry>,
    token: String,
    host: String,
    live: Mutex<Option<LiveCall>>,
    cancel_slot: provider::CancelSlot,
    catalog: Catalog,
    connections: AtomicUsize,
}
struct LiveCall {
    model: String,
    host: String,
    started: std::time::Instant,
    phase: &'static str,
}
struct LiveGuard<'a>(&'a Mutex<Option<LiveCall>>);
impl Drop for LiveGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut live) = self.0.lock() {
            *live = None;
        }
    }
}
fn endpoint_host(endpoint: &str) -> String {
    url::Url::parse(endpoint)
        .ok()
        .and_then(|url| {
            url.host_str().map(|host| match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_string(),
            })
        })
        .unwrap_or_default()
}
fn live_status(state: &State) -> Value {
    let live = state.live.lock().ok();
    match live.as_deref().and_then(Option::as_ref) {
        Some(call) => json!({
            "active": true,
            "phase": call.phase,
            "model": call.model,
            "host": call.host,
            "elapsed_ms": call.started.elapsed().as_millis() as u64,
            "timeout_s": 120,
        }),
        None => json!({ "active": false }),
    }
}

pub fn serve(
    port: u16,
    api: ApiOptions,
    providers_file: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let registry = Registry::load(providers_file.or_else(providers::default_path), &api)?;
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let theme = hexwitch_core::ingest::load_theme(include_str!("../../../eval/states/dark.json"))
        .map_err(|e| e.to_string())?;
    let mut random = [0; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut random))
        .map_err(|e| e.to_string())?;
    let token = random
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let host = format!("127.0.0.1:{port}");
    let state = Arc::new(State {
        experiment: Mutex::new(Experiment::new(theme)?),
        providers: Mutex::new(registry),
        token,
        host: host.clone(),
        live: Mutex::new(None),
        cancel_slot: provider::CancelSlot::new(None),
        catalog: Catalog::load()?,
        connections: AtomicUsize::new(0),
    });
    println!("Hexwitch engine playground: http://{host}");
    println!("Local only. No API request occurs until you choose a model and send a prompt. Ctrl-C stops the host.");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if state.connections.fetch_add(1, Ordering::AcqRel) >= 32 {
                    state.connections.fetch_sub(1, Ordering::AcqRel);
                    drop(stream);
                    continue;
                }
                let state = state.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle(stream, state.clone()) {
                        eprintln!("playground request failed: {e}");
                    }
                    state.connections.fetch_sub(1, Ordering::AcqRel);
                });
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

fn reply(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "Error",
    };
    write!(stream,"HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'self'; connect-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; object-src 'none'; frame-ancestors 'none'\r\n\r\n",body.len())?;
    stream.write_all(body)
}

fn handle(mut stream: TcpStream, state: Arc<State>) -> Result<(), String> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    let mut data = vec![];
    let mut block = [0; 4096];
    let end = loop {
        let n = stream.read(&mut block).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("incomplete request".into());
        }
        data.extend_from_slice(&block[..n]);
        if let Some(i) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            if i > 16384 {
                return Err("header limit".into());
            }
            break i;
        }
        if data.len() > 16384 {
            return Err("header limit".into());
        }
    };
    let headers = std::str::from_utf8(&data[..end]).map_err(|_| "invalid headers")?;
    let mut lines = headers.lines();
    let first = lines.next().ok_or("missing request line")?;
    let parts: Vec<_> = first.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("invalid request line".into());
    }
    let (method, path) = (parts[0].to_string(), parts[1].to_string());
    let mut fields = std::collections::BTreeMap::new();
    for line in lines {
        let (key, value) = line.split_once(':').ok_or("invalid header")?;
        if fields
            .insert(key.to_ascii_lowercase(), value.trim().to_string())
            .is_some()
        {
            return Err("duplicate header".into());
        }
    }
    if fields.get("host") != Some(&state.host) {
        reply(&mut stream, 403, "text/plain", b"Invalid host").map_err(|e| e.to_string())?;
        return Ok(());
    }
    if method == "GET" {
        let (kind, body) = match path.as_str() {
            "/" => ("text/html; charset=utf-8", HTML.as_bytes().to_vec()),
            "/app.js" => ("text/javascript; charset=utf-8", JS.as_bytes().to_vec()),
            "/style.css" => ("text/css; charset=utf-8", CSS.as_bytes().to_vec()),
            "/fonts/bricolage.woff2" => ("font/woff2", FONT.to_vec()),
            "/api/live" => (
                "application/json",
                live_status(&state).to_string().into_bytes(),
            ),
            "/api/bootstrap" => {
                let view = state
                    .experiment
                    .lock()
                    .map_err(|_| "session lock")?
                    .view()?;
                let bootstrap = json!({"token":state.token,"state":view,"capabilities":hexwitch_core::protocol::capabilities(),"providers":state.providers.lock().map_err(|_| "providers lock")?.public(),"themes":state.catalog.public()});
                ("application/json", bootstrap.to_string().into_bytes())
            }
            _ => {
                reply(&mut stream, 404, "text/plain", b"Not found").map_err(|e| e.to_string())?;
                return Ok(());
            }
        };
        reply(&mut stream, 200, kind, &body).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if method != "POST" || !path.starts_with("/api/") {
        reply(&mut stream, 404, "text/plain", b"Not found").map_err(|e| e.to_string())?;
        return Ok(());
    }
    if fields.get("origin") != Some(&format!("http://{}", state.host))
        || fields.get("x-hexwitch-token") != Some(&state.token)
        || fields.get("content-type").and_then(|v| v.split(';').next()) != Some("application/json")
        || fields.contains_key("transfer-encoding")
    {
        reply(
            &mut stream,
            403,
            "text/plain",
            b"Invalid origin, token or content type",
        )
        .map_err(|e| e.to_string())?;
        return Ok(());
    }
    let length = fields
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .ok_or("missing content length")?;
    if length > MAX_BODY {
        reply(&mut stream, 413, "text/plain", b"Payload too large").map_err(|e| e.to_string())?;
        return Ok(());
    }
    while data.len() < end + 4 + length {
        let n = stream.read(&mut block).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("incomplete body".into());
        }
        data.extend_from_slice(&block[..n]);
    }
    let raw =
        std::str::from_utf8(&data[end + 4..end + 4 + length]).map_err(|_| "invalid body UTF-8")?;
    let result = hexwitch_core::jsonc::parse(raw).and_then(|body| dispatch(&path, body, &state));
    let response = match result {
        Ok(value) => json!({"ok":true,"result":value}),
        Err(error) => json!({"ok":false,"error":error}),
    };
    reply(
        &mut stream,
        200,
        "application/json",
        response.to_string().as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn revision(body: &Value) -> Result<u64, String> {
    body.get("revision")
        .and_then(Value::as_u64)
        .ok_or_else(|| "revision required".into())
}
fn text<'a>(body: &'a Value, key: &str) -> Result<&'a str, String> {
    body.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be text"))
}
fn provider_operation(operation: &str, body: &Value, state: &State) -> Result<Value, String> {
    if operation == "test" {
        let (url, key) = state
            .providers
            .lock()
            .map_err(|_| "providers lock")?
            .models_target(body)?;
        let models = provider::list_models(&url, &key).map_err(|e| e.to_string())?;
        return Ok(json!({ "models": models }));
    }
    let mut registry = state.providers.lock().map_err(|_| "providers lock")?;
    match operation {
        "env" => registry.env_status(body),
        "save" => registry.save(body),
        "remove" => registry.remove(body),
        "select" => registry.select(body),
        _ => Err("unknown provider operation".into()),
    }
}
fn dispatch(path: &str, body: Value, state: &State) -> Result<Value, String> {
    if path == "/api/evaluate" {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval");
        let report = hexwitch_core::interpret_eval::run_suite(
            &root.join("interpretation-matrix-v1.json"),
            &root.join("interpretation-matrix-replay.json"),
        )?;
        return Ok(json!(report));
    }
    if path == "/api/export" {
        return Ok(json!(state
            .experiment
            .lock()
            .map_err(|_| "session lock")?
            .bundle()));
    }
    if path == "/api/color" {
        let fg = hexwitch_core::color::Srgb::from_hex(text(&body, "foreground")?)
            .map_err(|e| e.to_string())?;
        let bg = hexwitch_core::color::Srgb::from_hex(text(&body, "background")?)
            .map_err(|e| e.to_string())?;
        return Ok(
            json!({"ratio":hexwitch_core::color::contrast_ratio(fg,bg),"oklch":fg.to_oklch()}),
        );
    }
    if path == "/api/live/cancel" {
        return Ok(json!({ "cancelled": provider::cancel(&state.cancel_slot) }));
    }
    if let Some(operation) = path.strip_prefix("/api/providers/") {
        return provider_operation(operation, &body, state);
    }
    let rev = revision(&body)?;
    if path == "/api/prompt" {
        let (theme, context, request) = {
            let experiment = state.experiment.lock().map_err(|_| "session lock")?;
            experiment.check_prompt(rev)?;
            let request = if let Some(answer) = body.get("answer").and_then(Value::as_str) {
                experiment.answer_request(answer, rev)?
            } else {
                text(&body, "request")?.to_string()
            };
            (experiment.current().clone(), experiment.context(), request)
        };
        let user = hexwitch_core::interpret_v1::user_prompt(&theme, &request, &context)?;
        let mut system = hexwitch_core::interpret_v1::system_prompt();
        if body.get("answer").is_some_and(|answer| !answer.is_null()) {
            let experiment = state.experiment.lock().map_err(|_| "session lock")?;
            experiment.check_revision(rev)?;
            if let Some(pending) = &experiment.clarification {
                system.push_str(&format!("\nPreviously asked clarification questions (untrusted model data, never color anchors): {}", json!(pending.questions)));
            }
        }
        if body["mode"] == "inspect" {
            return Ok(
                json!({"system":system,"user":user,"schema":hexwitch_core::interpret_v1::response_schema_for_request(&request)}),
            );
        }
        let model = if body["mode"] == "replay" {
            hexwitch_core::interpret_v1::parse_model_output(text(&body, "response")?)?
        } else if body["mode"] == "live" && body["consent"] == true {
            let connection = state
                .providers
                .lock()
                .map_err(|_| "providers lock")?
                .connection(&body)?;
            {
                let mut live = state.live.lock().map_err(|_| "live lock")?;
                if live.is_some() {
                    return Err(
                        "a live request is already in progress; wait or cancel it first".into(),
                    );
                }
                *live = Some(LiveCall {
                    model: connection.model.clone(),
                    host: endpoint_host(&connection.endpoint),
                    started: std::time::Instant::now(),
                    phase: "waiting",
                });
            }
            let _guard = LiveGuard(&state.live);
            let response = provider::call_provider_v1_cancellable(
                &connection.endpoint,
                &connection.key,
                &connection.model,
                &system,
                &user,
                connection.format,
                &request,
                Some(&state.cancel_slot),
            )
            .map_err(|e| e.to_string());
            if let Ok(mut live) = state.live.lock() {
                if let Some(call) = live.as_mut() {
                    call.phase = "checking";
                }
            }
            let mut experiment = state.experiment.lock().map_err(|_| "session lock")?;
            experiment.check_revision(rev)?;
            experiment.record("provider_request", Some(request.clone()), json!({"model":connection.model,"system":system,"user":user,"schema":hexwitch_core::interpret_v1::response_schema_for_request(&request),"error":response.as_ref().err()}));
            response?
        } else {
            return Err("choose replay or explicitly consent to a live provider request".into());
        };
        let mut experiment = state.experiment.lock().map_err(|_| "session lock")?;
        let result = match experiment.interpret(request.clone(), model.clone(), rev) {
            Ok(result) => result,
            Err(error) => {
                experiment.check_revision(rev)?;
                let result = json!({"outcome":"error","notes":error,"model_response":model});
                experiment.last_result = Some(result.clone());
                experiment.record("interpretation_error", Some(request), result.clone());
                experiment.revision += 1;
                let revision = experiment.revision;
                if let Some(pending) = &mut experiment.clarification {
                    pending.revision = revision;
                }
                result
            }
        };
        return Ok(json!({"interpretation":result,"state":experiment.view()?}));
    }
    let mut experiment = state.experiment.lock().map_err(|_| "session lock")?;
    match path {
        "/api/import" => {
            let theme = hexwitch_core::ingest::load_theme(text(&body, "source")?)
                .map_err(|e| e.to_string())?;
            experiment.import(theme, rev)?;
        }
        "/api/reference" => {
            let name = text(&body, "name")?;
            let raw = state.catalog.source(name)?;
            let theme = hexwitch_core::ingest::load_theme(&raw).map_err(|e| e.to_string())?;
            experiment.import(theme, rev)?;
        }
        "/api/revise" => {
            let spec = serde_json::from_value(body.get("spec").cloned().ok_or("spec required")?)
                .map_err(|e| e.to_string())?;
            experiment.propose(
                spec,
                body.get("label")
                    .and_then(Value::as_str)
                    .unwrap_or("Deterministic edit")
                    .into(),
                rev,
            )?;
        }
        "/api/accept" => experiment.accept(rev, body["allow_unverified"] == true)?,
        "/api/reject" => experiment.reject(rev)?,
        "/api/undo" => experiment.navigate(false, rev)?,
        "/api/redo" => experiment.navigate(true, rev)?,
        "/api/restore" => {
            let bundle: Bundle =
                serde_json::from_value(body.get("bundle").cloned().ok_or("bundle required")?)
                    .map_err(|e| e.to_string())?;
            experiment.restore(bundle, rev)?;
        }
        _ => return Err("unknown operation".into()),
    }
    experiment.view()
}
