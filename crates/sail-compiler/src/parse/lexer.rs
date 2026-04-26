//! Tokenizer helpers for the Sail parser.
//!
//! Used by `parser.rs` to walk template HTML, script JS/TS, and CSS
//! without pulling in a full parser library. Same approach as the
//! legacy SFC compiler at `apps/shipd/src/views/sfc/script_scan.rs`.
//!
//! Implementation deferred — see `02-compiler-architecture.md § Phase 1`.
