use crate::ir::Role;

pub fn valid_selector(selector: &str) -> bool {
    if selector == "*" {
        return true;
    }
    if let Some(prefix) = selector.strip_suffix(".*") {
        return matches!(
            prefix,
            "surface"
                | "text"
                | "syntax"
                | "interaction"
                | "diagnostic"
                | "diff"
                | "terminal"
                | "other"
        ) || [
            "diagnostic.error",
            "diagnostic.warning",
            "diagnostic.info",
            "diagnostic.hint",
            "diff.added",
            "diff.removed",
            "diff.changed",
        ]
        .contains(&prefix);
    }
    selector.parse::<Role>().is_ok()
}

pub fn matches(selector: &str, role: &Role) -> bool {
    valid_selector(selector)
        && (selector == "*"
            || selector.strip_suffix(".*").map_or_else(
                || selector == role.to_string(),
                |prefix| role.to_string().starts_with(&format!("{prefix}.")),
            ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arbitrary_wildcards_do_not_validate() {
        for invalid in [
            "nope.*",
            ".*",
            "syntax.nonsense.*",
            "syntax.**",
            "syntax.*.*",
        ] {
            assert!(!valid_selector(invalid), "{invalid}");
        }
        for valid in [
            "*",
            "syntax.*",
            "diagnostic.error.*",
            "diff.added.*",
            "other.*",
            "terminal.15",
        ] {
            assert!(valid_selector(valid), "{valid}");
        }
    }
}
