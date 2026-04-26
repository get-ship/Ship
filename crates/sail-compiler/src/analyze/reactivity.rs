//! Phase 5 — reactivity classification.
//!
//! For each `{expr}` in the template and each binding's value
//! expression, walk the AST and determine:
//!
//! - **Static** — no reactive dependencies → bake into SSR HTML
//! - **Reactive** — at least one reactive dep → emit a reactive binding
//!
//! Reactive deps are: `state`, `derived`, `data`, and `prop` declarations.
//!
//! See `.artifacts/specs/sail/02-compiler-architecture.md § Phase 5`.
