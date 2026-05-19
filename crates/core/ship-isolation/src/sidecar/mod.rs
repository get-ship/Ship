//! Sidecar protocol — host ↔ in-sandbox daemon wire format.
//!
//! The sidecar is the daemon that runs *inside* a sandbox (container,
//! microVM, or cloud box) and accepts process control + lifecycle
//! commands. The host-side isolator talks to it the same way
//! regardless of where the sandbox physically lives.
//!
//! See `docs/sidecar-protocol.md` for the wire contract and
//! `proto/ship/sidecar/v1/sidecar.proto` for the schema.
//!
//! ## v1 vs legacy
//!
//! [`proto`] is the canonical v1 protocol — generated from
//! `proto/ship/sidecar/v1/sidecar.proto` via ConnectRPC. Use this for
//! every new caller and for the upcoming sidecar handler rewrite.
//!
//! [`protocol`] is the hand-typed JSON shape from the prototype
//! sidecar (`bin/ship-sidecar/`). Kept until the sidecar binary
//! ports to Connect — flagged with `#[deprecated]` so callers can
//! see the migration target.

pub mod proto;
pub mod protocol;
