//! Canonical op.* event family — the L1 payloads home for the
//! tool-use op-card surface.
//!
//! R2 hook conversion (pending #25b): third-party-harness hook data WILL be
//! converted into canonical `op.*` events, with harness/provenance riding the
//! envelope instead of a separate `hook.*` family. Today only the op.*
//! contract is landed (payloads + registration); hook ingestion still emits
//! `hook.*` and nothing emits `op.*` yet. The op.* tier is the FINE-GRAINED
//! complement to
//! the coarse session.* / medium session.turn.* tiers (see
//! `crate::canonical_session`):
//!
//! - **session.* (coarse)** — Space-scoped, Durable
//! - **session.turn.* (medium)** — Runtime-scoped, Ephemeral
//! - **op.* (fine-grained)** — Runtime-scoped, Ephemeral
//!
//! Scope and retention are enforced by the `register_event!` calls
//! in `crate::event_registrations`. The `event_scope_discipline`
//! test (in `crate::tests`) checks the registry metadata at
//! compile time.
//!
//! Wire-shape: payloads are intentionally minimal so the op-card
//! surface can stream to the dashboard without re-parsing. The
//! actor that materializes an op-card picks the rich fields it
//! needs from the kernel's view of the canonical session.

use serde::{Deserialize, Serialize};

// ── Tool-use op-card ─────────────────────────────────────────────────────────

/// Fired BEFORE a tool invocation. The canonical "tool is about
/// to run" signal. Carries the structured arguments so analytics
/// can count "all bash commands" or "all file edits to
/// $WORKSPACE/auth" without re-parsing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpToolUseStarted {
    /// Ship-side session id (canonical).
    pub session_id: String,
    /// Harness-native session id when available (claude's session
    /// UUID, codex's thread id, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_session_id: Option<String>,
    /// Tool name as the harness reports it. Conventional set:
    /// "Bash", "Read", "Write", "Edit", "Grep", "Glob", "Task",
    /// "WebFetch", and any user-defined MCP tool name.
    pub tool: String,
    /// Structured tool input. Shape is tool-specific (Bash →
    /// {command, description?}; Edit → {file_path, old_string,
    /// new_string}). May be omitted when the harness doesn't
    /// surface inputs (rare).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Blast-radius summary (best-effort). `None` when the
    /// runtime can't classify before invocation. Drives the
    /// op-card's color/severity band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blast: Option<String>,
    /// ISO-8601 timestamp of the start observation.
    pub started_at: String,
}

/// Fired AFTER a tool invocation completes. Pairs with
/// [`OpToolUseStarted`] (oldest open `started` row for the same
/// `(session_id, tool)` pair is updated on completion).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpToolUseCompleted {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_session_id: Option<String>,
    pub tool: String,
    /// Coarse outcome classifier — `"ok"` / `"error"` / `"unknown"`.
    /// String, not enum, so harnesses can extend without breaking
    /// the schema.
    pub status: String,
    /// Error message when status = "error". `None` on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Structured tool response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
    /// ISO-8601 timestamp of the end observation.
    pub ended_at: String,
}

/// A tool call was REJECTED before invocation. Distinct from
/// `OpToolUseCompleted{status="error"}` because the call never
/// produced a started/completed pair — the tool never ran.
///
/// NOT yet emitted anywhere: the Ship-integrity gate's denials currently
/// surface as `hook.permission_denied` (hooks/http_ingress.rs). Wiring
/// `op.tool_use_denied` emission is part of the hook→op conversion (#25b);
/// operator-approval flows would later emit it with `denied_by="operator"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpToolUseDenied {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_session_id: Option<String>,
    /// Tool name the harness was about to invoke.
    pub tool: String,
    /// Structured tool input the gate inspected. Useful for
    /// forensics ("which Bash command did we block?"). Optional
    /// in case the harness didn't surface input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Operator-readable reason from the gate. Carries the exact
    /// rule message ("ship-gate: blocked write inside .git/ …").
    pub reason: String,
    /// Origin of the denial. Today always "ship-gate"; later
    /// "operator" (UI approval flow) or "harness" (provider-side
    /// permission check) once those wire up.
    pub denied_by: String,
    pub denied_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_tool_use_started_round_trips() {
        let s = OpToolUseStarted {
            session_id: "s1".into(),
            harness_session_id: Some("ses_xyz".into()),
            tool: "Bash".into(),
            input: Some(serde_json::json!({"command": "ls -la"})),
            blast: Some("read".into()),
            started_at: "2026-06-09T10:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: OpToolUseStarted = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn op_tool_use_started_minimal_required_fields() {
        // Older harnesses that don't surface input/blast can emit
        // a bare-minimum envelope. All optional fields must
        // serde-default cleanly.
        let json = r#"{
            "session_id": "s1",
            "tool": "Read",
            "started_at": "2026-06-09T10:00:00+00:00"
        }"#;
        let s: OpToolUseStarted = serde_json::from_str(json).unwrap();
        assert_eq!(s.tool, "Read");
        assert!(s.harness_session_id.is_none());
        assert!(s.input.is_none());
        assert!(s.blast.is_none());
    }

    #[test]
    fn op_tool_use_completed_round_trips() {
        let c = OpToolUseCompleted {
            session_id: "s1".into(),
            harness_session_id: None,
            tool: "Read".into(),
            status: "ok".into(),
            error: None,
            response: Some(serde_json::json!({"content": "file body"})),
            ended_at: "2026-06-09T10:00:01+00:00".into(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: OpToolUseCompleted = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn op_tool_use_denied_round_trips() {
        let d = OpToolUseDenied {
            session_id: "s1".into(),
            harness_session_id: None,
            tool: "Bash".into(),
            input: Some(serde_json::json!({"command": "rm -rf /"})),
            reason: "ship-gate: blocked destructive command".into(),
            denied_by: "ship-gate".into(),
            denied_at: "2026-06-09T10:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: OpToolUseDenied = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
        assert!(json.contains("\"denied_by\":\"ship-gate\""));
    }

    /// No driver-edge fields on any op.* payload — these are
    /// canonical envelopes.
    #[test]
    fn op_payloads_carry_no_driver_edges() {
        use crate::guards::DRIVER_EDGE_FIELDS;
        for (name, json) in [
            (
                "OpToolUseStarted",
                serde_json::to_value(OpToolUseStarted {
                    session_id: "s".into(),
                    harness_session_id: None,
                    tool: "Bash".into(),
                    input: None,
                    blast: None,
                    started_at: "2026-06-09T10:00:00+00:00".into(),
                })
                .unwrap(),
            ),
            (
                "OpToolUseCompleted",
                serde_json::to_value(OpToolUseCompleted {
                    session_id: "s".into(),
                    harness_session_id: None,
                    tool: "Bash".into(),
                    status: "ok".into(),
                    error: None,
                    response: None,
                    ended_at: "2026-06-09T10:00:00+00:00".into(),
                })
                .unwrap(),
            ),
            (
                "OpToolUseDenied",
                serde_json::to_value(OpToolUseDenied {
                    session_id: "s".into(),
                    harness_session_id: None,
                    tool: "Bash".into(),
                    input: None,
                    reason: "blocked".into(),
                    denied_by: "ship-gate".into(),
                    denied_at: "2026-06-09T10:00:00+00:00".into(),
                })
                .unwrap(),
            ),
        ] {
            let s = json.to_string();
            for f in DRIVER_EDGE_FIELDS {
                assert!(
                    !s.contains(f),
                    "{name} must not carry {f}: {s}"
                );
            }
        }
    }
}
