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
//! ## v1 (canonical) vs legacy
//!
//! [`proto`] is the canonical v1 protocol — generated from
//! `proto/ship/sidecar/v1/sidecar.proto` via ConnectRPC. The
//! `bin/ship-sidecar/` binary speaks this. Every new host-side
//! caller should use the generated `ProcessesClient` / `EventsClient`
//! / `HandshakeClient`.
//!
//! [`protocol`] is the hand-typed JSON shape from before the
//! ConnectRPC cutover. Kept alive only for the host-side
//! `backends::youki_sidecar` driver, which still drives the
//! sidecar via reqwest + SSE. That backend's port to the v1
//! `ProcessesClient` is the next chunk and will let us delete
//! `protocol` entirely.

pub mod proto;
pub mod protocol;
