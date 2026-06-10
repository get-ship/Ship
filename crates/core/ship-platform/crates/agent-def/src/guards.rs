//! Single source of truth (SSOT) for the no-driver-edges invariant.
//!
//! The canonical event families ([`crate::canonical_session`] and
//! [`crate::op`]) are deliberately runtime-agnostic: their payloads carry only
//! session-shaped primitives — NEVER "driver edges" (the process/worktree
//! handles a *driver* pulls in). That is the whole point of the canonical
//! tier: the kernel, projections, and observers consume one vocabulary without
//! knowing which runtime produced it.
//!
//! [`DRIVER_EDGE_FIELDS`] is the ONE place that names the forbidden set. Every
//! guard test references it; none keeps its own hand-copied array. A prior
//! regression weakened the invariant by deleting `"branch"` from one hand-copied
//! list (the `op.rs` copy had drifted to only `["pid","worktree","exit_code"]`).
//! Centralizing the set makes that class of drift impossible: shrink the set in
//! one spot and EVERY guard shrinks with it — and the structural full-coverage
//! test below still asserts the complete shared const against every canonical
//! payload — so a field can only be removed deliberately, here, in plain sight.

/// Driver-edge field names that canonical payloads MUST NOT carry.
///
/// "Driver edges" are the runtime/process handles a driver pulls in:
/// - `pid`        — OS process id
/// - `branch`     — git branch the driver checked out
/// - `worktree`   — driver worktree path
/// - `exit_code`  — OS process exit status
///
/// The canonical families re-express session lifecycle without any of these so
/// the kernel/projections/observers stay runtime-agnostic. This is the SSOT;
/// do not copy the literals — reference this const.
///
/// **`dispatch.*` is the ONE sanctioned driver-edge carrier** (#26 spawn-intent
/// channel). `DispatchRequestedPayload` intentionally carries `branch`,
/// `source_spec_path`, and `baseline_sha`. It is EXEMPT from the
/// `all_canonical_payloads_carry_no_driver_edges` guard — do NOT add it to that
/// test. The guard covers `session.*` and `op.*` only.
pub const DRIVER_EDGE_FIELDS: &[&str] = &["pid", "branch", "worktree", "exit_code"];

#[cfg(test)]
mod tests {
    use super::DRIVER_EDGE_FIELDS;
    use crate::canonical_session::{
        SessionCancelled, SessionCompleted, SessionFailed, SessionStarted,
        SessionSuspended, TurnEnded, TurnStarted,
    };
    use crate::op::{OpToolUseCompleted, OpToolUseDenied, OpToolUseStarted};

    /// Assert a serialized payload's JSON contains NONE of the SSOT
    /// driver-edge field names.
    fn assert_no_driver_edges(name: &str, json: &str) {
        for field in DRIVER_EDGE_FIELDS {
            assert!(
                !json.contains(field),
                "{name} payload must not carry driver edge `{field}`: {json}"
            );
        }
    }

    /// Structural full-coverage guard for the no-driver-edges invariant.
    ///
    /// Serializes EVERY canonical session payload constructor AND every
    /// `op.*` payload constructor and asserts the serialized JSON carries
    /// NONE of [`DRIVER_EDGE_FIELDS`]. Driven entirely by the one shared
    /// const, so (a) adding a new payload that leaks a driver edge and
    /// (b) shrinking the forbidden set in any single guard copy are both
    /// caught here — neither can quietly weaken the invariant.
    ///
    /// Every optional field is populated (`Some(..)`) so its field name
    /// actually appears in the JSON: an optional driver-edge field that
    /// only serializes when set still gets exercised.
    #[test]
    fn all_canonical_payloads_carry_no_driver_edges() {
        let cases: Vec<(&str, serde_json::Value)> = vec![
            // ── canonical_session family (7) ───────────────────────────────
            (
                "SessionStarted",
                serde_json::to_value(SessionStarted {
                    session_id: "s1".into(),
                    agent_id: "claude".into(),
                    harness: "claude".into(),
                    model: Some("sonnet".into()),
                    space_id: "sp".into(),
                    slug: Some("my-agent-abc12345".into()),
                    input: Some("hi".into()),
                    parent: Some("parent-1".into()),
                })
                .unwrap(),
            ),
            (
                "SessionCompleted",
                serde_json::to_value(SessionCompleted { session_id: "s1".into() }).unwrap(),
            ),
            (
                "SessionFailed",
                serde_json::to_value(SessionFailed {
                    session_id: "s1".into(),
                    reason: "harness exit 1".into(),
                })
                .unwrap(),
            ),
            (
                "SessionCancelled",
                serde_json::to_value(SessionCancelled {
                    session_id: "s1".into(),
                    reason: Some("user pressed stop".into()),
                })
                .unwrap(),
            ),
            (
                "SessionSuspended",
                serde_json::to_value(SessionSuspended {
                    session_id: "s1".into(),
                    checkpoint: "ckpt-abc".into(),
                })
                .unwrap(),
            ),
            (
                "TurnStarted",
                serde_json::to_value(TurnStarted { session_id: "s1".into(), idx: 0 }).unwrap(),
            ),
            (
                "TurnEnded",
                serde_json::to_value(TurnEnded {
                    session_id: "s1".into(),
                    idx: 0,
                    stop_reason: Some("end_turn".into()),
                    ..Default::default()
                })
                .unwrap(),
            ),
            // ── op.* family (3) ────────────────────────────────────────────
            (
                "OpToolUseStarted",
                serde_json::to_value(OpToolUseStarted {
                    session_id: "s1".into(),
                    harness_session_id: Some("ses_xyz".into()),
                    tool: "Bash".into(),
                    input: Some(serde_json::json!({"command": "ls -la"})),
                    blast: Some("read".into()),
                    started_at: "2026-06-09T10:00:00+00:00".into(),
                })
                .unwrap(),
            ),
            (
                "OpToolUseCompleted",
                serde_json::to_value(OpToolUseCompleted {
                    session_id: "s1".into(),
                    harness_session_id: Some("ses_xyz".into()),
                    tool: "Read".into(),
                    status: "ok".into(),
                    error: Some("none".into()),
                    response: Some(serde_json::json!({"content": "file body"})),
                    ended_at: "2026-06-09T10:00:01+00:00".into(),
                })
                .unwrap(),
            ),
            (
                "OpToolUseDenied",
                serde_json::to_value(OpToolUseDenied {
                    session_id: "s1".into(),
                    harness_session_id: Some("ses_xyz".into()),
                    tool: "Bash".into(),
                    input: Some(serde_json::json!({"command": "rm -rf /"})),
                    reason: "ship-gate: blocked destructive command".into(),
                    denied_by: "ship-gate".into(),
                    denied_at: "2026-06-09T10:00:00+00:00".into(),
                })
                .unwrap(),
            ),
        ];

        for (name, value) in cases {
            assert_no_driver_edges(name, &value.to_string());
        }
    }
}
