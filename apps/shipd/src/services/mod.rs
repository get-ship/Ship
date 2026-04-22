//! Domain services — the daemon's HTTP surface as consumed by first-party
//! views, CLI, and any third-party view. One module per domain, one service
//! per module, one `/api/<domain>/*` namespace each. Internal `files` helper
//! is shared across domains that persist to disk.
//!
//! Canonical services architecture doc: `.artifacts/daemon-services.md`.

pub mod files;
pub mod jsonc;
pub mod path_tolerant;
pub mod registry;
pub mod ship_ns;
pub mod agents;
pub mod pipelines;
pub mod skills;
pub mod rules;
pub mod mcp;
pub mod permissions;
pub mod git;
