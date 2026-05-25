//! `ship mcp serve` — run the Ship MCP server over stdio in-process.
//!
//! A thin local shim over the same `ShipServer` surface the daemon hosts at
//! `$DAEMON/mcp`. Claude Code (and other harnesses) spawn `ship mcp serve`
//! directly. The standalone `--http` listener was retired — the daemon is the
//! one HTTP transport.
//!
//! Logging is already initialised by the CLI before this is called;
//! everything writes to ~/.ship/machine/logs/ship.log via the tracing subscriber.

use anyhow::Result;

pub fn run() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(mcp::run())
}
