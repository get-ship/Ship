//! Sidecar-backed implementations of the substrate runtime traits.
//!
//! The substrate defines:
//!   - [`ship_substrate::supervisor::ActorRuntime`] — produces actors
//!   - [`ship_substrate::supervisor::ChildBackend`] — services the
//!     control surface (write_input / stop / pid / resize)
//!
//! In-process implementations of both live in substrate today
//! ([`PtyChildBackend`] / [`HeadlessChildBackend`] /
//! [`InProcessActorRuntime`]). The two impls in this module are the
//! sidecar-backed counterparts: they speak `ship.sidecar.v1` over
//! ConnectRPC to an in-sandbox runtime instead of forking locally.
//!
//! [`SidecarActorRuntime::new`] takes the sidecar's endpoint URL.
//! When the orchestrator (Chunk C3) drives spawn through a
//! `SandboxRegistry`-resolved `Arc<dyn ActorRuntime>`, sandboxes
//! whose runtime endpoint is `RuntimeEndpoint::Remote { addr, .. }`
//! get one of these. Tier 2/3 sandboxes use this path; Tier 0/1
//! keeps using [`InProcessActorRuntime`].
//!
//! The PtyChildBackend / HeadlessChildBackend / InProcessActorRuntime
//! types are unchanged — sidecar simply slots in alongside.

pub mod sidecar_backend;
pub mod sidecar_runtime;

pub use sidecar_backend::SidecarChildBackend;
pub use sidecar_runtime::SidecarActorRuntime;
