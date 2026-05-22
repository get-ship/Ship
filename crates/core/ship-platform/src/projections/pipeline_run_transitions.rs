//! Status transition table for job events.
//!
//! The single place that says "which events change `PipelineRunStatus`, and to
//! what." Payload extraction stays in [`super::pipeline_run::apply`]. This module
//! only answers: given an event type, what is the new status?
//!
//! Events not listed here either don't mutate status, or the transition
//! is conditional on payload (e.g. `PIPELINE_STEP_OBSERVED`). Those stay inline
//! in `apply` until a follow-up pass extracts them too.

use super::pipeline_run::PipelineRunStatus;
use ship_substrate::events::pipeline::event_types;

/// Unconditional status transition for an event type.
///
/// Returns `None` if the event does not mutate status, or if the
/// transition depends on payload. Callers that care about payload-driven
/// transitions (PIPELINE_STEP_OBSERVED, PIPELINE_STEP_COMPLETED) handle them separately.
pub fn status_for(event_type: &str) -> Option<PipelineRunStatus> {
    match event_type {
        event_types::PIPELINE_RUN_DISPATCHED => Some(PipelineRunStatus::Dispatched),
        event_types::PIPELINE_GATE_REQUESTED => Some(PipelineRunStatus::GatePending),
        event_types::PIPELINE_GATE_PASSED => Some(PipelineRunStatus::Pending),
        event_types::PIPELINE_GATE_FAILED => Some(PipelineRunStatus::Failed),
        event_types::PIPELINE_RUN_COMPLETED => Some(PipelineRunStatus::Completed),
        event_types::PIPELINE_RUN_BLOCKED => Some(PipelineRunStatus::Blocked),
        event_types::PIPELINE_RUN_MERGED => Some(PipelineRunStatus::Merged),
        event_types::PIPELINE_RUN_FAILED => Some(PipelineRunStatus::Failed),
        event_types::PIPELINE_STEP_SPAWN_FAILED => Some(PipelineRunStatus::Failed),
        event_types::PIPELINE_RUN_CANCELLED => Some(PipelineRunStatus::Cancelled),
        event_types::PIPELINE_STEP_STARTED => Some(PipelineRunStatus::Dispatched),
        event_types::PIPELINE_STEP_FAILED => Some(PipelineRunStatus::Failed),
        event_types::PIPELINE_STEP_RETRY => Some(PipelineRunStatus::Dispatched),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_dispatched_sets_dispatched() {
        assert_eq!(status_for(event_types::PIPELINE_RUN_DISPATCHED), Some(PipelineRunStatus::Dispatched));
    }

    #[test]
    fn run_gate_requested_sets_gate_pending() {
        assert_eq!(status_for(event_types::PIPELINE_GATE_REQUESTED), Some(PipelineRunStatus::GatePending));
    }

    #[test]
    fn run_gate_passed_sets_pending() {
        assert_eq!(status_for(event_types::PIPELINE_GATE_PASSED), Some(PipelineRunStatus::Pending));
    }

    #[test]
    fn run_gate_failed_sets_failed() {
        assert_eq!(status_for(event_types::PIPELINE_GATE_FAILED), Some(PipelineRunStatus::Failed));
    }

    #[test]
    fn run_completed_sets_completed() {
        assert_eq!(status_for(event_types::PIPELINE_RUN_COMPLETED), Some(PipelineRunStatus::Completed));
    }

    #[test]
    fn job_blocked_sets_blocked() {
        assert_eq!(status_for(event_types::PIPELINE_RUN_BLOCKED), Some(PipelineRunStatus::Blocked));
    }

    #[test]
    fn run_merged_sets_merged() {
        assert_eq!(status_for(event_types::PIPELINE_RUN_MERGED), Some(PipelineRunStatus::Merged));
    }

    #[test]
    fn run_failed_sets_failed() {
        assert_eq!(status_for(event_types::PIPELINE_RUN_FAILED), Some(PipelineRunStatus::Failed));
    }

    #[test]
    fn step_dispatched_sets_dispatched() {
        assert_eq!(status_for(event_types::PIPELINE_STEP_STARTED), Some(PipelineRunStatus::Dispatched));
    }

    #[test]
    fn step_failed_sets_failed() {
        assert_eq!(status_for(event_types::PIPELINE_STEP_FAILED), Some(PipelineRunStatus::Failed));
    }

    #[test]
    fn job_spawn_failed_sets_failed() {
        assert_eq!(status_for(event_types::PIPELINE_STEP_SPAWN_FAILED), Some(PipelineRunStatus::Failed));
    }

    #[test]
    fn job_cancelled_sets_cancelled() {
        assert_eq!(status_for(event_types::PIPELINE_RUN_CANCELLED), Some(PipelineRunStatus::Cancelled));
    }

    #[test]
    fn step_retry_sets_dispatched() {
        assert_eq!(status_for(event_types::PIPELINE_STEP_RETRY), Some(PipelineRunStatus::Dispatched));
    }

    #[test]
    fn step_completed_is_none() {
        // PIPELINE_STEP_COMPLETED mutates completed_steps but not status unconditionally.
        assert_eq!(status_for(event_types::PIPELINE_STEP_COMPLETED), None);
    }

    #[test]
    fn step_observed_is_none() {
        // PIPELINE_STEP_OBSERVED's status mutation is payload-conditional (verdict).
        assert_eq!(status_for(event_types::PIPELINE_STEP_OBSERVED), None);
    }

    #[test]
    fn unknown_event_is_none() {
        assert_eq!(status_for("pipeline.never_heard_of_it"), None);
        assert_eq!(status_for(""), None);
    }
}
