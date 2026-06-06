//! Post-step diff gate — daemon-side glue for `ship-pipelines::gate`.
//!
//! The pure logic (rule compilation, evaluation, validation) lives in
//! `ship-pipelines` so cloud workers and the browser embed can reuse it.
//! This file keeps the daemon-only concern: computing the changed-paths input
//! that `evaluate` needs via `ship_substrate::git::diff_paths`.

// Re-export so existing callers (`gate_rules::evaluate`, `gate_rules::validate`,
// `gate_rules::GateOutcome`) keep working without changes.
pub use ship_pipelines::gate::{evaluate, validate, GateOutcome};

/// Compute changed file paths between two SHAs (inclusive of everything the
/// step committed). Returns empty if git fails — the caller's `require_changed`
/// rules will then naturally fail with an empty path list.
pub fn diff_paths(work_dir: &std::path::Path, start_sha: &str) -> Vec<String> {
    ship_substrate::git::diff_paths(work_dir, start_sha)
}
