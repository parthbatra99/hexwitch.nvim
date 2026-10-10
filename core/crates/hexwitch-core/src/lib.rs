//! hexwitch-core: the editor-independent theme engine.
//!
//! The engine's job is not to pick colors by taste. It is to:
//! 1. Represent a theme as a graph of roles and styles ([`ir`]).
//! 2. Do color science that is deterministic and testable ([`color`]).
//! 3. Later: analyze, revise under constraints, validate rendered scenes,
//!    and emit artifacts for concrete editors.
//!
//! Layered plan (see repository README):
//! M0 foundations (this crate's color + IR) -> M1 ingest/analyze ->
//! M2 revise/emit -> M3 interpretation bridge -> M4 second adapter.

pub mod analyze;
pub mod color;
pub mod emit;
pub mod eval;
pub mod ingest;
pub mod interpret;
pub mod ir;
pub mod revise;

/// Version of the intermediate representation itself.
/// Bumped on breaking changes to the IR schema; themes carry it for forward
/// compatibility checks.
pub const IR_VERSION: u32 = 0;
