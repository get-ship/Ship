//! Generated ConnectRPC bindings for `ship.sidecar.v1`.
//!
//! Re-exports the codegen so consumers import
//! `ship_sidecar_proto::*` instead of poking into the codegen
//! path. Source of truth is `proto/ship/sidecar/v1/sidecar.proto`.
//!
//! This crate exists to break a cycle: `ship-substrate` needs the
//! boundary protocol types for its sandbox-aware transport, but
//! also defines the lower-level `Isolator` trait that
//! `ship-isolation` consumes. Putting the proto in its own
//! zero-dependency crate lets both substrate and isolation depend
//! on it.

#![allow(
    clippy::all,
    non_camel_case_types,
    dead_code,
    unused_imports,
    unused_qualifications,
    non_snake_case
)]

pub mod ship_sidecar_v1 {
    include!(concat!(env!("OUT_DIR"), "/ship.sidecar.v1.mod.rs"));
}

pub use ship_sidecar_v1::*;
