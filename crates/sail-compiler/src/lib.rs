//! Sail compiler — turns `.set` single-file components into shippable JS,
//! SSR templates, TypeScript declarations, and JSON schemas.
//!
//! Layer 2 of the Sail framework. Open source. No Ship dependencies.
//!
//! See `.artifacts/specs/sail/02-compiler-architecture.md` for the design.
//!
//! # Pipeline
//!
//! ```text
//! .set file
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

use anyhow::{Context, Result};
use std::path::Path;

/// Compile a single `.set` file. Returns the four emission outputs:
/// the runtime ES module, the SSR template, the TypeScript declaration,
/// and the JSON schema.
///
/// Currently wired phases:
/// - Phase 1 (parse) — full
/// - Phase 8 (runtime emit) — MVP subset (see `emit::runtime`)
/// - Phases 2-7, 9-10 — stubs (`ssr`, `dts`, `schema` strings empty)
pub fn compile_file(path: &Path, opts: &CompileOpts) -> Result<CompileResult> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let base = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let fs_loader: Box<dyn lower::layout::Loader> =
        Box::new(lower::layout::FsLoader::new(base));
    // Use the caller's opts but inject our filesystem loader if they
    // didn't supply one. Other fields stay theirs.
    let loader = opts.loader.as_deref().unwrap_or(&*fs_loader);
    compile_source_inner(&source, &tag_for(path), opts, Some(loader))
}

/// Compile a `.set` source string with an explicit tag name.
pub fn compile_source(source: &str, tag: &str) -> Result<CompileResult> {
    compile_source_with(source, tag, &CompileOpts::default())
}

/// `compile_source` with explicit options. Tokens, schema sources, and
/// other analyzer inputs come from `opts`.
pub fn compile_source_with(
    source: &str,
    tag: &str,
    opts: &CompileOpts,
) -> Result<CompileResult> {
    let loader_ref: Option<&dyn lower::layout::Loader> =
        opts.loader.as_deref().map(|l| l as &dyn lower::layout::Loader);
    compile_source_inner(source, tag, opts, loader_ref)
}

fn compile_source_inner(
    source: &str,
    tag: &str,
    opts: &CompileOpts,
    loader: Option<&dyn lower::layout::Loader>,
) -> Result<CompileResult> {
    let mut file = parse::parse(source).context("parse")?;
    // Resolve compile-time layout primitives (<extends>, <block>,
    // <fill>, <include>). With no loader, files that don't use
    // extends/include resolve to themselves; files that DO use them
    // hit "source not found" errors (which is the correct failure
    // mode — silent fallback would mask layout breakage).
    if let Some(template) = file.template.as_mut() {
        let resolved = match loader {
            Some(l) => lower::layout::resolve(template, l).context("layout resolve")?,
            None => {
                // Standalone path: only honor blocks-with-defaults and
                // strip orphan fills. Treat absent loader as an empty
                // map, which errors on extends/include but no-ops
                // otherwise.
                lower::layout::resolve(
                    template,
                    &lower::layout::MapLoader { files: std::collections::HashMap::new() },
                )
                .context("layout resolve (no loader)")?
            }
        };
        *template = resolved;
    }
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    analyze::reactivity::analyze(&file, &mut diagnostics);
    analyze::tokens::analyze(&file, opts.tokens.as_ref(), &mut diagnostics);
    let (js, _html) = emit::runtime::emit_with_html(&file, tag);
    let ssr = emit::ssr::emit(&file, tag);
    let dts = emit::types::emit(&file, tag);
    let schema = emit::schema::emit(&file, tag);
    Ok(CompileResult {
        js,
        ssr,
        dts,
        schema,
        diagnostics,
    })
}

/// Derive the custom-element tag from a file path: stem in kebab-case.
/// `MyCounter.set` → `my-counter`; `app-counter.set` → `app-counter`.
/// Single-word stems get a `sail-` prefix so they remain valid custom
/// element names (which require a hyphen).
pub fn tag_for(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("component");
    let mut out = String::new();
    for (i, c) in stem.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 { out.push('-'); }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    if !out.contains('-') {
        out = format!("sail-{out}");
    }
    out
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
    /// Configured design tokens (`color.fg`, `space.3`, …). When
    /// `None`, token references are accepted unchecked. When `Some`,
    /// unknown references emit SAIL/E0010.
    pub tokens: Option<analyze::tokens::TokenSet>,
    /// Layout loader for `<extends>` / `<include>` resolution. When
    /// `None`, files that use extends/include error with "not found";
    /// `compile_file` injects a filesystem loader rooted at the file's
    /// parent dir.
    pub loader: Option<Box<dyn lower::layout::Loader + Send + Sync>>,
}

/// Output of compiling a single `.set` file.
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
