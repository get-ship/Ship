//! v1 sidecar protocol — re-export of `ship-sidecar-proto`.
//!
//! Codegen lives in the `ship-sidecar-proto` crate so substrate can
//! depend on the wire types without a cycle through ship-isolation.
//! Importing through this module keeps existing
//! `ship_isolation::sidecar::proto::*` paths working unchanged.

pub use ship_sidecar_proto::*;
