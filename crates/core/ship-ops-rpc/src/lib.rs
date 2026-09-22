//! ConnectRPC surface helpers for the Ship operation registry (`ship.ops.v1`).
//!
//! **Proto types live in `ship_proto`** (ship-contracts SSOT). This crate no
//! longer re-exports them — consumers import `ship_proto::ship::ops::v1::*`
//! (and `ship_proto::ship::code::v1` / gateway / internal) directly.
//!
//! What stays here is Ship implementation, not contract:
//! - [`policy`] — BlastRadius + FDS-backed method policy reader
//! - [`catalog`] — hand-authored COMMANDS/OPS descriptors for schema/MCP/codegen
//! - [`FILE_DESCRIPTOR_SET`] — ops descriptor bytes (also at
//!   `ship_proto::OPS_FILE_DESCRIPTOR_SET`)

pub mod policy;
pub use policy::BlastRadius;

pub mod catalog;
pub use catalog::{ArgDescriptor, COMMANDS, CommandDescriptor, OPS, OpDescriptor};

/// The compiled `FileDescriptorSet`, including the custom method options
/// (`capability`, `blast_radius`, `op_sensitive`). The daemon gate reads policy
/// from this at startup (buffa reflection) rather than from the generated Rust,
/// since the runtime drops custom options in normal codegen — see #74.
/// Re-exported from `ship-proto`, which embeds it via `--include_imports`.
pub const FILE_DESCRIPTOR_SET: &[u8] = ship_proto::OPS_FILE_DESCRIPTOR_SET;
