use crate::api_config::{self, ApiOptions, Connection};
use crate::provider::OutputFormat;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const KINDS: [&str; 4] = ["openai", "openrouter", "local", "custom"];
const FORMATS: [&str; 3] = ["json_schema", "json_object", "none"];
const LAUNCH_ID: &str = "launch";
const MAX_PROVIDERS: usize = 24;
const MAX_MODELS: usize = 64;

#[derive(Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub base_url: String,
    #[serde(default)]
    pub key_env: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default = "default_format")]
    pub response_format: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Selection {
    pub provider: String,
    pub model: String,
}

#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    #[serde(default)]
    selected: Option<Selection>,
    #[serde(default)]
    providers: Vec<Provider>,
}

pub struct Registry {
    path: Option<PathBuf>,
    launch: Option<Provider>,
    providers: Vec<Provider>,
    selected: Option<Selection>,
    session_keys: HashMap<String, String>,
}

fn default_format() -> String {
    "json_schema".into()
}

fn output_format(name: &str) -> OutputFormat {
    match name {
        "json_object" => OutputFormat::JsonObject,
        "none" => OutputFormat::None,
        _ => OutputFormat::JsonSchema,
    }
}

fn identifier_valid(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}

fn validate_model(model: &str) -> Result<(), String> {
    if identifier_valid(model, 256) {
        Ok(())
    } else {
        Err("model must be a nonempty identifier without control characters".into())
    }
}

fn validate(provider: &Provider) -> Result<(), String> {
    if !identifier_valid(&provider.name, 80) {
        return Err("provider name must be 1..80 characters".into());
    }
    if !KINDS.contains(&provider.kind.as_str()) {
        return Err("provider kind must be openai, openrouter, local or custom".into());
    }
    if !FORMATS.contains(&provider.response_format.as_str()) {
        return Err("response format must be json_schema, json_object or none".into());
    }
    api_config::endpoint(&provider.base_url)?;
    if let Some(name) = &provider.key_env {
        if !api_config::key_name_valid(name) {
            return Err("key variable must be an environment variable name, not a key".into());
        }
    }
    if provider.models.len() > MAX_MODELS {
        return Err(format!("keep at most {MAX_MODELS} models per provider"));
    }
    for model in provider.models.iter().chain(provider.default_model.iter()) {
        validate_model(model)?;
    }
    Ok(())
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_ascii_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out
        .trim_end_matches('-')
        .chars()
        .take(32)
        .collect::<String>();
    if out.is_empty() || out == LAUNCH_ID {
        "provider".into()
    } else {
        out
    }
}

fn host_of(base_url: &str) -> String {
    let Ok(url) = url::Url::parse(base_url) else {
        return String::new();
    };
    match (url.host_str(), url.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_string(),
        _ => String::new(),
    }
}

fn env_present(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| !value.is_empty())
}

pub fn default_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("HEXWITCH_PROVIDERS_FILE").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path));
    }
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config.join("hexwitch").join("providers.json"))
}

fn launch_provider(api: &ApiOptions) -> Result<Option<Provider>, String> {
    if !api.explicit() {
        return Ok(None);
    }
    let config = api.public_config()?;
    let base_url = config["base_url"].as_str().unwrap_or_default().to_string();
    let model = config["model"].as_str().unwrap_or_default().to_string();
    let provider = Provider {
        id: LAUNCH_ID.into(),
        name: host_of(&base_url),
        kind: "custom".into(),
        key_env: config["api_key_env"].as_str().map(str::to_string),
        models: vec![model.clone()],
        default_model: Some(model),
        response_format: config["response_format"]
            .as_str()
            .unwrap_or("json_schema")
            .to_string(),
        base_url,
    };
    validate(&provider)?;
    Ok(Some(provider))
}

fn read_stored(path: &Path) -> Result<Option<Stored>, String> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    let stored: Stored = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "{} is not a valid providers file ({error}); fix or delete it",
            path.display()
        )
    })?;
    if stored.version != 1 {
        return Err(format!("{} has an unsupported version", path.display()));
    }
    for provider in &stored.providers {
        validate(provider).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(Some(stored))
}

fn write_private(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    std::fs::rename(&temporary, path).map_err(|e| e.to_string())
}

fn text<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    body.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn session_key(body: &Value) -> Result<Option<String>, String> {
    match body.get("key").and_then(Value::as_str).map(str::trim) {
        None | Some("") => Ok(None),
        Some(key) if key.len() > 4096 || key.chars().any(char::is_control) => {
            Err("pasted key contains invalid characters".into())
        }
        Some(key) => Ok(Some(key.to_string())),
    }
}

fn draft(body: &Value) -> Result<Provider, String> {
    let kind = text(body, "kind").unwrap_or("custom").to_string();
    let base_url = text(body, "base_url")
        .ok_or("API root is required")?
        .to_string();
    let models = body
        .get("models")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut unique = Vec::new();
    for model in models {
        if !unique.contains(&model) {
            unique.push(model);
        }
    }
    let default_model = text(body, "default_model")
        .map(str::to_string)
        .or_else(|| unique.first().cloned());
    let provider = Provider {
        id: text(body, "id").unwrap_or_default().to_string(),
        name: text(body, "name")
            .map(str::to_string)
            .unwrap_or_else(|| host_of(&base_url)),
        kind,
        key_env: text(body, "key_env").map(str::to_string),
        models: unique,
        default_model,
        response_format: text(body, "response_format")
            .unwrap_or("json_schema")
            .to_string(),
        base_url,
    };
    validate(&provider)?;
    Ok(provider)
}

impl Registry {
    pub fn load(path: Option<PathBuf>, api: &ApiOptions) -> Result<Self, String> {
        let launch = launch_provider(api)?;
        let stored = match &path {
            Some(path) => read_stored(path)?,
            None => None,
        };
        let (providers, selected) = stored
            .map(|stored| (stored.providers, stored.selected))
            .unwrap_or_default();
        let mut registry = Self {
            path,
            launch,
            providers,
            selected,
            session_keys: HashMap::new(),
        };
        if let Some(launch) = &registry.launch {
            registry.selected = Some(Selection {
                provider: LAUNCH_ID.into(),
                model: launch.default_model.clone().unwrap_or_default(),
            });
        }
        if registry
            .selected
            .as_ref()
            .is_some_and(|s| registry.find(&s.provider).is_none())
        {
            registry.selected = None;
        }
        Ok(registry)
    }

    fn all(&self) -> impl Iterator<Item = &Provider> {
        self.launch.iter().chain(self.providers.iter())
    }

    fn find(&self, id: &str) -> Option<&Provider> {
        self.all().find(|provider| provider.id == id)
    }

    fn key_ready(&self, provider: &Provider) -> bool {
        match &provider.key_env {
            Some(name) => env_present(name),
            None => provider.kind == "local" || self.session_keys.contains_key(&provider.id),
        }
    }

    fn key_source(&self, provider: &Provider) -> &'static str {
        match (&provider.key_env, provider.kind.as_str()) {
            (Some(_), _) => "env",
            (None, "local") if !self.session_keys.contains_key(&provider.id) => "none",
            _ => "session",
        }
    }

    pub fn public(&self) -> Value {
        let providers = self
            .all()
            .map(|p| {
                json!({
                    "id": p.id,
                    "name": p.name,
                    "kind": p.kind,
                    "base_url": p.base_url,
                    "host": host_of(&p.base_url),
                    "key_env": p.key_env,
                    "key_source": self.key_source(p),
                    "key_ready": self.key_ready(p),
                    "models": p.models,
                    "default_model": p.default_model,
                    "response_format": p.response_format,
                    "launch": p.id == LAUNCH_ID,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "providers": providers,
            "selected": self.selected,
            "path": self.path.as_ref().map(|p| p.display().to_string()),
        })
    }

    fn key_for(&self, provider: &Provider, pasted: Option<String>) -> Result<String, String> {
        if let Some(key) = pasted {
            return Ok(key);
        }
        match &provider.key_env {
            Some(name) => std::env::var(name)
                .ok()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| format!("{name} is not set in the engine environment")),
            None => match self.session_keys.get(&provider.id) {
                Some(key) => Ok(key.clone()),
                None if provider.kind == "local" => Ok(String::new()),
                None => Err(format!(
                    "{} needs its key pasted again; pasted keys are forgotten when the engine stops",
                    provider.name
                )),
            },
        }
    }

    pub fn env_status(&self, body: &Value) -> Result<Value, String> {
        let name = text(body, "name").ok_or("name required")?;
        if !api_config::key_name_valid(name) {
            return Err("that is not an environment variable name".into());
        }
        Ok(json!({ "name": name, "present": env_present(name) }))
    }

    pub fn models_target(&self, body: &Value) -> Result<(String, String), String> {
        let provider = match text(body, "id") {
            Some(id) => self.find(id).ok_or("unknown provider")?.clone(),
            None => draft(body)?,
        };
        let key = self.key_for(&provider, session_key(body)?)?;
        Ok((api_config::models_endpoint(&provider.base_url)?, key))
    }

    pub fn save(&mut self, body: &Value) -> Result<Value, String> {
        let mut provider = draft(body)?;
        if provider.id == LAUNCH_ID {
            return Err(
                "the launch provider comes from command-line flags and cannot be edited".into(),
            );
        }
        let pasted = session_key(body)?;
        if pasted.is_some() {
            provider.key_env = None;
        }
        if provider.key_env.is_none()
            && provider.kind != "local"
            && pasted.is_none()
            && !self.session_keys.contains_key(&provider.id)
        {
            return Err("choose a key variable or paste a key".into());
        }
        if provider.id.is_empty() || self.find(&provider.id).is_none() {
            if self.providers.len() >= MAX_PROVIDERS {
                return Err(format!("keep at most {MAX_PROVIDERS} providers"));
            }
            let base = slug(&provider.name);
            let mut id = base.clone();
            let mut n = 2;
            while self.find(&id).is_some() {
                id = format!("{base}-{n}");
                n += 1;
            }
            provider.id = id;
            self.providers.push(provider.clone());
        } else if let Some(existing) = self.providers.iter_mut().find(|p| p.id == provider.id) {
            *existing = provider.clone();
        }
        match pasted {
            Some(key) => {
                self.session_keys.insert(provider.id.clone(), key);
            }
            None if provider.key_env.is_some() => {
                self.session_keys.remove(&provider.id);
            }
            None => {}
        }
        let keep_selection = self
            .selected
            .as_ref()
            .is_some_and(|s| s.provider != provider.id || provider.models.contains(&s.model));
        if !keep_selection {
            self.selected = provider.default_model.clone().map(|model| Selection {
                provider: provider.id.clone(),
                model,
            });
        }
        self.persist()?;
        Ok(json!({ "id": provider.id, "providers": self.public() }))
    }

    pub fn remove(&mut self, body: &Value) -> Result<Value, String> {
        let id = text(body, "id").ok_or("id required")?;
        if id == LAUNCH_ID {
            return Err("the launch provider comes from command-line flags".into());
        }
        let before = self.providers.len();
        self.providers.retain(|p| p.id != id);
        if self.providers.len() == before {
            return Err("unknown provider".into());
        }
        self.session_keys.remove(id);
        if self.selected.as_ref().is_some_and(|s| s.provider == id) {
            self.selected = None;
        }
        self.persist()?;
        Ok(self.public())
    }

    pub fn select(&mut self, body: &Value) -> Result<Value, String> {
        match text(body, "provider") {
            None => self.selected = None,
            Some(id) => {
                self.find(id).ok_or("unknown provider")?;
                let model = text(body, "model").ok_or("model required")?;
                validate_model(model)?;
                self.selected = Some(Selection {
                    provider: id.into(),
                    model: model.into(),
                });
            }
        }
        self.persist()?;
        Ok(self.public())
    }

    pub fn connection(&self, body: &Value) -> Result<Connection, String> {
        let requested = text(body, "provider");
        let id = requested
            .or(self.selected.as_ref().map(|s| s.provider.as_str()))
            .ok_or("connect a model first, or switch to offline")?;
        let provider = self.find(id).ok_or("unknown provider")?;
        let selected_model = self
            .selected
            .as_ref()
            .filter(|s| s.provider == provider.id)
            .map(|s| s.model.clone());
        let model = text(body, "model")
            .map(str::to_string)
            .or(selected_model)
            .or_else(|| provider.default_model.clone())
            .ok_or("choose a model for this provider")?;
        validate_model(&model)?;
        Ok(Connection {
            endpoint: api_config::endpoint(&provider.base_url)?,
            key: self.key_for(provider, None)?,
            model,
            format: output_format(&provider.response_format),
        })
    }

    fn persist(&self) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let selected = self.selected.clone().filter(|s| s.provider != LAUNCH_ID);
        let stored = Stored {
            version: 1,
            selected,
            providers: self.providers.clone(),
        };
        let contents = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
        write_private(path, &contents)
            .map_err(|error| format!("cannot save {}: {error}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_safe_and_never_collide_with_launch() {
        assert_eq!(slug("OpenRouter"), "openrouter");
        assert_eq!(slug("  Team gateway / prod "), "team-gateway-prod");
        assert_eq!(slug("launch"), "provider");
        assert_eq!(slug("!!!"), "provider");
    }

    #[test]
    fn drafts_reject_keys_in_variable_names_and_bad_roots() {
        assert!(draft(
            &json!({"kind":"openai","base_url":"https://api.openai.com/v1","key_env":"sk-live-abc"})
        )
        .is_err());
        assert!(draft(
            &json!({"kind":"openai","base_url":"https://api.openai.com/v1/chat/completions"})
        )
        .is_err());
        assert!(draft(&json!({"kind":"weird","base_url":"https://api.openai.com/v1"})).is_err());
        let ok = draft(
            &json!({"kind":"openai","base_url":"https://api.openai.com/v1","models":["b","a","b"]}),
        )
        .unwrap();
        assert_eq!(ok.models, vec!["b", "a"]);
        assert_eq!(ok.default_model.as_deref(), Some("b"));
        assert_eq!(ok.name, "api.openai.com");
    }
}
