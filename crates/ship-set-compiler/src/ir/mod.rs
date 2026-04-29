//! Intermediate representation. Built by `parse`, consumed by
//! `lower` + `emit`. Nodes carry the source `Span` so diagnostics
//! generated downstream still point at the user's file.

pub mod directive;
pub mod expr;
pub mod template;

pub use directive::*;
pub use expr::*;
pub use template::*;

use crate::diagnostics::Span;

/// The full IR for a single `.html.set` source file. Mirrors the
/// shape produced by `packages/ship-set-dom/src/parse.ts` so the
/// compiler accepts every shape the interpreter accepts.
#[derive(Debug, Clone)]
pub struct Program {
    pub data: Vec<DataDecl>,
    pub state: Vec<StateDecl>,
    pub component: Option<ComponentDecl>,
    pub template: Vec<Instr>,
    /// Total source length — useful for serialization tools.
    pub source_len: usize,
    /// Span of the whole input, for top-level diagnostics.
    pub span: Span,
}
