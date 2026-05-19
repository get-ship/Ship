//! v1 sidecar protocol — generated bindings.
//!
//! Re-exports the generated types so the rest of the crate (and the
//! sidecar binary) imports `ship_isolation::sidecar::proto::*` instead
//! of poking into the codegen path.
//!
//! Source of truth: `proto/ship/sidecar/v1/sidecar.proto`. See
//! `docs/sidecar-protocol.md` for the rationale + version discipline.

#![allow(clippy::all, non_camel_case_types, dead_code, unused_imports, unused_qualifications, non_snake_case)]

pub mod ship_sidecar_v1 {
    // codegen emits one mod.rs per package that includes all the
    // per-file shards. We follow what the gateway-client pattern does.
    include!(concat!(env!("OUT_DIR"), "/ship.sidecar.v1.mod.rs"));
}

pub use ship_sidecar_v1::*;
