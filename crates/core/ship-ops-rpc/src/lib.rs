//! ConnectRPC surface for the Ship operation registry (`ship.ops.v1`).
//!
//! The proto bindings are generated once in the published `ship-proto` crate
//! (ship-contracts is the single source of truth for op **shapes and policy**,
//! decision #67, option a) and re-exported here. `ship-proto` emits both the
//! server-side `OpsService` trait (implemented by the daemon — compile-time
//! 1-1 dispatch conformance) and a typed client (CLI/GUI). The daemon mounts
//! the service on its existing axum router via `connectrpc`'s `axum` integration
//! (enabled graph-wide by shipd via feature-unification); the surface is
//! additive beside the REST/SSE routes. The hand-written `policy` + `catalog`
//! modules below are Ship implementation, not contract, and stay here.

// Proto bindings are produced once in the published `ship-proto` crate
// (ship-contracts is the SSOT) and re-exported here so existing consumers keep
// importing `ship_ops_rpc::ship::ops::v1::*` and the crate-root glob unchanged.
pub use ship_proto::ship;
pub use ship_proto::ship::ops::v1::*;

/// Yoke harness session Connect surface (`ship.yoke.v1`) — Bridge server,
/// shipd client. Not part of the public ops registry.
pub use ship_proto::ship::yoke;

pub mod policy;
pub use policy::BlastRadius;

pub mod catalog;
pub use catalog::{OpDescriptor, ArgDescriptor, CommandDescriptor, COMMANDS, OPS};

/// The compiled `FileDescriptorSet`, including the custom method options
/// (`capability`, `blast_radius`, `op_sensitive`). The daemon gate reads policy
/// from this at startup (buffa reflection) rather than from the generated Rust,
/// since the runtime drops custom options in normal codegen — see #74.
/// Re-exported from `ship-proto`, which embeds it via `--include_imports`.
pub const FILE_DESCRIPTOR_SET: &[u8] = ship_proto::OPS_FILE_DESCRIPTOR_SET;
