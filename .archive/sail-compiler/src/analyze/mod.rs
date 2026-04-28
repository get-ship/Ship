//! Phases 2-5 — analysis: schema resolution, type analysis, lowering,
//! reactivity classification.
//!
//! Reads the parser's AST + schema sources + (optionally) a TypeScript
//! type checker; emits an `AnalyzedFile` with typed declarations and
//! a dependency graph for derived/effect/data.
//!
//! See `.artifacts/specs/sail/02-compiler-architecture.md § Phases 2-5`.

pub mod schema;
pub mod types;
pub mod reactivity;
pub mod deps;
pub mod tokens;
pub mod symbols;
pub mod data_paths;
pub mod rules;

use crate::parse::ast::SailFile;
use crate::Diagnostic;
use anyhow::Result;

#[derive(Debug, Default)]
pub struct AnalyzedFile {
    pub ast: SailFile,
    pub diagnostics: Vec<Diagnostic>,
    // pub schema: AnalyzedSchema,         (filled in as phases land)
}

pub fn analyze(_file: SailFile, _opts: &crate::CompileOpts) -> Result<AnalyzedFile> {
    todo!("phases 2-5 analysis")
}
