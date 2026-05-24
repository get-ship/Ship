//! ConnectRPC surface for the Ship operation registry (`ship.ops.v1`).
//!
//! Generated from `proto/ship/ops/v1/*.proto` via `connectrpc-build` in
//! `build.rs` — the single source of truth for op **shapes and policy**
//! (decision #67, option a). `connectrpc-build` emits both the server-side
//! `OpsService` trait (implemented by the daemon — compile-time 1-1 dispatch
//! conformance) and a typed client (CLI/GUI). The daemon mounts the service on
//! its existing axum router via `connectrpc`'s `axum` integration; the surface
//! is additive beside the REST/SSE routes.

// Generated code trips a few stylistic clippy lints (default-impl match
// bindings, large descriptor blobs); allow on the module rather than patch the
// generator output. Mirrors ship-gateway-client.
#[allow(
    clippy::match_single_binding,
    clippy::large_stack_arrays,
    clippy::needless_borrow,
    clippy::useless_conversion,
    clippy::redundant_clone
)]
pub mod ship {
    pub mod ops {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/ship.ops.v1.mod.rs"));
        }
    }
}

pub use ship::ops::v1::*;

pub mod policy;

/// The compiled `FileDescriptorSet`, including the custom method options
/// (`capability`, `blast_radius`, `op_sensitive`). The daemon gate reads policy
/// from this at startup (buffa reflection) rather than from the generated Rust,
/// since the runtime drops custom options in normal codegen — see #74.
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ops_descriptor.bin"));
