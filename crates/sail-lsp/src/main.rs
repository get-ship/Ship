//! `sail-lsp` — Language server for `.set` single-file components.
//!
//! Thin transport: stdio JSON-RPC in, [`server::run_stdio`] runs the
//! protocol loop. All semantic work delegates to `sail-compiler`; the
//! parse foundation is `tree-sitter-set` (stubbed in v0 — see
//! [`document`] for the integration shape).

mod diagnostics;
mod document;
mod server;

use anyhow::Result;

fn main() -> Result<()> {
    server::run_stdio()
}
