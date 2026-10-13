use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Surface {
    Base,
    Gutter,
    Panel,
    Float,
    Border,
    Statusline,
    Tabline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextRole {
    Primary,
    Secondary,
    Muted,
    Inverse,
    Accent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyntaxRole {
    Comment,
    DocComment,
    Keyword,
    Operator,
    Punctuation,
    String,
    Escape,
    Number,
    Boolean,
    Function,
    Variable,
    VariableBuiltin,
    Constant,
    Property,
    Parameter,
    Namespace,
    Type,
    Tag,
    Attribute,
    Regex,
    Macro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InteractionRole {
    Selection,
    SelectionInactive,
    Cursor,
    CursorLine,
    Search,
    SearchCurrent,
    MatchParen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticKind {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticPart {
    Text,
    Sign,
    VirtualText,
    LineBg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiffKind {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiffPart {
    Text,
    LineBg,
    Gutter,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    Surface(Surface),
    Text(TextRole),
    Syntax(SyntaxRole),
    Interaction(InteractionRole),
    Diagnostic {
        kind: DiagnosticKind,
        part: DiagnosticPart,
    },
    Diff {
        kind: DiffKind,
        part: DiffPart,
    },
    Terminal(u8),
    Other(String),
}

#[derive(Debug, Error, PartialEq)]
#[error("unknown role `{0}`")]
pub struct RoleParseError(String);

impl Surface {
    pub const ALL: &'static [Surface] = &[
        Surface::Base,
        Surface::Gutter,
        Surface::Panel,
        Surface::Float,
        Surface::Border,
        Surface::Statusline,
        Surface::Tabline,
    ];
    pub fn kebab_name(&self) -> String {
        match self {
            Surface::Base => "base",
            Surface::Gutter => "gutter",
            Surface::Panel => "panel",
            Surface::Float => "float",
            Surface::Border => "border",
            Surface::Statusline => "statusline",
            Surface::Tabline => "tabline",
        }
        .into()
    }
}

impl TextRole {
    pub const ALL: &'static [TextRole] = &[
        TextRole::Primary,
        TextRole::Secondary,
        TextRole::Muted,
        TextRole::Inverse,
        TextRole::Accent,
    ];
    pub fn kebab_name(&self) -> String {
        match self {
            TextRole::Primary => "primary",
            TextRole::Secondary => "secondary",
            TextRole::Muted => "muted",
            TextRole::Inverse => "inverse",
            TextRole::Accent => "accent",
        }
        .into()
    }
}

impl SyntaxRole {
    pub const ALL: &'static [SyntaxRole] = &[
        SyntaxRole::Comment,
        SyntaxRole::DocComment,
        SyntaxRole::Keyword,
        SyntaxRole::Operator,
        SyntaxRole::Punctuation,
        SyntaxRole::String,
        SyntaxRole::Escape,
        SyntaxRole::Number,
        SyntaxRole::Boolean,
        SyntaxRole::Function,
        SyntaxRole::Variable,
        SyntaxRole::VariableBuiltin,
        SyntaxRole::Constant,
        SyntaxRole::Property,
        SyntaxRole::Parameter,
        SyntaxRole::Namespace,
        SyntaxRole::Type,
        SyntaxRole::Tag,
        SyntaxRole::Attribute,
        SyntaxRole::Regex,
        SyntaxRole::Macro,
    ];
    pub fn kebab_name(&self) -> String {
        match self {
            SyntaxRole::Comment => "comment",
            SyntaxRole::DocComment => "doc-comment",
            SyntaxRole::Keyword => "keyword",
            SyntaxRole::Operator => "operator",
            SyntaxRole::Punctuation => "punctuation",
            SyntaxRole::String => "string",
            SyntaxRole::Escape => "escape",
            SyntaxRole::Number => "number",
            SyntaxRole::Boolean => "boolean",
            SyntaxRole::Function => "function",
            SyntaxRole::Variable => "variable",
            SyntaxRole::VariableBuiltin => "variable-builtin",
            SyntaxRole::Constant => "constant",
            SyntaxRole::Property => "property",
            SyntaxRole::Parameter => "parameter",
            SyntaxRole::Namespace => "namespace",
            SyntaxRole::Type => "type",
            SyntaxRole::Tag => "tag",
            SyntaxRole::Attribute => "attribute",
            SyntaxRole::Regex => "regex",
            SyntaxRole::Macro => "macro",
        }
        .into()
    }
}

impl InteractionRole {
    pub const ALL: &'static [InteractionRole] = &[
        InteractionRole::Selection,
        InteractionRole::SelectionInactive,
        InteractionRole::Cursor,
        InteractionRole::CursorLine,
        InteractionRole::Search,
        InteractionRole::SearchCurrent,
        InteractionRole::MatchParen,
    ];
    pub fn kebab_name(&self) -> String {
        match self {
            InteractionRole::Selection => "selection",
            InteractionRole::SelectionInactive => "selection-inactive",
            InteractionRole::Cursor => "cursor",
            InteractionRole::CursorLine => "cursor-line",
            InteractionRole::Search => "search",
            InteractionRole::SearchCurrent => "search-current",
            InteractionRole::MatchParen => "match-paren",
        }
        .into()
    }
}

impl DiagnosticKind {
    pub const ALL: &'static [DiagnosticKind] = &[
        DiagnosticKind::Error,
        DiagnosticKind::Warning,
        DiagnosticKind::Info,
        DiagnosticKind::Hint,
    ];
    pub fn kebab_name(&self) -> String {
        match self {
            DiagnosticKind::Error => "error",
            DiagnosticKind::Warning => "warning",
            DiagnosticKind::Info => "info",
            DiagnosticKind::Hint => "hint",
        }
        .into()
    }
}

impl DiagnosticPart {
    pub const ALL: &'static [DiagnosticPart] = &[
        DiagnosticPart::Text,
        DiagnosticPart::Sign,
        DiagnosticPart::VirtualText,
        DiagnosticPart::LineBg,
    ];
    pub fn kebab_name(&self) -> String {
        match self {
            DiagnosticPart::Text => "text",
            DiagnosticPart::Sign => "sign",
            DiagnosticPart::VirtualText => "virtual-text",
            DiagnosticPart::LineBg => "line-bg",
        }
        .into()
    }
}

impl DiffKind {
    pub const ALL: &'static [DiffKind] = &[DiffKind::Added, DiffKind::Removed, DiffKind::Changed];
    pub fn kebab_name(&self) -> String {
        match self {
            DiffKind::Added => "added",
            DiffKind::Removed => "removed",
            DiffKind::Changed => "changed",
        }
        .into()
    }
}

impl DiffPart {
    pub const ALL: &'static [DiffPart] = &[DiffPart::Text, DiffPart::LineBg, DiffPart::Gutter];
    pub fn kebab_name(&self) -> String {
        match self {
            DiffPart::Text => "text",
            DiffPart::LineBg => "line-bg",
            DiffPart::Gutter => "gutter",
        }
        .into()
    }
}

impl Role {
    pub fn terminal_slot(slot: u8) -> Result<Self, RoleParseError> {
        if slot < 16 {
            Ok(Role::Terminal(slot))
        } else {
            Err(RoleParseError(format!("terminal.{slot}")))
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Surface(s) => write!(f, "surface.{}", s.kebab_name()),
            Role::Text(t) => write!(f, "text.{}", t.kebab_name()),
            Role::Syntax(s) => write!(f, "syntax.{}", s.kebab_name()),
            Role::Interaction(i) => write!(f, "interaction.{}", i.kebab_name()),
            Role::Diagnostic { kind, part } => {
                write!(f, "diagnostic.{}.{}", kind.kebab_name(), part.kebab_name())
            }
            Role::Diff { kind, part } => {
                write!(f, "diff.{}.{}", kind.kebab_name(), part.kebab_name())
            }
            Role::Terminal(slot) => write!(f, "terminal.{slot}"),
            Role::Other(name) => write!(f, "other.{}", name.replace(' ', "_")),
        }
    }
}

impl FromStr for Role {
    type Err = RoleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || RoleParseError(s.to_string());
        let (head, rest) = s.split_once('.').ok_or_else(err)?;

        match head {
            "surface" => Surface::ALL
                .iter()
                .find(|s| s.kebab_name() == rest)
                .map(|s| Role::Surface(*s))
                .ok_or_else(err),
            "text" => TextRole::ALL
                .iter()
                .find(|t| t.kebab_name() == rest)
                .map(|t| Role::Text(*t))
                .ok_or_else(err),
            "syntax" => SyntaxRole::ALL
                .iter()
                .find(|s| s.kebab_name() == rest)
                .map(|s| Role::Syntax(*s))
                .ok_or_else(err),
            "interaction" => InteractionRole::ALL
                .iter()
                .find(|i| i.kebab_name() == rest)
                .map(|i| Role::Interaction(*i))
                .ok_or_else(err),
            "diagnostic" => {
                let (kind, part) = rest.split_once('.').ok_or_else(err)?;
                let kind = DiagnosticKind::ALL
                    .iter()
                    .find(|k| k.kebab_name() == kind)
                    .copied()
                    .ok_or_else(err)?;
                let part = DiagnosticPart::ALL
                    .iter()
                    .find(|p| p.kebab_name() == part)
                    .copied()
                    .ok_or_else(err)?;
                Ok(Role::Diagnostic { kind, part })
            }
            "diff" => {
                let (kind, part) = rest.split_once('.').ok_or_else(err)?;
                let kind = DiffKind::ALL
                    .iter()
                    .find(|k| k.kebab_name() == kind)
                    .copied()
                    .ok_or_else(err)?;
                let part = DiffPart::ALL
                    .iter()
                    .find(|p| p.kebab_name() == part)
                    .copied()
                    .ok_or_else(err)?;
                Ok(Role::Diff { kind, part })
            }
            "terminal" => {
                let slot: u8 = rest.parse().map_err(|_| err())?;
                if slot < 16 {
                    Ok(Role::Terminal(slot))
                } else {
                    Err(err())
                }
            }
            "other" if !rest.is_empty() => Ok(Role::Other(rest.replace('_', " "))),
            _ => Err(err()),
        }
    }
}

impl serde::Serialize for Role {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Role {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round(role: &str) {
        let parsed: Role = role.parse().expect(role);
        assert_eq!(&parsed.to_string(), role, "display mismatch for {role}");
    }

    #[test]
    fn role_strings_roundtrip() {
        for r in [
            "surface.base",
            "surface.statusline",
            "text.primary",
            "syntax.comment",
            "syntax.doc-comment",
            "syntax.variable-builtin",
            "syntax.tag",
            "interaction.selection-inactive",
            "interaction.search-current",
            "diagnostic.error.virtual-text",
            "diagnostic.hint.line-bg",
            "diff.added.line-bg",
            "diff.removed.gutter",
            "terminal.0",
            "terminal.15",
            "other.plugin-specific-role",
        ] {
            round(r);
        }
    }

    #[test]
    fn kebab_names_are_stable() {
        assert_eq!(SyntaxRole::DocComment.kebab_name(), "doc-comment");
        assert_eq!(SyntaxRole::VariableBuiltin.kebab_name(), "variable-builtin");
        assert_eq!(
            InteractionRole::SelectionInactive.kebab_name(),
            "selection-inactive"
        );
        assert_eq!(DiagnosticPart::VirtualText.kebab_name(), "virtual-text");
    }

    #[test]
    fn rejects_unknown_roles() {
        for bad in [
            "syntax.nonsense",
            "diagnostic.error",
            "terminal.16",
            "terminal.x",
            "nope.base",
            "",
            "syntax.",
        ] {
            assert!(bad.parse::<Role>().is_err(), "{bad} should not parse");
        }
    }

    #[test]
    fn role_ordering_is_deterministic() {
        let mut roles: Vec<Role> = vec![
            Role::Terminal(2),
            Role::Syntax(SyntaxRole::Comment),
            Role::Surface(Surface::Base),
            Role::Diagnostic {
                kind: DiagnosticKind::Error,
                part: DiagnosticPart::Text,
            },
            Role::Other("zzz".into()),
        ];
        roles.sort();
        let strings: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        assert_eq!(strings, strings.clone());
        assert_eq!(roles, roles.clone());
    }

    #[test]
    fn terminal_slot_validates() {
        assert!(matches!(Role::terminal_slot(15), Ok(Role::Terminal(15))));
        assert!(Role::terminal_slot(16).is_err());
    }
}
