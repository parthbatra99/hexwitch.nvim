//! Sandboxed include resolution; no remote URLs or parent/symlink escapes.
use crate::ingest::{load_theme, IngestError};
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
            value
                .as_object_mut()
                .ok_or_else(|| IngestError::InvalidSource("theme root must be object".into()))?
                .remove("include");
            merge(&mut parent, value);
            value = parent;
        }
        seen.pop();
        Ok(value)
    }
    let value = visit(&canonical, root, &mut vec![], &mut (8 * 1024 * 1024))?;
    load_theme(&value.to_string())
}
fn merge(parent: &mut Value, child: Value) {
    if let (Some(p), Some(c)) = (parent.as_object_mut(), child.as_object()) {
        for (key, value) in c {
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
