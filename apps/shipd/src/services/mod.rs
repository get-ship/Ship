//! Domain services — the daemon's HTTP surface as consumed by first-party
//! views, CLI, and any third-party view. One module per domain, one service
//! per module, one `/api/<domain>/*` namespace each. Internal `files` helper
//! is shared across domains that persist to disk.
//!
//! Canonical services architecture doc: `.artifacts/daemon-services.md`.

pub mod files;
pub mod agents;
pub mod pipelines;
