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
//! * **`None` means unspecified/inheritable**, not transparent and not
//!   "no opinion" — fallback resolution is an explicit adapter concern.
//! * **Provenance is first-class.** Which colors the user loves (and must
//!   survive revision) is recorded in the IR itself.

pub mod role;
pub mod style;
pub mod theme;

pub use role::{
    DiagnosticKind, DiagnosticPart, DiffKind, DiffPart, InteractionRole, Role, RoleParseError,
    Surface, SyntaxRole, TextRole,
};
pub use style::{Provenance, Style, UnderlineStyle};
pub use theme::{Appearance, Theme, ThemeMeta};
