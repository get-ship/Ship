//! Op and command catalog — the single source of truth for what view templates,
//! `window.ship.*` codegen, and MCP tool auto-registration all read.
//!
//! Lives here (L0 ConnectRPC surface) so CLI/gateway can reach it without
//! depending on the daemon. The daemon's `schema::serve_schema()` handler sources
//! from this module.

mod commands;
mod types;

pub use commands::COMMANDS;
pub use types::{ArgDescriptor, CommandDescriptor, OpDescriptor};

/// Read-side op catalog. All domains have migrated to typed Connect RPCs;
/// this slice is empty and kept only for xtask / SDK codegen compatibility.
pub const OPS: &[OpDescriptor] = &[];
