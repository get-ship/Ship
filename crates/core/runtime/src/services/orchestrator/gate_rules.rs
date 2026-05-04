//! Post-step diff gate — daemon-side glue for `ship-pipelines::gate`.
//!
//! The pure logic (rule compilation, evaluation, validation) lives in
//! `ship-pipelines` so cloud workers and the browser embed can reuse it.
//! This file keeps the daemon-only concern: shelling out to `git diff` to
//! compute the changed-paths input that `evaluate` needs.

// Re-export so existing callers (`gate_rules::evaluate`, `gate_rules::validate`,
// `gate_rules::GateOutcome`) keep working without changes.
pub use ship_pipelines::gate::{evaluate, validate, GateOutcome};

/// Compute changed file paths between two SHAs (inclusive of everything the
/// step committed). Returns empty if git fails — the caller's `require_changed`
/// rules will then naturally fail with an empty path list.
pub fn diff_paths(work_dir: &std::path::Path, start_sha: &str) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["diff", "--name-only", &format!("{start_sha}..HEAD")])
        .current_dir(work_dir)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}
