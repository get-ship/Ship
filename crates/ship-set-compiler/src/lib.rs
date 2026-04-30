//! `ship-set-compiler` — ahead-of-time compiler for `.html.set` files.
//!
//! Same semantics as the Phase-2 interpreter
//! (`packages/ship-set-dom`); everything that can be resolved at build
//! time is. The emitted ESM module imports from `@ship-set/runtime`
//! and constructs the rendered tree directly, with no parser code in
//! the runtime payload.
//!
//! # Public API
//!
//! ```ignore
//! use ship_set_compiler::{compile, CompileOpts, CompileMode};
//!
//! let out = compile(source, CompileOpts {
//!     setup_url: Some("./view.ts".into()),
//!     mode: CompileMode::View,
//!     source_path: None,
//! })?;
//! println!("{}", out.js);
//! ```
//!
//! See `crates/ship-set-compiler/src/parse/` for the parse pipeline,
//! `src/lower/` for IR lowering, and `src/emit/` for JS / CEM
//! generation. Diagnostics carry byte spans pointing at the user's
//! source — see `src/diagnostics.rs`.

use std::path::PathBuf;

pub mod check;
pub mod diagnostics;
pub mod emit;
pub mod ir;
pub mod lower;
pub mod parse;
pub mod render;

pub use diagnostics::{CompileError, Span};

/// Compile mode. View files have no `<component>` declaration; component
/// files declare exactly one and emit `customElements.define`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// Auto-detect by inspecting the parsed program. The compiler picks
    /// `Component` when a `<component>` declaration is present.
    Auto,
    View,
    Component,
}

#[derive(Debug, Clone)]
pub struct CompileOpts {
    /// Path to the paired `.ts` setup module, relative to the emitted
    /// JS module's location. When `None`, no `import setup …` is emitted.
    pub setup_url: Option<String>,
    pub mode: CompileMode,
    /// For diagnostics. Optional.
    pub source_path: Option<PathBuf>,
}

impl Default for CompileOpts {
    fn default() -> Self {
        Self { setup_url: None, mode: CompileMode::Auto, source_path: None }
    }
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub js: String,
    pub css: Option<String>,
    pub cem: Option<serde_json::Value>,
    pub schema: Option<serde_json::Value>,
}

/// Compile a `.html.set` source string.
pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError> {
    let program = parse::parse(source).map_err(|e| attach_path(e, &opts))?;
    check::run(&program).map_err(|e| attach_path(e, &opts))?;
    let lowered = lower::lower(&program);
    let mode = match opts.mode {
        CompileMode::Auto => {
            if program.component.is_some() {
                CompileMode::Component
            } else {
                CompileMode::View
            }
        }
        m => m,
    };
    if mode == CompileMode::Component && program.component.is_none() {
        return Err(CompileError::new(
            "component mode requested but no <component> declaration found",
        ));
    }
    if mode == CompileMode::View && program.component.is_some() {
        return Err(CompileError::new(
            "view mode requested but a <component> declaration is present",
        ));
    }

    let js = emit::js::emit_module(&program, &lowered, &opts, mode);
    let cem = match (mode, &program.component) {
        (CompileMode::Component, Some(comp)) => Some(emit::cem::emit_cem(comp, &opts)),
        _ => None,
    };
    let schema = Some(emit::schema::emit_schema(&program));
    Ok(CompileOutput { js, css: None, cem, schema })
}

fn attach_path(err: CompileError, opts: &CompileOpts) -> CompileError {
    match &opts.source_path {
        Some(p) => err.with_path(p.clone()),
        None => err,
    }
}
