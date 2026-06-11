//! View serving — resolves and serves view assets from project and global dirs.
//!
//! Supports HTML views (index.html) and web component views (JS bundles with
//! optional manifests). Resolution checks project `.ship/views/` first, then
//! falls back to global `~/.ship/views/`.
//!
//! ## SSE + state submodules
//!
//! The `stream` and `state` submodules carry the Datastar SSE plumbing
//! used by SUI views and agent-canvas view sessions.
//!
//! - [`stream`] — `ViewStream` helper wrapping Datastar's `PatchElements` /
//!   `PatchSignals` with an `mpsc`-backed SSE response.
//! - [`state`] — read helpers that centralize runtime state access for views.

pub mod stream;
pub mod state;
// Daemon-coupled SUI handlers — still here because they reach into
// ApiState, services::registry, auth, and scopes (all daemon-internal).
// Moving them needs a `ship-runtime` extraction first; tracked separately.
// Truly framework-level pieces (view_payload, view_host, asset bundles,
// resolver, render_helpers, bundle, runtime asset) live in
// crates/deck/.
pub mod session;
pub mod session_types;
pub mod sanitize;
pub mod discovery;
pub mod resolve;

#[cfg(test)]
#[allow(unused_imports)]
use std::path::PathBuf;





