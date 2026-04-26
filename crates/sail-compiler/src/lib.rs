//! Sail compiler — turns `.sail` single-file components into shippable JS,
//! SSR templates, TypeScript declarations, and JSON schemas.
//!
//! Layer 2 of the Sail framework. Open source. No Ship dependencies.
//!
//! See `.artifacts/specs/sail/02-compiler-architecture.md` for the design.
//!
//! # Pipeline
//!
//! ```text
//! .sail file
//!   ↓ parse        (phase 1)
//!   ↓ schema       (phase 2)  — resolves `data x = ship.x.y(args)` against pluggable sources
//!   ↓ types        (phase 3)  — TypeScript via type-stripping or full check
//!   ↓ lower        (phase 4)  — AST → IR with reactive bindings explicit
//!   ↓ classify     (phase 5)  — static vs reactive interpolation classification
//!   ↓ scope css    (phase 6)  — `:host` rewrite, token resolution
//!   ↓ emit ssr     (phase 7)  — server-renderable HTML/template
//!   ↓ emit js      (phase 8)  — ES module with custom-element class
//!   ↓ emit dts     (phase 9)  — TypeScript declarations
//!   ↓ emit schema  (phase 10) — JSON schema for LSP/devtools
//! ```
//!
//! Each phase has a clean input → output contract. Phases 1-6 are pure
//! (deterministic given inputs). Phases 2-3 read external state. Phases
//! 7-10 are emission only.
//!
//! # Stability
//!
//! Public API (this `lib.rs` and the `parse::ast`, `analyze`,
//! `emit::*` outputs) is the OSS contract. Internal modules can move.

pub mod parse;
pub mod analyze;
pub mod lower;
pub mod emit;
pub mod schema_source;

use anyhow::Result;
use std::path::Path;

/// Compile a single `.sail` file. Returns the four emission outputs:
/// the runtime ES module, the SSR template, the TypeScript declaration,
/// and the JSON schema.
///
/// Phases 1-10 wired here. The actual phase implementations are stubs
/// to be filled in per the spec.
pub fn compile_file(_path: &Path, _opts: &CompileOpts) -> Result<CompileResult> {
    // Phase 1: parse
    // Phase 2-3: analyze (schema + types)
    // Phase 4-5: lower (template + reactivity classification)
    // Phase 6: scope CSS
    // Phase 7-10: emit
    todo!("phases not yet implemented — see 02-compiler-architecture.md")
}

/// Options to the compiler. Schema sources are pluggable so the same
/// compiler runs against a Ship daemon, an OpenAPI spec, a static
/// fixture, or nothing at all.
#[derive(Default)]
pub struct CompileOpts {
    /// Schema sources resolved in order; first match wins.
    pub schema_sources: Vec<Box<dyn schema_source::SchemaSource>>,
    /// Whether to run full TypeScript type-checking (slow) or just
    /// type-stripping (fast). Default: stripping.
    pub typecheck_full: bool,
}

/// Output of compiling a single `.sail` file.
#[derive(Debug, Default)]
pub struct CompileResult {
    /// The ES module emitted at `<tag>.js`.
    pub js: String,
    /// SSR-callable template (function source or HTML template).
    pub ssr: String,
    /// TypeScript declaration emitted at `<tag>.d.ts`.
    pub dts: String,
    /// JSON schema at `<tag>.schema.json` (LSP + devtools).
    pub schema: String,
    /// Diagnostics produced during compilation.
    pub diagnostics: Vec<Diagnostic>,
}

/// Compile-time diagnostic. File:line:col + message + severity.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,           // e.g., "SAIL/E0042"
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_opts_default() {
        let opts = CompileOpts::default();
        assert!(opts.schema_sources.is_empty());
        assert!(!opts.typecheck_full);
    }

    #[test]
    fn compile_result_default() {
        let r = CompileResult::default();
        assert!(r.js.is_empty());
        assert!(r.diagnostics.is_empty());
    }
}
