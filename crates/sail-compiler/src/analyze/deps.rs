//! Dependency graph builder.
//!
//! For each `derived` and `effect`, parse the expression's referenced
//! identifiers and resolve them to declarations. The graph drives:
//!
//! - Compile-time validation (effect on undeclared name = error)
//! - Runtime emission (the binding subscribes to its declared deps)
//!
//! See `02-compiler-architecture.md § Phase 4` for AnalyzedSchema shape.
