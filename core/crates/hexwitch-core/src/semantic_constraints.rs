use crate::ir::{Role, Style, Theme};
use crate::semantic::{
    portable_role, resolve_semantic_prevalidated, SemanticContext, STANDARD_TYPES,
};
use std::collections::BTreeSet;

const MAX_CONTEXTS: usize = 2048;

pub(crate) fn treatments(
    theme: &Theme,
    role: &Role,
    base: &Style,
) -> Result<Vec<(SemanticContext, Style)>, String> {
    let rules: Vec<_> = theme.modifiers.iter().filter(|r| r.role == *role).collect();
    if rules.is_empty()
        || theme
            .extensions
            .get("vscode.semanticHighlighting")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    {
        return Ok(vec![]);
    }
    theme.validate()?;
    base.validate()?;
    let modifiers: BTreeSet<_> = rules
        .iter()
        .flat_map(|r| r.modifiers.iter().cloned())
        .collect();
    let modifiers: Vec<_> = modifiers.into_iter().collect();
    let mut languages = BTreeSet::from([None]);
    languages.extend(rules.iter().map(|r| r.language.clone()));
    let mut types: BTreeSet<String> = STANDARD_TYPES
        .iter()
        .filter(|t| portable_role(t).as_ref() == Some(role))
        .map(|t| t.to_string())
        .collect();
    types.extend(rules.iter().filter_map(|r| r.token_type.clone()));
    if types.is_empty() {
        return Err("semantic constraint type mapping unavailable; floor unverified".into());
    }
    let count = 1usize
        .checked_shl(modifiers.len() as u32)
        .and_then(|n| n.checked_mul(languages.len()))
        .and_then(|n| n.checked_mul(types.len()));
    if !count.is_some_and(|count| count <= MAX_CONTEXTS) {
        return Err(format!(
            "semantic constraint context limit exceeded: {} modifiers x {} languages x {} types exceeds {MAX_CONTEXTS} contexts; floor unverified",
            modifiers.len(),
            languages.len(),
            types.len()
        ));
    }
    let mut treatments = vec![];
    for token_type in types {
        for language in &languages {
            for mask in 0..(1usize << modifiers.len()) {
                let context = SemanticContext {
                    token_type: token_type.clone(),
                    language: language.clone(),
                    modifiers: modifiers
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| mask & (1 << i) != 0)
                        .map(|(_, m)| m.clone())
                        .collect(),
                };
                let resolved = resolve_semantic_prevalidated(theme, role, &context, base)?;
                if !resolved.limitations.is_empty() {
                    return Err(format!(
                        "semantic floor unverified: {}",
                        resolved.limitations.join("; ")
                    ));
                }
                treatments.push((context, resolved.style));
            }
        }
    }
    Ok(treatments)
}
