//! Domain services — the daemon's HTTP surface as consumed by first-party
//! views, CLI, and any third-party view. One module per domain, one service
//! per module, one `/api/<domain>/*` namespace each. Internal `files` helper
//! is shared across domains that persist to disk.
//!
//! Canonical services architecture doc: `.artifacts/daemon-services.md`.

// FilesService relocated to ship_platform::services::files (Phase 2).
// `jsonc` keeps the agent-config-specific helpers (parse_file, resolve_array_field,
// first_letter); strip_jsonc delegates to ship_substrate::fs_util::strip_jsonc_comments.

pub mod command;
pub mod launcher;
pub mod registry;
pub mod schema;
pub mod ship_ns;
pub mod pipelines;
pub mod specs;
pub mod settings;
pub mod chat;
