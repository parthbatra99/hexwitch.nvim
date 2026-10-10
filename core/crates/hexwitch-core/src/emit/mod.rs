//! Emit: lower IR into concrete editor artifacts.
//!
//! Emitting is a *translation with losses*, and the contract is that
//! losses are reported, not hidden. Each target returns its artifact plus
//! an [`EmitReport`] describing what was defined directly, what was linked
//! (aliased intentionally), what was skipped because the IR lacks the
//! role, and which IR roles the target has no place for.

pub mod nvim;
mod semantic_nvim;

pub use nvim::{emit_nvim, EmitError, EmitReport, NvimEmit};

/// Version stamp embedded in generated artifacts.
pub const EMITTED_BY: &str = concat!("hexwitch ", env!("CARGO_PKG_VERSION"));
