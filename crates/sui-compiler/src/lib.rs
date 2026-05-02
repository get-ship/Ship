//! `sui-compiler` — ahead-of-time compiler for `.sui` files.
//!
//! Same semantics as the Phase-2 interpreter
//! (`packages/ui-dom`); everything that can be resolved at build
//! time is. The emitted ESM module imports from `@ship/ui-runtime`
//! and constructs the rendered tree directly, with no parser code in
//! the runtime payload.
//!
//! # Public API
//!
//! ```ignore
//! use sui_compiler::{compile, CompileOpts, CompileMode, ComponentMeta};
//!
//! let out = compile(source, CompileOpts {
//!     setup_url: Some("./view.ts".into()),
//!     mode: CompileMode::View,
//!     source_path: None,
//!     component_meta: None,
//!     styles: None,
//!     emit_dts: true,
//! })?;
//! println!("{}", out.js);
//! println!("{:?}", out.dts);
//! ```
//!
//! See `crates/sui-compiler/src/parse/` for the parse pipeline,
//! `src/lower/` for IR lowering, and `src/emit/` for JS / CEM
//! generation. Diagnostics carry byte spans pointing at the user's
//! source — see `src/diagnostics.rs`.

use std::path::PathBuf;

pub mod bindings;
pub mod check;
pub mod diagnostics;
pub mod emit;
pub mod ir;
pub mod islands;
pub mod keys;
pub mod lower;
pub mod parse;
pub mod render;

pub use diagnostics::{CompileError, Span};

/// Compile mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// Auto-detect: picks `Component` when `component_meta` is set, otherwise `View`.
    Auto,
    View,
    Component,
}

/// Tag name and shadow-root flag for a component file. Passed in from the
/// build system (read from the per-component `.sui.json` metadata file) so
/// the template source no longer needs a `<component>` wrapper element.
#[derive(Debug, Clone)]
pub struct ComponentMeta {
    pub name: String,
    pub shadow: bool,
}

#[derive(Debug, Clone)]
pub struct CompileOpts {
    /// Path to the paired `.ts` setup module, relative to the emitted
    /// JS module's location. When `None`, no `import setup …` is emitted.
    pub setup_url: Option<String>,
    pub mode: CompileMode,
    /// For diagnostics. Optional.
    pub source_path: Option<PathBuf>,
    /// External CSS to prepend as a `<style>` block before parsing. Only
    /// valid for component sources (`component_meta` must be set or `mode`
    /// must be `Component`); otherwise `compile()` errors.
    pub styles: Option<String>,
    /// Component metadata (tag name + shadow flag). When set, `compile()`
    /// treats the source as a component file regardless of `mode`.
    pub component_meta: Option<ComponentMeta>,
    /// Emit a `.sui.d.ts` declaration file alongside the JS module.
    /// Defaults to `true`. Pass `false` (via `--no-dts`) to suppress.
    pub emit_dts: bool,
}

impl Default for CompileOpts {
    fn default() -> Self {
        Self {
            setup_url: None,
            mode: CompileMode::Auto,
            source_path: None,
            styles: None,
            component_meta: None,
            emit_dts: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub js: String,
    pub css: Option<String>,
    pub cem: Option<serde_json::Value>,
    pub schema: Option<serde_json::Value>,
    /// Emitted `.sui.d.ts` text, present when `CompileOpts::emit_dts` is true.
    pub dts: Option<String>,
}

/// Compile a `.sui` source string.
pub fn compile(source: &str, opts: CompileOpts) -> Result<CompileOutput, CompileError> {
    let merged_owned;
    let effective_source: &str = if let Some(css) = opts.styles.as_deref() {
        let is_component =
            opts.component_meta.is_some() || opts.mode == CompileMode::Component;
        if !is_component {
            return Err(CompileError::new(
                "--styles is only valid for component sources",
            ));
        }
        merged_owned = format!("<style>{css}</style>\n{source}");
        &merged_owned
    } else {
        source
    };

    let mode = match opts.mode {
        CompileMode::Auto => {
            if opts.component_meta.is_some() {
                CompileMode::Component
            } else {
                CompileMode::View
            }
        }
        m => m,
    };

    let program = parse::parse(effective_source).map_err(|e| attach_path(e, &opts))?;
    check::run(&program, opts.source_path.as_deref()).map_err(|e| attach_path(e, &opts))?;
    let bindings = bindings::build(&program);
    let lowered = lower::lower(&program, mode);

    let js = emit::js::emit_module(&program, &lowered, &bindings, &opts, mode);
    let cem = if mode == CompileMode::Component {
        Some(emit::cem::emit_cem(&program, &opts))
    } else {
        None
    };
    let schema = Some(emit::schema::emit_schema(&program, &opts));
    let dts = if opts.emit_dts {
        Some(emit::dts::emit_dts(&program))
    } else {
        None
    };
    Ok(CompileOutput { js, css: None, cem, schema, dts })
}

fn attach_path(err: CompileError, opts: &CompileOpts) -> CompileError {
    match &opts.source_path {
        Some(p) => err.with_path(p.clone()),
        None => err,
    }
}
