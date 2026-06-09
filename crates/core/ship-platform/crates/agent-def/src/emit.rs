//! Canonical session event-emit helpers — shared by both runtimes (Floki,
//! ship-code). The whole point of the canonical family: BOTH runtimes call
//! these builders; BOTH produce identical, ship-shaped envelopes; the
//! kernel, projections, and observers see one event vocabulary.
//!
//! Design:
//! - [`SessionEventSink`] is the kernel-emit port. Production code wraps a
//!   real `Arc<dyn ship_kernel::EventSink>` — typically
//!   `ship_runner::KernelEventSink`, which already implements `EventSink`
//!   by delegating to `persist_and_route` (the shared runner helper, NOT
//!   reinvented here). Tests wrap [`ship_kernel::testing::MockSink`],
//!   which also implements `EventSink`. The port is the same in both — no
//!   `if kernel else mock` branching at call sites.
//! - The `emit_session_*` / `emit_turn_*` builders take the typed payload
//!   from [`ship_substrate::events::canonical_session`], build an
//!   `EventEnvelope`, stamp the scope (`Space` for `session.*` coarse
//!   events, `Runtime` for `session.turn.*`), then call
//!   `sink.persist(&envelope)`.
//!
//! Scope (jobs `canonical-session-events` + `workbench-teardown`): the
//! COMPLETE canonical agent-session family — 5 coarse-grain events
//! (`session.started`, `session.completed`, `session.failed`,
//! `session.cancelled`, `session.suspended`) + 2 medium-grain turn events
//! (`session.turn.started`, `session.turn.ended`). The workbench
//! `session.*` family is RETIRED (R6); the canonical `session.started`
//! adopted in `workbench-teardown` is now the only session-start emit.
//!
//! Spec: `.artifacts/agents-architecture.md` (R1–R11).
use std::sync::Arc;
use serde::Serialize;
use ship_kernel::EventSink;

// Re-export canonical payload types so downstream crates (e.g. floki) can
// build session event payloads through agent-def without adding ship-substrate
// as a direct runtime dependency.
pub use crate::canonical_session::{
    SessionCancelled, SessionCompleted, SessionFailed, SessionStarted,
    SessionSuspended, TurnEnded, TurnStarted,
};
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use ship_types::event_types::canonical_session as event_types;

/// Kernel-emit port for the canonical session family (7 events — see
/// module docs for the slice scope).
///
/// Production: wrap a kernel-backed `Arc<dyn EventSink>` (e.g.
/// `ship_runner::KernelEventSink`). The sink persists the canonical
/// envelope; routing through the kernel happens inside the kernel-backed
/// sink's implementation (which delegates to `persist_and_route`).
///
/// Tests: wrap [`ship_kernel::testing::MockSink`] — `MockSink` implements
/// `EventSink`, so the same `SessionEventSink` works.
#[derive(Clone)]
pub struct SessionEventSink {
    inner: Arc<dyn EventSink>,
}

impl SessionEventSink {
    /// Wrap an existing `EventSink` (kernel-backed or test mock).
    pub fn new(sink: Arc<dyn EventSink>) -> Self {
        Self { inner: sink }
    }

    /// Borrow the wrapped sink — for tests asserting on the captured
    /// events. Callers typically downcast to
    /// `ship_kernel::testing::MockSink` to read `captured()`.
    pub fn inner(&self) -> &Arc<dyn EventSink> {
        &self.inner
    }
    /// Build + emit a `session.started` envelope. Space-scoped, Durable.
    /// Adopted in `workbench-teardown`: the canonical family is now
    /// complete (started + 4 terminals + 2 turn).
    pub fn emit_session_started(
        &self,
        session_id: &str,
        payload: &SessionStarted,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_with_space(
            &self.inner,
            event_types::SESSION_STARTED,
            session_id,
            payload,
        )
    }

    /// Build + emit a `session.completed` envelope. Space-scoped, Durable.
    pub fn emit_session_completed(
        &self,
        session_id: &str,
        payload: &SessionCompleted,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_with_space(
            &self.inner,
            event_types::SESSION_COMPLETED,
            session_id,
            payload,
        )
    }

    /// Build + emit a `session.failed` envelope. Space-scoped, Durable.
    pub fn emit_session_failed(
        &self,
        session_id: &str,
        payload: &SessionFailed,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_with_space(
            &self.inner,
            event_types::SESSION_FAILED,
            session_id,
            payload,
        )
    }

    /// Build + emit a `session.cancelled` envelope. Space-scoped, Durable.
    pub fn emit_session_cancelled(
        &self,
        session_id: &str,
        payload: &SessionCancelled,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_with_space(
            &self.inner,
            event_types::SESSION_CANCELLED,
            session_id,
            payload,
        )
    }

    /// Build + emit a `session.suspended` envelope. Space-scoped, Durable.
    pub fn emit_session_suspended(
        &self,
        session_id: &str,
        payload: &SessionSuspended,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_with_space(
            &self.inner,
            event_types::SESSION_SUSPENDED,
            session_id,
            payload,
        )
    }

    /// Build + emit a `session.turn.started` envelope. Runtime-scoped,
    /// Ephemeral.
    pub fn emit_turn_started(
        &self,
        session_id: &str,
        payload: &TurnStarted,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_runtime(
            &self.inner,
            event_types::TURN_STARTED,
            session_id,
            payload,
        )
    }

    /// Build + emit a `session.turn.ended` envelope. Runtime-scoped,
    /// Ephemeral.
    pub fn emit_turn_ended(
        &self,
        session_id: &str,
        payload: &TurnEnded,
    ) -> Result<EventEnvelope, serde_json::Error> {
        emit_runtime(
            &self.inner,
            event_types::TURN_ENDED,
            session_id,
            payload,
        )
    }
}

// ── Internal envelope-build helpers ────────────────────────────────────────

fn emit_with_space<P: Serialize>(
    sink: &Arc<dyn EventSink>,
    event_type: &str,
    entity_id: &str,
    payload: &P,
) -> Result<EventEnvelope, serde_json::Error> {
    let envelope = build_space_envelope(event_type, entity_id, payload)?;
    // The MockSink / KernelEventSink are infallible at this seam; bubble
    // any other sink's error as a panic because emit-side failures are
    // unrecoverable here (the kernel-emit port contract is fire-and-forget).
    sink.persist(&envelope)
        .expect("EventSink::persist errored — kernel-emit port must be infallible at the test seam (see MockSink)");
    Ok(envelope)
}

fn emit_runtime<P: Serialize>(
    sink: &Arc<dyn EventSink>,
    event_type: &str,
    entity_id: &str,
    payload: &P,
) -> Result<EventEnvelope, serde_json::Error> {
    let envelope = build_runtime_envelope(event_type, entity_id, payload)?;
    sink.persist(&envelope)
        .expect("EventSink::persist errored — kernel-emit port must be infallible at the test seam (see MockSink)");
    Ok(envelope)
}

fn build_space_envelope<P: Serialize>(
    event_type: &str,
    entity_id: &str,
    payload: &P,
) -> Result<EventEnvelope, serde_json::Error> {
    // Session-coarse events (session.completed, etc.) ride Space.
    // We use the session_id itself as the scope_key — the canonical
    // payloads for these terminals only carry a session_id, and per the
    // substrate's `enforce_scope` the scope_key is required for Space
    // events. Using the session_id lets the (Space, scope_key) be unique
    // per session; the Space-level read-back keys off the session.
    let mut envelope = EventEnvelope::new(event_type, entity_id, payload)?;
    envelope.scope = EventScope::Space;
    envelope.scope_key = Some(entity_id.to_string());
    envelope.session_id = Some(entity_id.to_string());
    Ok(envelope)
}

fn build_runtime_envelope<P: Serialize>(
    event_type: &str,
    entity_id: &str,
    payload: &P,
) -> Result<EventEnvelope, serde_json::Error> {
    let mut envelope = EventEnvelope::new(event_type, entity_id, payload)?;
    // EventScope resolved from the registry (Runtime for session.turn.*).
    // session_id carries the session, but scope_key is left None — turn
    // events are per-machine, not per-space.
    envelope.session_id = Some(entity_id.to_string());
    Ok(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_kernel::testing::MockSink;
    use ship_types::event_registry::{EventScope, Retention};

    fn fresh_sink() -> (Arc<MockSink>, SessionEventSink) {
        let mock = Arc::new(MockSink::new());
        let session_sink = SessionEventSink::new(mock.clone());
        (mock, session_sink)
    }

    fn assert_one_event(
        mock: &Arc<MockSink>,
        expected_type: &str,
        expected_scope: EventScope,
        expected_retention: Retention,
        expected_entity_id: &str,
    ) {
        let events = mock.captured();
        assert_eq!(events.len(), 1, "expected exactly one captured event");
        let e = &events[0];
        assert_eq!(e.event_type, expected_type);
        assert_eq!(e.scope, expected_scope);
        assert_eq!(e.entity_id, expected_entity_id);
        // scope/retention must agree with the registry
        let def = ship_types::event_registry::lookup(expected_type)
            .expect("event must be registered");
        assert_eq!(def.retention, expected_retention);
    }

    #[test]
    fn session_started_emits_space_scoped_durable_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = SessionStarted {
            session_id: "s1".into(),
            agent_id: "claude".into(),
            harness: "claude".into(),
            model: Some("opus-4".into()),
            space_id: "ws-1".into(),
            input: Some("first turn".into()),
            parent: None,
        };
        let env = sink
            .emit_session_started("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.started");
        assert_eq!(env.scope, EventScope::Space);
        assert_eq!(env.scope_key.as_deref(), Some("s1"));
        let back: SessionStarted = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(&mock, "session.started", EventScope::Space, Retention::Durable, "s1");
    }

    #[test]
    fn session_completed_emits_space_scoped_durable_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = SessionCompleted { session_id: "s1".into() };
        let env = sink
            .emit_session_completed("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.completed");
        assert_eq!(env.scope, EventScope::Space);
        assert_eq!(env.scope_key.as_deref(), Some("s1"));
        let back: SessionCompleted = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(&mock, "session.completed", EventScope::Space, Retention::Durable, "s1");
    }

    #[test]
    fn session_failed_emits_space_scoped_durable_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = SessionFailed {
            session_id: "s1".into(),
            reason: "harness exit 1".into(),
        };
        let env = sink
            .emit_session_failed("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.failed");
        assert_eq!(env.scope, EventScope::Space);
        let back: SessionFailed = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(&mock, "session.failed", EventScope::Space, Retention::Durable, "s1");
    }

    #[test]
    fn session_cancelled_emits_space_scoped_durable_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = SessionCancelled {
            session_id: "s1".into(),
            reason: Some("user pressed stop".into()),
        };
        let env = sink
            .emit_session_cancelled("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.cancelled");
        assert_eq!(env.scope, EventScope::Space);
        let back: SessionCancelled = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(&mock, "session.cancelled", EventScope::Space, Retention::Durable, "s1");
    }

    #[test]
    fn session_suspended_emits_space_scoped_durable_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = SessionSuspended {
            session_id: "s1".into(),
            checkpoint: "ckpt-7".into(),
        };
        let env = sink
            .emit_session_suspended("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.suspended");
        assert_eq!(env.scope, EventScope::Space);
        let back: SessionSuspended = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(&mock, "session.suspended", EventScope::Space, Retention::Durable, "s1");
    }

    #[test]
    fn turn_started_emits_runtime_scoped_ephemeral_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = TurnStarted { session_id: "s1".into(), idx: 0 };
        let env = sink
            .emit_turn_started("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.turn.started");
        // Runtime scope is the registry-resolved primary for session.turn.*
        assert_eq!(env.scope, EventScope::Runtime);
        assert_eq!(env.session_id.as_deref(), Some("s1"));
        let back: TurnStarted = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(
            &mock,
            "session.turn.started",
            EventScope::Runtime,
            Retention::Ephemeral,
            "s1",
        );
    }

    #[test]
    fn turn_ended_emits_runtime_scoped_ephemeral_envelope() {
        let (mock, sink) = fresh_sink();
        let payload = TurnEnded {
            session_id: "s1".into(),
            idx: 2,
            stop_reason: Some("end_turn".into()),
        };
        let env = sink
            .emit_turn_ended("s1", &payload)
            .expect("envelope builds");
        assert_eq!(env.event_type, "session.turn.ended");
        assert_eq!(env.scope, EventScope::Runtime);
        let back: TurnEnded = serde_json::from_str(&env.payload_json).unwrap();
        assert_eq!(back, payload);
        assert_one_event(
            &mock,
            "session.turn.ended",
            EventScope::Runtime,
            Retention::Ephemeral,
            "s1",
        );
    }

    #[test]
    fn multiple_emits_each_capture_independently() {
        let (mock, sink) = fresh_sink();
        sink.emit_session_completed(
            "s1",
            &SessionCompleted { session_id: "s1".into() },
        )
        .unwrap();
        sink.emit_session_failed(
            "s2",
            &SessionFailed {
                session_id: "s2".into(),
                reason: "boom".into(),
            },
        )
        .unwrap();
        sink.emit_turn_started("s3", &TurnStarted { session_id: "s3".into(), idx: 0 })
            .unwrap();
        let events = mock.captured();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event_type, "session.completed");
        assert_eq!(events[1].event_type, "session.failed");
        assert_eq!(events[2].event_type, "session.turn.started");
    }

    #[test]
    fn session_payloads_carry_no_driver_edges() {
        // Acceptance criterion: payloads carry NO pid/branch/worktree/exit_code.
        // Smoke-test all FIVE coarse-session payloads + the two turn payloads.
        let cases: Vec<(String, serde_json::Value)> = vec![
            (
                "session.started".into(),
                serde_json::to_value(SessionStarted {
                    session_id: "s".into(),
                    agent_id: "a".into(),
                    harness: "h".into(),
                    model: None,
                    space_id: "ws".into(),
                    input: None,
                    parent: None,
                })
                .unwrap(),
            ),
            (
                "session.completed".into(),
                serde_json::to_value(SessionCompleted { session_id: "s".into() }).unwrap(),
            ),
            (
                "session.failed".into(),
                serde_json::to_value(SessionFailed {
                    session_id: "s".into(),
                    reason: "x".into(),
                })
                .unwrap(),
            ),
            (
                "session.cancelled".into(),
                serde_json::to_value(SessionCancelled {
                    session_id: "s".into(),
                    reason: None,
                })
                .unwrap(),
            ),
            (
                "session.suspended".into(),
                serde_json::to_value(SessionSuspended {
                    session_id: "s".into(),
                    checkpoint: "c".into(),
                })
                .unwrap(),
            ),
            (
                "session.turn.started".into(),
                serde_json::to_value(TurnStarted { session_id: "s".into(), idx: 0 }).unwrap(),
            ),
            (
                "session.turn.ended".into(),
                serde_json::to_value(TurnEnded {
                    session_id: "s".into(),
                    idx: 0,
                    stop_reason: None,
                })
                .unwrap(),
            ),
        ];
        for (name, v) in cases {
            for forbidden in ["pid", "branch", "worktree", "exit_code"] {
                assert!(
                    !v.to_string().contains(forbidden),
                    "{name} payload must not carry {forbidden}: {v}"
                );
            }
        }
    }
}
