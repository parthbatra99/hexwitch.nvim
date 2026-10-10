//! Theme intermediate representation (IR).
//!
//! The IR is the engine's central artifact. Everything operates on it:
//! ingest (reference theme -> IR), analyze (IR -> fingerprint), revise
//! (IR + edit spec -> IR'), validate (IR -> scene report), emit
//! (IR -> editor artifact).
//!
//! Core commitments, from the first-principles design:
//!
//! * **Roles, not editor groups.** `syntax.keyword` is a *meaning*; whether
//!   it lands in `@keyword` (Neovim), `keyword` (TextMate/semantic,
//!   VS Code) or a capture (Zed) is each adapter's lowering problem.
//! * **Styles carry more than color.** Foreground, background, special
//!   (underline/undercurl color), font attributes, blend, and provenance.
//! * **Property states are explicit.** Omission, inherit, clear and concrete
//!   values are distinct; opacity and typography remain separate properties.
//! * **Source and protection are independent.** Generated properties can be
//!   locked without losing origin; legacy provenance is compatibility-only.

pub mod role;
pub mod style;
pub mod theme;

pub use role::{
    DiagnosticKind, DiagnosticPart, DiffKind, DiffPart, InteractionRole, Role, RoleParseError,
    Surface, SyntaxRole, TextRole,
};
pub use style::{Origin, Property, PropertyState, Provenance, SourceInfo, Style, UnderlineStyle};
pub use theme::{Appearance, Relationship, RelationshipKind, SemanticStyle, Theme, ThemeMeta};
