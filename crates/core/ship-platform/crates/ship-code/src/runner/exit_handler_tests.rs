//! Vocabulary-coverage tests for the parameterized exit handler.
//!
//! The per-vocabulary divergence is the pure [`super::build_completed`] /
//! [`super::build_failed`] mapping; these assert it for BOTH the pipeline-run
//! and agent-session event sets so the one L0 mechanism cannot silently drift
//! one side's event type, entity key, payload shape, or owning actor.

use super::*;
use crate::dispatch_ports::{ExitCause, ExitSignal};
use ship_substrate::events::agent_session::event_types as agent_session_event_types;
use ship_substrate::events::pipeline::event_types as pipeline_event_types;

/// The completed-text differs by vocabulary; the failed-text per-cause too.
/// These pure-function checks pin BOTH vocabularies so the one mechanism
/// can't silently drift one side's wording or stage classifier.
#[test]
fn failed_message_covers_both_vocabularies() {
    assert_eq!(
        failed_message(&ExitVocabulary::PipelineRun, 7),
        "process exited with code 7"
    );
    assert_eq!(
        failed_message(&ExitVocabulary::AgentSession { session_id: "s1".into() }, 7),
        "harness exited with code 7"
    );
}

#[test]
fn no_code_message_covers_both_vocabularies() {
    assert_eq!(
        no_code_message(&ExitVocabulary::PipelineRun),
        "agent process exited with no captured exit code"
    );
    assert_eq!(
        no_code_message(&ExitVocabulary::AgentSession { session_id: "s1".into() }),
        "harness exited with no captured exit code"
    );
}

#[test]
fn session_destroyed_message_covers_both_vocabularies() {
    assert_eq!(
        session_destroyed_message(&ExitVocabulary::PipelineRun),
        "executor session destroyed externally (operator kill or substrate failure)"
    );
    assert_eq!(
        session_destroyed_message(&ExitVocabulary::AgentSession { session_id: "s1".into() }),
        "executor session destroyed externally"
    );
}

fn pipeline_signal(code: Option<i32>) -> ExitSignal {
    ExitSignal {
        slug: "demo".into(),
        pipeline_run_id: "run-1".into(),
        exit_code: code,
        cause: ExitCause::AgentExited,
    }
}

fn session_signal(code: Option<i32>) -> ExitSignal {
    // The bare-session route reuses pipeline_run_id as the routing ulid;
    // the terminal event must still key on the carried session_id.
    ExitSignal {
        slug: "demo".into(),
        pipeline_run_id: "routing-ulid".into(),
        exit_code: code,
        cause: ExitCause::AgentExited,
    }
}

// ── Pipeline vocabulary ──────────────────────────────────────────────────

#[test]
fn pipeline_completed_keys_on_run_id() {
    let t = build_completed(&ExitVocabulary::PipelineRun, &pipeline_signal(Some(0)));
    assert_eq!(t.event_type, pipeline_event_types::PIPELINE_RUN_COMPLETED);
    assert_eq!(t.entity_id, "run-1");
    assert_eq!(t.actor_id, "service.exit-handler");
    assert_eq!(t.payload["pipeline_run_id"], "run-1");
    assert_eq!(t.payload["slug"], "demo");
}

#[test]
fn pipeline_failed_nonzero_keys_on_run_id() {
    let sig = pipeline_signal(Some(3));
    let t = build_failed(
        &ExitVocabulary::PipelineRun,
        &sig,
        &failed_message(&ExitVocabulary::PipelineRun, 3),
        Some(3),
        "exit_nonzero",
    );
    assert_eq!(t.event_type, pipeline_event_types::PIPELINE_RUN_FAILED);
    assert_eq!(t.entity_id, "run-1");
    assert_eq!(t.actor_id, "service.exit-handler");
    assert_eq!(t.payload["pipeline_run_id"], "run-1");
    assert_eq!(t.payload["error"], "process exited with code 3");
    // The pipeline failed payload carries no stage/exit_code fields.
    assert!(t.payload.get("stage").is_none());
}

// ── Agent-session vocabulary ─────────────────────────────────────────────

#[test]
fn agent_session_completed_keys_on_session_id() {
    let vocab = ExitVocabulary::AgentSession { session_id: "sess-9".into() };
    let t = build_completed(&vocab, &session_signal(Some(0)));
    assert_eq!(t.event_type, agent_session_event_types::AGENT_SESSION_COMPLETED);
    // Keyed on session_id, NOT the routing ulid in pipeline_run_id.
    assert_eq!(t.entity_id, "sess-9");
    assert_eq!(t.actor_id, "service.agent-session-dispatch");
    assert_eq!(t.payload["session_id"], "sess-9");
    assert_eq!(t.payload["slug"], "demo");
    assert_eq!(t.payload["exit_code"], 0);
}

#[test]
fn agent_session_failed_nonzero_keys_on_session_id() {
    let vocab = ExitVocabulary::AgentSession { session_id: "sess-9".into() };
    let sig = session_signal(Some(2));
    let t = build_failed(&vocab, &sig, &failed_message(&vocab, 2), Some(2), "exit_nonzero");
    assert_eq!(t.event_type, agent_session_event_types::AGENT_SESSION_FAILED);
    assert_eq!(t.entity_id, "sess-9");
    assert_eq!(t.actor_id, "service.agent-session-dispatch");
    assert_eq!(t.payload["error"], "harness exited with code 2");
    assert_eq!(t.payload["exit_code"], 2);
    assert_eq!(t.payload["stage"], "exit_nonzero");
}

#[test]
fn agent_session_no_code_is_failure_not_completion() {
    // The historic pipeline bug emitted completed on a None exit code; the
    // session vocabulary must classify a missing code as failed too.
    let vocab = ExitVocabulary::AgentSession { session_id: "sess-9".into() };
    let t = build_failed(
        &vocab,
        &session_signal(None),
        no_code_message(&vocab),
        None,
        "exit_no_code",
    );
    assert_eq!(t.event_type, agent_session_event_types::AGENT_SESSION_FAILED);
    assert_eq!(t.payload["error"], "harness exited with no captured exit code");
    assert_eq!(t.payload["stage"], "exit_no_code");
}
