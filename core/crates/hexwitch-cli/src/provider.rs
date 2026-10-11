//! Curl transport: credentials and request data only pass over stdin, never disk/argv.
use hexwitch_core::interpret::{self, LlmSpec};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
#[value(rename_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    JsonSchema,
    JsonObject,
    None,
}

fn payload(
    model: &str,
    system: &str,
    user: &str,
    schema: serde_json::Value,
    format: OutputFormat,
) -> serde_json::Value {
    let system = if matches!(format, OutputFormat::JsonSchema) {
        system.to_string()
    } else {
        format!("{system}\nReturn a JSON object matching this exact schema (required nullable fields must be present): {}",schema["schema"])
    };
    let mut payload = serde_json::json!({"model":model,"messages":[{"role":"system","content":system},{"role":"user","content":user}]});
    match format {
        OutputFormat::JsonSchema => {
            payload["response_format"] =
                serde_json::json!({"type":"json_schema","json_schema":schema})
        }
        OutputFormat::JsonObject => {
            payload["response_format"] = serde_json::json!({"type":"json_object"})
        }
        OutputFormat::None => {}
    }
    payload
}

const MAX_PAYLOAD: usize = 1024 * 1024;
const MAX_RESPONSE: usize = 2 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
#[error("provider {code}: {message}")]
pub struct ProviderError {
    pub code: &'static str,
    pub message: &'static str,
}
fn error(code: &'static str, message: &'static str) -> ProviderError {
    ProviderError { code, message }
}

fn quote(value: &str) -> Result<String, ProviderError> {
    if value
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
    {
        return Err(error("invalid_input", "unsupported control character"));
    }
    Ok(format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    ))
}

pub fn call_provider(
    endpoint: &str,
    key: &str,
    model: &str,
    system: &str,
    user: &str,
    format: OutputFormat,
) -> Result<LlmSpec, ProviderError> {
    if key.is_empty() || key.chars().any(char::is_control) {
        return Err(error("invalid_input", "invalid credential"));
    }
    let payload = payload(model, system, user, interpret::response_schema(), format);
    let body = transport(endpoint, key, &payload.to_string(), 120)?;
    decode_response(&body)
}

fn transport(
    endpoint: &str,
    key: &str,
    payload: &str,
    timeout: u32,
) -> Result<String, ProviderError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(error("payload_too_large", "request exceeds 1 MiB"));
    }
    let parsed =
        url::Url::parse(endpoint).map_err(|_| error("invalid_input", "invalid endpoint URL"))?;
    if parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(parsed.scheme(), "http" | "https")
    {
        return Err(error(
            "invalid_input",
            "endpoint must use HTTP(S) without credentials, query or fragment",
        ));
    }
    if parsed.scheme() == "http" {
        eprintln!("warning: HTTP API connection is unencrypted; credentials and prompts can be read or modified in transit");
    }
    let config = format!("url = {}\nrequest = \"POST\"\nheader = {}\nheader = \"Content-Type: application/json\"\ndata-binary = {}\n",quote(endpoint)?,quote(&format!("Authorization: Bearer {key}"))?,quote(payload)?);
    let mut child = Command::new("curl")
        .args([
            "-q",
            "--silent",
            "--show-error",
            "--config",
            "-",
            "--connect-timeout",
            "10",
            "--max-time",
            &timeout.to_string(),
            "--max-filesize",
            &MAX_RESPONSE.to_string(),
            "--write-out",
            "\n%{http_code}",
            "--proto",
            if parsed.scheme() == "http" {
                "=http"
            } else {
                "=https"
            },
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| error("unavailable", "cannot start curl"))?;
    let written = child.stdin.take().unwrap().write_all(config.as_bytes());
    if written.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error("network", "request stream failed"));
    }
    // Bound memory independently of curl version/content-length handling.
    use std::io::Read;
    let mut stdout = child.stdout.take().unwrap();
    let mut bytes = Vec::new();
    let read = stdout
        .by_ref()
        .take(MAX_RESPONSE as u64 + 9)
        .read_to_end(&mut bytes);
    if read.is_err() || bytes.len() > MAX_RESPONSE + 8 {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error(
            "payload_too_large",
            "response exceeds 2 MiB or stream failed",
        ));
    }
    drop(stdout);
    let status = child
        .wait()
        .map_err(|_| error("network", "cannot wait for HTTP process"))?;
    if !status.success() {
        return Err(match status.code() {
            Some(28) => error("timeout", "request timed out"),
            Some(63) => error("payload_too_large", "response exceeds limit"),
            _ => error("network", "HTTP transport failed"),
        });
    }
    let output =
        String::from_utf8(bytes).map_err(|_| error("invalid_response", "response is not UTF-8"))?;
    let (body, status) = output
        .rsplit_once('\n')
        .ok_or_else(|| error("invalid_response", "missing HTTP status"))?;
    match status.parse::<u16>().unwrap_or(0) {
        200..=299 => Ok(body.to_string()),
        429 => Err(error(
            "rate_limited",
            "provider rate limit; retry explicitly later",
        )),
        401 | 403 => Err(error("authentication", "provider rejected credentials")),
        _ => Err(error("http", "provider returned unsuccessful HTTP status")),
    }
}

pub fn call_provider_v1(
    endpoint: &str,
    key: &str,
    model: &str,
    system: &str,
    user: &str,
    format: OutputFormat,
    request: &str,
) -> Result<hexwitch_core::interpret_v1::ModelInterpretation, ProviderError> {
    if key.is_empty() || key.chars().any(char::is_control) {
        return Err(error("invalid_input", "invalid credential"));
    }
    let payload = payload(
        model,
        system,
        user,
        hexwitch_core::interpret_v1::response_schema_for_request(request),
        format,
    );
    let body = transport(endpoint, key, &payload.to_string(), 120)?;
    let content = decode_content(&body)?;
    hexwitch_core::interpret_v1::parse_model_output(&content)
        .map_err(|_| error("invalid_response", "model output failed v1 schema parsing"))
}

fn decode_content(body: &str) -> Result<String, ProviderError> {
    let value = hexwitch_core::jsonc::parse(body)
        .map_err(|_| error("invalid_response", "invalid or duplicate-key JSON response"))?;
    if value
        .pointer("/choices/0/message/refusal")
        .is_some_and(|v| !v.is_null())
    {
        return Err(error("refusal", "provider refused request"));
    }
    match value
        .pointer("/choices/0/finish_reason")
        .and_then(|v| v.as_str())
    {
        Some("length") => return Err(error("truncated", "provider response was truncated")),
        Some("content_filter") => {
            return Err(error("refusal", "provider content filter refused request"))
        }
        Some("stop") => {}
        _ => {
            return Err(error(
                "invalid_response",
                "unsupported or missing completion status",
            ))
        }
    }
    let content = value
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| error("invalid_response", "missing message content"))?;
    Ok(content.to_string())
}

pub fn decode_response(body: &str) -> Result<LlmSpec, ProviderError> {
    let content = decode_content(body)?;
    let spec = interpret::parse_model_output(&content)
        .map_err(|_| error("invalid_response", "model output failed schema parsing"))?;
    interpret::llm_spec_to_edit_spec(&spec).map_err(|_| {
        error(
            "invalid_response",
            "model output failed semantic validation",
        )
    })?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn refuses_and_truncation_never_succeed() {
        for (reason, code) in [("length", "truncated"), ("content_filter", "refusal")] {
            let body = serde_json::json!({"choices":[{"finish_reason":reason,"message":{"content":"{}"}}]}).to_string();
            assert_eq!(decode_response(&body).unwrap_err().code, code);
        }
        assert!(!decode_response("a credential echoed by a server")
            .unwrap_err()
            .to_string()
            .contains("credential"));
    }
    #[test]
    fn v1_response_decode_preserves_clarification_and_checks_completion_status() {
        let model = include_str!("../../../eval/llm/clarify-blue.json");
        let body =
            serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":model}}]})
                .to_string();
        let content = decode_content(&body).unwrap();
        assert_eq!(
            hexwitch_core::interpret_v1::parse_model_output(&content)
                .unwrap()
                .decision,
            hexwitch_core::interpret_v1::Decision::NeedsClarification
        );
        let refusal=serde_json::json!({"choices":[{"finish_reason":"stop","message":{"refusal":"declined","content":model}}]}).to_string();
        assert_eq!(decode_content(&refusal).unwrap_err().code, "refusal");
        let duplicate = r#"{"choices":[],"choices":[]}"#;
        assert!(decode_content(duplicate).is_err());
    }
    #[test]
    fn real_transport_mock_checks_status_and_stream() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut request = Vec::new();
            let mut bytes = [0; 4096];
            loop {
                let n = stream.read(&mut bytes).unwrap();
                request.extend_from_slice(&bytes[..n]);
                if String::from_utf8_lossy(&request).contains("{\"test\":true}") {
                    break;
                }
            }
            assert!(String::from_utf8_lossy(&request).contains("Authorization: Bearer mock-secret"));
            stream.write_all(b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        let error = transport(
            &format!("http://{address}"),
            "mock-secret",
            r#"{"test":true}"#,
            5,
        )
        .unwrap_err();
        assert_eq!(error.code, "rate_limited");
        assert!(!error.to_string().contains("mock-secret"));
        worker.join().unwrap();
    }
}
