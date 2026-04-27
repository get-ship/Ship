//! Phase 4 — AST → lowered IR.
//!
//! The parser's AST is friendly to authors (deeply structured, control
//! flow blocks, etc.). The emitter wants something flatter:
//! - Static HTML with placeholder anchors
//! - A list of runtime bindings (text/show/attr/event/for/if) with
//!   their dependency arrays
//!
//! Lowering walks the AST, splits each `Node` into static parts +
//! reactive parts, and produces a `LoweredFile` ready for emission.
//!
//! See `.artifacts/specs/sail/02-compiler-architecture.md § Phase 4`.

pub mod template;
pub mod style;
pub mod ir;
