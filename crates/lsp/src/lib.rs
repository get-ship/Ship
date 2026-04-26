//! Ship LSP — diagnostics-only language server v1.
//!
//! Wraps the existing `lint` crate so editors can render the same
//! findings the CLI / daemon already produce. Sync, single-threaded,
//! built on `lsp-server`. The lint engine is fast enough that no
//! debounce is needed at typical view sizes; if a large view shows
//! latency we can debounce in [`server`] without touching `lint`.
//!
//! Public entry points:
//! - [`run_stdio`] — block the current thread serving LSP over stdio.
//!   Wired into the `ship lsp` subcommand.
//! - [`diagnostics::findings_to_diagnostics`] — pure conversion used by
//!   tests and any future re-use.

pub mod diagnostics;
pub mod uri;
pub mod workspace;

mod server;

pub use server::run_stdio;

#[cfg(test)]
mod tests;
