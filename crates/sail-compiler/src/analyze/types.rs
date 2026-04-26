//! Phase 3 — type analysis.
//!
//! Two paths:
//! - **Type-stripping (default)**: run swc/oxc/esbuild over the script
//!   block, emit JS. ~100µs. No type errors caught at the compiler;
//!   the LSP runs full TS for that.
//! - **Full type-check (LSP/CI)**: spawn `tsc` with a synthesized
//!   tsconfig that includes ambient types for prop/state/derived/data.
//!
//! See `.artifacts/specs/sail/02-compiler-architecture.md § Phase 3`.
