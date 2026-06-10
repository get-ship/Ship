//! Canonical ship-shaped agent-session event payloads — the L1 payloads home.
//!
//! The legacy driver-edge family lives in
//! `ship-substrate/src/events/agent_session.rs` (the `agent.session.*`
//! events, still dual-emitted transitionally pending the spawn-intent
//! channel, #26): its payloads carry `pid`, `branch`, `worktree`,
//! `exit_code` — driver edges that drivers pull in. THIS module is the
//! **canonical** family: payloads carry only the session-shaped primitives
//! that the kernel, projections, and observers can consume without knowing
//! which runtime produced them.
//!
//! Self-contained primitives only — no driver edges:
//! - `SessionStarted`    — session_id, agent_id, harness, model, space_id, input, parent
//! - `SessionCompleted`  — session_id
//! - `SessionFailed`     — session_id, reason
//! - `SessionCancelled`  — session_id, reason
//! - `SessionSuspended`  — session_id, checkpoint
//! - `TurnStarted`       — session_id, idx
//! - `TurnEnded`         — session_id, idx, stop_reason
//!
//! The canonical `SessionStarted` defined here IS the registered payload for
//! `session.started` (see `event_registrations.rs`) and, since workbench-teardown
//! (R6), the only session-start emit. The legacy `agent.session.started`
//! (ship-substrate `events/agent_session.rs`) is still dual-emitted as the
//! spawn trigger, transitionally, pending the spawn-intent channel (#26).
//!
//! Scope and retention (canonical axis, set at `register_event!`):
//! - `session.*` (coarse)  — Space-scoped, Durable
//! - `session.turn.*` (medium) — Runtime-scoped, Ephemeral
//!
//! See the `canonical-session-events` ADR / `.artifacts/agents-architecture.md`
//! for the design rationale.
//!
//! Wire-shape note: the fields below are intentionally minimal so the
//! payload round-trips cleanly through the kernel envelope
//! (`payload_json` is a UTF-8 string of `serde_json::to_string(&payload)`).
//! Both runtimes (Floki, ship-code) call the emit helpers in
//! `agent_def::emit::*` to build the envelope; drivers don't author these
//! payloads by hand.
use serde::{Deserialize, Serialize};

// Re-export the consts from ship-types so the L1 module and the type-names
// passed to `register_event!` live next to the payloads they describe.
pub use ship_types::event_types::canonical_session as event_types;

// ── Session (coarse) — Space-scoped, Durable ───────────────────────────────

/// Payload for `session.started`. The first event of a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStarted {
    /// ULID. Primary key; stable for the life of the session.
    pub session_id: String,
    /// The agent definition the session is running (e.g. "claude", "floki").
    pub agent_id: String,
    /// The runtime that produced the session (e.g. "floki", "claude",
    /// "codex"). Distinct from `agent_id` because a single agent can run
    /// on multiple harnesses.
    pub harness: String,
    /// Resolved model id (after default-chain resolution), when the
    /// runtime exposes one. `None` for harnesses that don't surface it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The Space this session belongs to. Stamped onto the envelope as
    /// `(scope=Space, scope_key=space_id)` at emit.
    pub space_id: String,
    /// Human-readable, url-safe operational identifier. Derived from
    /// workspace + suffix at dispatch time. Used in cancel paths,
    /// sweep orphan checks, and pipeline compose-terminal lookups.
    /// `None` for Floki sessions and other runtime-internal sessions
    /// that have no external slug identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// The initial input that started the session (free text). For
    /// pipeline-composed sessions this is the step prompt; for
    /// `ship use` it is the user prompt; for chat it is the first turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    /// The originating session or pipeline-run id, when this session
    /// was composed from another. `None` for top-level user invocations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

/// Payload for `session.completed`. Clean terminal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionCompleted {
    pub session_id: String,
}

/// Payload for `session.failed`. Abnormal terminal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionFailed {
    pub session_id: String,
    /// Operator-readable cause. Examples: "harness exit code 1",
    /// "process killed externally", "spawn failed: claude binary not
    /// on PATH", "coherence sweep: process missing". Free text; not
    /// an enum, so drivers don't have to round-trip every failure mode.
    pub reason: String,
}

/// Payload for `session.cancelled`. Operator stop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionCancelled {
    pub session_id: String,
    /// Optional operator-supplied reason. Free text; mostly for
    /// dashboards rendering "why was this stopped".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Payload for `session.suspended`. Paused for later resume.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSuspended {
    pub session_id: String,
    /// Opaque checkpoint handle the runtime reloads to skip already-
    /// completed work. Runtime-defined format; not parsed by the kernel.
    pub checkpoint: String,
}

// ── Turn (medium) — Runtime-scoped, Ephemeral ──────────────────────────────

/// Payload for `session.turn.started`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnStarted {
    pub session_id: String,
    /// 0-based turn index. Stable across the session.
    pub idx: u32,
}

/// Payload for `session.turn.ended`.
///
/// Token-usage fields (model, input_tokens, …) are additive (all `Option<>`
/// with `serde(default)` so old emitters that don't carry usage produce `None`).
/// These fields are the canonical home for per-turn token telemetry (#25b):
/// the supervisor synth path writes them here; the Floki emitter writes only
/// the minimal `session_id / idx / stop_reason` slice.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TurnEnded {
    pub session_id: String,
    pub idx: u32,
    /// Optional harness-reported stop reason (e.g. "end_turn",
    /// "max_tokens", "tool_use"). Free text; not an enum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    // ── Token-usage fields (canonical home for per-turn usage, #25b) ──────
    /// Harness-native session id when available (claude's session UUID,
    /// opencode's session, etc.). Carried for analytics joins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_session_id: Option<String>,
    /// Model used for this turn — may differ from the session's initial model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Input tokens consumed this turn. `None` when the harness didn't surface it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    /// Output tokens produced this turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    /// Reasoning/thinking tokens (Claude extended thinking). `None` for models without it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
    /// Cache-write tokens (Anthropic prompt-cache API).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_tokens: Option<u64>,
    /// Cache-read tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    /// Cost in USD as reported by the harness. Captured verbatim, never
    /// recomputed by Ship. `None` when the harness didn't surface it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_reported_cost_usd: Option<f64>,
    /// ISO-8601 timestamp of the turn-end observation. `None` when the
    /// emitter didn't stamp it (use the envelope's `created_at` instead).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip + invariant check. The canonical family must NOT carry any
    /// driver-edge fields (pid, branch, worktree, exit_code) — that's
    /// the whole point of being canonical.  Covers all 7 canonical structs.
    #[test]
    fn payloads_carry_no_driver_edges() {
        use crate::guards::DRIVER_EDGE_FIELDS;

        macro_rules! assert_no_driver_edges {
            ($val:expr, $name:literal) => {{
                let json = serde_json::to_string(&$val).unwrap();
                for f in DRIVER_EDGE_FIELDS {
                    assert!(
                        !json.contains(f),
                        "{} must not carry {}: {}",
                        $name,
                        f,
                        json
                    );
                }
            }};
        }

        assert_no_driver_edges!(
            SessionStarted {
                session_id: "s1".into(),
                agent_id: "claude".into(),
                harness: "claude".into(),
                model: Some("sonnet".into()),
                space_id: "sp".into(),
                slug: Some("my-agent-abc12345".into()),
                input: Some("hi".into()),
                parent: None,
            },
            "SessionStarted"
        );
        let _: SessionStarted =
            serde_json::from_str(&serde_json::to_string(&SessionStarted {
                session_id: "s1".into(),
                agent_id: "claude".into(),
                harness: "claude".into(),
                model: None,
                space_id: "sp".into(),
                slug: None,
                input: None,
                parent: None,
            })
            .unwrap())
            .unwrap();

        assert_no_driver_edges!(SessionCompleted { session_id: "s1".into() }, "SessionCompleted");

        assert_no_driver_edges!(
            SessionFailed { session_id: "s1".into(), reason: "exit 1".into() },
            "SessionFailed"
        );

        assert_no_driver_edges!(
            SessionCancelled { session_id: "s1".into(), reason: None },
            "SessionCancelled"
        );

        assert_no_driver_edges!(
            SessionSuspended { session_id: "s1".into(), checkpoint: "ckpt-abc".into() },
            "SessionSuspended"
        );

        assert_no_driver_edges!(TurnStarted { session_id: "s1".into(), idx: 0 }, "TurnStarted");

        assert_no_driver_edges!(
            TurnEnded { session_id: "s1".into(), idx: 0, stop_reason: None, ..Default::default() },
            "TurnEnded"
        );
    }

    #[test]
    fn session_failed_has_reason() {
        let f = SessionFailed {
            session_id: "s1".into(),
            reason: "harness exit 1".into(),
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: SessionFailed = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn turn_end_carries_index_and_optional_stop_reason() {
        let t = TurnEnded {
            session_id: "s1".into(),
            idx: 3,
            stop_reason: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(!json.contains("stop_reason")); // skipped when None
        let back: TurnEnded = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }
}
