//! Step executor — trait and executor-selection contract for pipeline steps.
//!
//! This module is the runtime-level statement of how a pipeline step is run.
//! It intentionally **reuses** the existing `SessionExecutor` trait rather than
//! introducing a parallel `StepExecutor` trait, because the orchestrator
//! already dispatches everything (single-step jobs and pipeline steps alike)
//! through `SessionExecutor` — a second trait would be a parallel system with no
//! added power (see `CLAUDE.md`: "One way to do things.").
//!
//! The two concrete implementations live in `apps/shipd/src/supervisor/
//! executors/`:
//!
//!   - `TmuxWindowExecutor` — tmux session `ship-<slug>`, one window per
//!                            step, command teed to
//!                            `.artifacts/pipeline/NN-<step>.log`. Default
//!                            path for `ship run`. Not a PTY — it
//!                            multiplexes tmux windows.
//!   - `ProcessExecutor`    — headless child process running under a real
//!                            PTY. Selected when the job's `headless`
//!                            flag is set, when tmux is unavailable, or on
//!                            Windows.
//!
//! The shipd-side `RoutingExecutor` selects between them per-job based on
//! `SessionContext::headless`.
//!
//! # Log path convention
//!
//! The tee command inside the tmux pane writes to
//! `.artifacts/pipeline/<NN>-<step>.log` (same directory as the per-step
//! spec bodies — see `pipeline_specs.rs`). `tee -a` means retry runs
//! append instead of truncating.

pub use ship_substrate::services::dispatch_ports::{SessionContext, SessionExecutor, StepInfo};

/// Relative log path for a given step in the pipeline log convention.
///
/// `.artifacts/pipeline/NN-<step>.log`, 2-digit zero-padded index.
pub fn step_log_path(index: usize, step_id: &str) -> String {
    format!(".artifacts/pipeline/{:02}-{}.log", index + 1, step_id)
}

/// Sanitize a slug into a tmux-safe session name suffix.
/// tmux rejects `.` and `:` in names; we map both to `_` and truncate to
/// 8 characters so the attach command is actually typable.
pub fn tmux_session_name(slug: &str) -> String {
    let sanitized: String = slug
        .chars()
        .map(|c| if c == '.' || c == ':' { '_' } else { c })
        .collect();
    let short: String = sanitized.chars().take(8).collect();
    format!("ship-{short}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_log_path_matches_pipeline_spec_convention() {
        assert_eq!(
            step_log_path(0, "test-writer"),
            ".artifacts/pipeline/01-test-writer.log"
        );
        assert_eq!(
            step_log_path(9, "impl"),
            ".artifacts/pipeline/10-impl.log"
        );
    }

    #[test]
    fn tmux_session_name_sanitizes_dots_and_colons() {
        let n = tmux_session_name("ab.cd:ef0123");
        assert!(!n.contains('.'));
        assert!(!n.contains(':'));
        assert_eq!(n, "ship-ab_cd_ef");
    }

    #[test]
    fn tmux_session_name_truncates_long_slugs() {
        let n = tmux_session_name("pipeline-abcdef123456");
        assert_eq!(n, "ship-pipeline");
        assert_eq!(n.len(), "ship-".len() + 8);
    }
}
