pub mod role;
pub mod style;
pub mod theme;

pub use role::{
    DiagnosticKind, DiagnosticPart, DiffKind, DiffPart, InteractionRole, Role, RoleParseError,
    Surface, SyntaxRole, TextRole,
};
pub use style::{Origin, Property, PropertyState, Provenance, SourceInfo, Style, UnderlineStyle};
pub use theme::{Appearance, Relationship, RelationshipKind, SemanticStyle, Theme, ThemeMeta};
