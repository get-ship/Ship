//! Sidecar protocol — host ↔ in-sandbox daemon wire format.
//!
//! The sidecar is the daemon that runs *inside* a sandbox (container,
//! microVM, or cloud box) and accepts policy + exec + lifecycle commands
//! over HTTP+SSE. The host-side isolator talks to it the same way
//! regardless of where the sandbox physically lives.
//!
//! See `docs/sidecar-protocol.md` (in this crate) for the wire contract.
//! Modeled on `rivet-dev/sandbox-agent`'s OpenAPI surface.

pub mod protocol;
