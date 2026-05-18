//! Host-side network mediation for sandboxed containers.
//!
//! A small HTTP CONNECT proxy that listens on a TCP port and enforces a
//! hostname allow / deny list before splicing client ↔ upstream bytes.
//! The container points `HTTPS_PROXY` and `HTTP_PROXY` at this listener.
//!
//! Scope is deliberately minimal:
//!   - HTTP CONNECT only (sufficient for HTTPS).
//!   - Hostname-string match (exact or `*.suffix` glob); no SNI inspect.
//!   - No TLS termination — host header in the CONNECT line is the
//!     policy surface. Mutual-TLS, MITM, body inspection are out of scope.
//!
//! Pair with a sidecar process that's already plumbed to inherit
//! `HTTPS_PROXY`; see `SidecarSandbox`.

pub mod proxy;

pub use proxy::{HostMatcher, ProxyDecision, ProxyHandle, spawn_allowlist_proxy};
