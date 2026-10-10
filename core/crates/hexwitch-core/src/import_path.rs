//! Sandboxed include resolution; no remote URLs or parent/symlink escapes.
use crate::ingest::{load_resolved_theme, IngestError};
use crate::ir::Theme;
use serde_json::Value;
use std::path::Path;

pub fn load_theme_path(path: &Path) -> Result<Theme, IngestError> {
    let canonical = path
        .canonicalize()
        .map_err(|e| IngestError::InvalidSource(e.to_string()))?;
    let root = canonical
        .parent()
        .ok_or_else(|| IngestError::InvalidSource("source has no parent".into()))?;
    fn visit(
        path: &Path,
        root: &Path,
        seen: &mut Vec<std::path::PathBuf>,
        budget: &mut usize,
    ) -> Result<Value, IngestError> {
        if seen.len() >= 16 {
            return Err(IngestError::InvalidSource("include depth limit".into()));
        }
        let real = path
            .canonicalize()
            .map_err(|e| IngestError::InvalidSource(e.to_string()))?;
        if !real.starts_with(root) {
            return Err(IngestError::InvalidSource(
                "include escapes source directory".into(),
            ));
        }
        if seen.contains(&real) {
            return Err(IngestError::InvalidSource("include cycle".into()));
        }
        let meta =
            std::fs::metadata(&real).map_err(|e| IngestError::InvalidSource(e.to_string()))?;
        if !meta.is_file() || meta.len() > *budget as u64 {
            return Err(IngestError::InvalidSource("include payload limit".into()));
        }
        use std::io::Read;
        let mut raw = String::new();
        std::fs::File::open(&real)
            .map_err(|e| IngestError::InvalidSource(e.to_string()))?
            .take(*budget as u64 + 1)
            .read_to_string(&mut raw)
            .map_err(|e| IngestError::InvalidSource(e.to_string()))?;
        if raw.len() > *budget {
            return Err(IngestError::InvalidSource("include payload limit".into()));
        }
        *budget -= raw.len();
        let mut value = crate::jsonc::parse(&raw).map_err(IngestError::InvalidSource)?;
        if value.get("hexwitchSemanticLayers").is_some() {
            return Err(IngestError::InvalidSource(
                "reserved internal include field".into(),
            ));
        }
        // Validate before merging: a child's overrides must not hide malformed
        // parent values or invalid enablement behind accumulated settings.
        if let Some(enabled) = value.get("semanticHighlighting") {
            if !enabled.is_boolean() {
                return Err(IngestError::InvalidSource(
                    "semanticHighlighting must be boolean".into(),
                ));
            }
        }
        if let Some(colors) = value.get("colors") {
            let colors = colors
                .as_object()
                .ok_or_else(|| IngestError::InvalidSource("colors must be an object".into()))?;
            for (key, color) in colors {
                let color = color.as_str().ok_or_else(|| {
                    IngestError::InvalidSource(format!("color {key} must be a string"))
                })?;
                if color != "default" {
                    crate::semantic::parse_color(color).map_err(IngestError::InvalidSource)?;
                }
            }
        }
        if value.get("tokenColors").is_some_and(|v| !v.is_array())
            || value
                .get("semanticTokenColors")
                .is_some_and(|v| !v.is_object())
        {
            return Err(IngestError::InvalidSource(
                "invalid included token rule type".into(),
            ));
        }
        seen.push(real.clone());
        if let Some(include) = value.get("include").cloned() {
            let include = include.as_str().ok_or_else(|| {
                IngestError::InvalidSource("include must be a relative path".into())
            })?;
            let include = Path::new(include);
            if include.is_absolute()
                || include
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(IngestError::InvalidSource("unsafe include path".into()));
            }
            let mut parent = visit(&real.parent().unwrap().join(include), root, seen, budget)?;
            // Keep semantic rules as an ordered list across includes. Replacing
            // duplicate selector keys would discard inherited property values.
            let mut semantic_layers = parent
                .get("hexwitchSemanticLayers")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if semantic_layers.is_empty() {
                if let Some(rules) = parent.get("semanticTokenColors") {
                    semantic_layers.push(rules.clone());
                }
            }
            if let Some(rules) = value.get("semanticTokenColors") {
                semantic_layers.push(rules.clone());
            }
            value
                .as_object_mut()
                .ok_or_else(|| IngestError::InvalidSource("theme root must be object".into()))?
                .remove("include");
            merge(&mut parent, value);
            if !semantic_layers.is_empty() {
                parent["hexwitchSemanticLayers"] = Value::Array(semantic_layers);
            }
            value = parent;
        }
        seen.pop();
        Ok(value)
    }
    let value = visit(&canonical, root, &mut vec![], &mut (8 * 1024 * 1024))?;
    load_resolved_theme(value)
}
fn merge(parent: &mut Value, child: Value) {
    if let (Some(p), Some(c)) = (parent.as_object_mut(), child.as_object()) {
        for (key, value) in c {
            if key == "semanticHighlighting"
                && (p.get(key).and_then(Value::as_bool) == Some(true)
                    || value.as_bool() == Some(true))
            {
                p.insert(key.clone(), Value::Bool(true));
                continue;
            }
            if key == "tokenColors" {
                if let (Some(existing), Some(more)) = (
                    p.get_mut(key).and_then(Value::as_array_mut),
                    value.as_array(),
                ) {
                    existing.extend(more.clone());
                    continue;
                }
            }
            if matches!(key.as_str(), "colors" | "semanticTokenColors") {
                if let (Some(existing), Some(more)) = (
                    p.get_mut(key).and_then(Value::as_object_mut),
                    value.as_object(),
                ) {
                    existing.extend(more.clone());
                    continue;
                }
            }
            p.insert(key.clone(), value.clone());
        }
    } else {
        *parent = child;
    }
}
