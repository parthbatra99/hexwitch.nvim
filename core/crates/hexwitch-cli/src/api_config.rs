//! One OpenAI-compatible Chat Completions connection, not provider-specific SDKs.
use crate::provider::OutputFormat;
use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct ApiOptions {
    /// API root, e.g. https://api.openai.com/v1. Appends /chat/completions.
    #[arg(long, alias = "baseurl")]
    pub base_url: Option<String>,
    /// Name of the environment variable containing the key; never the key itself.
    #[arg(long)]
    pub api_key_env: Option<String>,
    /// Provider-specific model identifier on the compatible API.
    #[arg(short, long)]
    pub model: Option<String>,
    /// json_schema, json_object, or none. Validation always stays strict locally.
    #[arg(long,value_enum,default_value_t=OutputFormat::JsonSchema)]
    pub response_format: OutputFormat,
    /// Legacy convenience alias; prefer explicit base URL and key environment.
    #[arg(long)]
    pub provider: Option<String>,
}

pub struct Connection {
    pub endpoint: String,
    pub key: String,
    pub model: String,
    pub format: OutputFormat,
}

pub fn endpoint(base_url: &str) -> Result<String, String> {
    if base_url.trim() != base_url || base_url.chars().any(char::is_control) {
        return Err("base URL contains whitespace or control characters".into());
    }
    let mut url = url::Url::parse(base_url)
        .map_err(|_| "base URL must be an absolute HTTP(S) URL".to_string())?;
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("base URL cannot include credentials, query or fragment".into());
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Err("base URL must use HTTP or HTTPS".into());
    }
    let path = url.path().trim_end_matches('/');
    if path.ends_with("/chat/completions") {
        return Err("base URL must be the API root, not the /chat/completions endpoint".into());
    }
    url.set_path(&format!("{path}/chat/completions"));
    Ok(url.to_string())
}

pub fn key_name_valid(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl ApiOptions {
    pub fn resolve(&self) -> Result<Connection, String> {
        let (default_base,default_key,default_model)=match self.provider.as_deref(){
            None|Some("openai")=>("https://api.openai.com/v1","OPENAI_API_KEY","gpt-4o-mini"),
            Some("openrouter")=>("https://openrouter.ai/api/v1","OPENROUTER_API_KEY","openai/gpt-4o-mini"),
            Some(_)=>return Err("legacy provider alias must be openai or openrouter; use --base-url for any compatible provider".into()),
        };
        let environment_base = std::env::var("OPENAI_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty());
        let endpoint = endpoint(
            self.base_url
                .as_deref()
                .or(environment_base.as_deref())
                .unwrap_or(default_base),
        )?;
        let key_name = self.api_key_env.as_deref().unwrap_or(default_key);
        if !key_name_valid(key_name) {
            return Err(
                "api-key-env must be an environment variable name, not a credential".into(),
            );
        }
        let key = std::env::var(key_name)
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("{key_name} is not set in the engine environment"))?;
        if key.chars().any(char::is_control) {
            return Err("API key contains invalid control characters".into());
        }
        let model = self.model.as_deref().unwrap_or(default_model);
        if model.trim().is_empty() || model.len() > 256 || model.chars().any(char::is_control) {
            return Err("model must be a nonempty identifier without control characters".into());
        }
        Ok(Connection {
            endpoint,
            key,
            model: model.into(),
            format: self.response_format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_base_paths_and_rejects_unsafe_targets() {
        assert_eq!(
            endpoint("https://api.openai.com/v1/").unwrap(),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint("https://openrouter.ai/api/v1").unwrap(),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert!(endpoint("http://127.0.0.1:8000/v1").is_ok());
        assert!(endpoint("http://[::1]:8000/v1").is_ok());
        assert_eq!(
            endpoint("http://gateway.example:8000/v1").unwrap(),
            "http://gateway.example:8000/v1/chat/completions"
        );
        for invalid in [
            "https://user:secret@example.com/v1",
            "https://example.com/v1?api_key=secret",
            "https://example.com/v1#x",
            "https://example.com/v1/chat/completions",
            "file:///etc/passwd",
        ] {
            assert!(endpoint(invalid).is_err(), "{invalid}");
        }
    }
}
