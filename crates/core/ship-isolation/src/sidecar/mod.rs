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
//! [`proto`] is the canonical v1 protocol — generated from
//! `proto/ship/sidecar/v1/sidecar.proto` via ConnectRPC. The
//! in-sandbox `bin/ship-sidecar/` binary serves it; the
//! host-side `backends::youki_sidecar` driver drives it via the
//! generated `ProcessesClient` / `EventsClient`.
//!
//! There is no legacy protocol module — the JSON shape that
//! preceded the cutover has been removed.

pub mod proto;
