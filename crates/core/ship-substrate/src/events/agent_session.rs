//! Agent-session lifecycle event payloads.
//!
//! The dispatch primitive. A single agent invocation — one harness
//! process bound to one workspace, one branch, one prompt — is an
//! agent session. Pipelines are the COMPOSITION of agent sessions.
//!
//! Why the split (vs the old "pipeline-as-primitive" shape): bare
//! `ship use rust-lane` used to travel through `pipeline.run.started`
//! with a synthetic single-step pipeline wrapped around it, dragging
//! the gate/merge/blocked vocabulary into surfaces (sweep, projection,
//! analytics) where they had no meaning. Single dispatch = single
//! event. Multi-step composition = pipeline.run wrapping N agent
//! sessions via `parent_pipeline_run_id`.
//!
//! Wire-shape note: payload field names align with ACP-style
//! semantics (session-centric, turn-centric) so the sidecar protocol
//! can adopt these envelopes directly when that work lands.
//!
//! Naming: this module sits next to `pipeline.rs`, `process.rs`,
//! `hooks.rs`. The mod-level `event_types` constants are re-exported
//! at module scope but not (yet) bundled into `events::types::event_types`;
//! callers import from `events::agent_session::event_types::*` while
//! the inversion arc is in flight.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod event_types {
    /// A new agent dispatch has been requested. Carries everything the
    /// orchestrator needs to bring up the harness exactly once. The
    /// orchestrator's `handle_agent_session_started` consumes this and
    /// emits `agent.session.dispatched` once the process is spawned
    /// and registered.
    pub const AGENT_SESSION_STARTED: &str = "agent.session.started";

    /// The harness process is up, registered with the supervisor,
    /// and observable. Flips status to `Dispatched`.
    pub const AGENT_SESSION_DISPATCHED: &str = "agent.session.dispatched";

    /// The session reached a normal terminal — harness exited zero,
    /// or a keepalive session was explicitly stopped without error.
    /// Flips status to `Completed`.
    pub const AGENT_SESSION_COMPLETED: &str = "agent.session.completed";

    /// The session terminated abnormally — harness exited non-zero,
    /// was killed externally, or spawn failed. `error` field carries
    /// the operator-readable cause. Flips status to `Failed`.
    pub const AGENT_SESSION_FAILED: &str = "agent.session.failed";

    /// Operator cancellation. Distinct from `failed` so observers can
    /// distinguish intentional stop from accidental failure (status
    /// rolls up the same way, but cancel is recorded as the cause).
    /// Flips status to `Cancelled`.
    pub const AGENT_SESSION_CANCELLED: &str = "agent.session.cancelled";

    /// The agent declared end-of-turn (Stop hook fired for claude,
    /// equivalent sentinels for other harnesses) and the session
    /// is now idle awaiting the next operator push. Only emitted for
    /// keepalive sessions; one-shot sessions skip this and go
    /// directly to `completed`. Flips status to `AwaitingPush`.
    pub const AGENT_SESSION_TURN_ENDED: &str = "agent.session.turn_ended";
}

/// How the harness is invoked. Replaces the old `headless: bool` that
/// conflated UX-attach with billing concerns.
///
/// - `Pty`: harness runs in a PTY (claude TUI, codex TUI, ...). Works
///   for all 5 harnesses. Subscription-billed where the harness has
///   a subscription/API split.
/// - `Print`: one-shot non-interactive mode (claude `-p`, codex `exec`).
///   No keepalive. API-billed for claude post-2026-06-19.
/// - `Stream`: server-mode JSON-RPC or REST session (codex
///   `app-server`, opencode REST). Long-lived; supports keepalive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InvocationMode {
    Pty,
    Print,
    Stream,
}

impl InvocationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvocationMode::Pty => "pty",
            InvocationMode::Print => "print",
            InvocationMode::Stream => "stream",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "pty" => InvocationMode::Pty,
            "print" => InvocationMode::Print,
            "stream" => InvocationMode::Stream,
            _ => InvocationMode::Pty,
        }
    }
}

/// Payload for `agent.session.started`. Carries the full dispatch
/// intent. The three orthogonal axes (`open_tab`, `invocation`,
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionStartedPayload {
    /// ULID. Primary key; stable for the life of the session.
    pub session_id: String,
    /// Human-readable, url-safe id. Derived from workspace + suffix
    /// at dispatch time. Used in CLI output and route paths.
    pub slug: String,

    pub agent: String,
    /// Harness override. None resolves the agent profile's default
    /// chain at dispatch time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    pub workspace: String,
    pub branch: String,
    /// Operator-facing source path for the spec (display only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_spec_path: Option<String>,

    // ── The three orthogonal axes ──────────────────────────────────
    /// Does the dispatch open an interactive tab/PTY-attach surface?
    /// Default: caller-supplied; `ship use` derives from TTY presence,
    /// `ship dispatch` requires explicit.
    pub open_tab: bool,
    pub invocation: InvocationMode,
    /// Does the harness stay alive after the current turn? Required
    /// for the push-message workflow (`POST /api/v1/agent_sessions/
    /// {slug}/push`). Mutually exclusive with `invocation=print`
    /// (print exits on completion).
    pub keepalive: bool,

    // ── Content ────────────────────────────────────────────────────
    /// Initial prompt sent to the harness on first turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Inline spec body — written to `.artifacts/job-spec.md` in the
    /// worktree, used by spec-driven harness flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,

    // ── Lifecycle / composition ────────────────────────────────────
    #[serde(default)]
    pub resume: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_session_id: Option<String>,
    /// Session slugs this dispatch waits on. The orchestrator defers
    /// dispatch until all listed sessions reach a terminal state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vars: Option<HashMap<String, String>>,
    /// Repo HEAD SHA at dispatch time. Used by gate evaluation in
    /// composed pipelines (so gates see commits made by earlier steps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_sha: Option<String>,

    /// When this session is one step of a multi-step pipeline run,
    /// points back at the composing `pipeline.run` entity id. `None`
    /// for bare `ship use` / `ship dispatch`. The pipeline composer
    /// watches `agent.session.completed` filtered on this field to
    /// advance to the next step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_pipeline_run_id: Option<String>,
}

/// Payload for `agent.session.dispatched`. Emitted by the orchestrator
/// once the executor returns a handle and the process is registered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionDispatchedPayload {
    pub session_id: String,
    pub slug: String,
    /// Absolute path to the prepared worktree.
    pub worktree: String,
    /// Resolved harness (after default-chain resolution).
    pub harness: String,
    /// Resolved model (after default-chain resolution).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Process id. `None` for server-mode sessions where the
    /// transport is a long-lived JSON-RPC stream (no per-dispatch
    /// child process).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// Payload for `agent.session.completed`. Clean terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionCompletedPayload {
    pub session_id: String,
    pub slug: String,
    /// Exit code. `Some(0)` for normal harness exit; `None` for
    /// operator-stopped keepalive sessions where there's no harness
    /// exit code to report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

/// Payload for `agent.session.failed`. Abnormal terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionFailedPayload {
    pub session_id: String,
    pub slug: String,
    /// Operator-readable cause. Examples: "harness exit code 1",
    /// "process killed externally", "spawn failed: claude binary not
    /// on PATH", "coherence sweep: process missing".
    pub error: String,
    /// Exit code when available; None for spawn-failed or
    /// kill-without-exit cases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Coarse classifier so observers don't have to string-match
    /// the `error` field. Examples: "exit_nonzero", "spawn_failed",
    /// "process_missing", "executor_session_destroyed".
    pub stage: String,
}

/// Payload for `agent.session.cancelled`. Operator stop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionCancelledPayload {
    pub session_id: String,
    pub slug: String,
    /// Optional operator-supplied reason. Free text; mostly for
    /// dashboards rendering "why was this stopped".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Payload for `agent.session.turn_ended`. Keepalive-only. Indicates
/// the agent declared end-of-turn (Stop hook for claude; harness-
/// specific sentinels for others) and is now idle, awaiting the next
/// `push` from the operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionTurnEndedPayload {
    pub session_id: String,
    pub slug: String,
    /// Ordinal of the turn that just ended, when the harness exposes
    /// it. `None` for harnesses that don't surface turn numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_idx: Option<u32>,
    /// Optional harness-reported stop reason.
    /// Same vocabulary as `HookTurnObservedPayload::stop_reason`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_mode_serde_lowercase() {
        assert_eq!(
            serde_json::to_string(&InvocationMode::Pty).unwrap(),
            "\"pty\""
        );
        assert_eq!(
            serde_json::to_string(&InvocationMode::Print).unwrap(),
            "\"print\""
        );
        assert_eq!(
            serde_json::to_string(&InvocationMode::Stream).unwrap(),
            "\"stream\""
        );
        let parsed: InvocationMode = serde_json::from_str("\"pty\"").unwrap();
        assert_eq!(parsed, InvocationMode::Pty);
    }

    #[test]
    fn invocation_mode_db_str_roundtrip() {
        for mode in [InvocationMode::Pty, InvocationMode::Print, InvocationMode::Stream] {
            assert_eq!(InvocationMode::from_db_str(mode.as_str()), mode);
        }
    }

    #[test]
    fn invocation_mode_unknown_db_str_defaults_pty() {
        // Defensive — if a future migration leaves a row with a
        // value we don't recognize (e.g. dropped variant), don't
        // panic; fall back to PTY since it's the safest default.
        assert_eq!(InvocationMode::from_db_str("nope"), InvocationMode::Pty);
        assert_eq!(InvocationMode::from_db_str(""), InvocationMode::Pty);
    }

    #[test]
    fn started_payload_roundtrips() {
        let payload = AgentSessionStartedPayload {
            session_id: "01HV0000000000000000000001".to_string(),
            slug: "auth-feature-abc12345".to_string(),
            agent: "rust-lane".to_string(),
            harness: Some("claude".to_string()),
            model: Some("claude-opus-4-7".to_string()),
            workspace: "auth-feature".to_string(),
            branch: "job/auth-feature-abc12345".to_string(),
            source_spec_path: Some("/host/abs/spec.md".to_string()),
            open_tab: true,
            invocation: InvocationMode::Pty,
            keepalive: true,
            prompt: Some("fix the failing test".to_string()),
            spec: None,
            resume: false,
            resume_session_id: None,
            depends_on: None,
            vars: None,
            baseline_sha: Some("abc123".to_string()),
            parent_pipeline_run_id: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: AgentSessionStartedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, payload.session_id);
        assert_eq!(parsed.invocation, InvocationMode::Pty);
        assert!(parsed.keepalive);
        assert!(parsed.open_tab);
        assert_eq!(parsed.parent_pipeline_run_id, None);
    }

    #[test]
    fn started_payload_minimal_required_fields() {
        // Verifies all the Optional<*> fields with skip_serializing_if
        // are truly optional in deserialization.
        let json = r#"{
            "session_id": "01HV0000000000000000000001",
            "slug": "ws-001",
            "agent": "rust-lane",
            "workspace": "ws",
            "branch": "job/ws-001",
            "open_tab": false,
            "invocation": "print",
            "keepalive": false
        }"#;
        let parsed: AgentSessionStartedPayload = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.agent, "rust-lane");
        assert!(!parsed.open_tab);
        assert_eq!(parsed.invocation, InvocationMode::Print);
        assert!(parsed.harness.is_none());
        assert!(parsed.model.is_none());
        assert!(parsed.prompt.is_none());
        assert!(parsed.spec.is_none());
        assert!(parsed.depends_on.is_none());
        assert!(parsed.vars.is_none());
        assert!(parsed.baseline_sha.is_none());
        assert!(parsed.parent_pipeline_run_id.is_none());
        assert!(parsed.source_spec_path.is_none());
        assert!(parsed.resume_session_id.is_none());
        assert!(!parsed.resume);
    }

    #[test]
    fn pipeline_composition_payload_carries_parent_id() {
        // Pipeline-composed sessions set parent_pipeline_run_id;
        // the orchestrator's pipeline runner filters agent.session.*
        // events on this field to know which step belongs to which
        // pipeline. Without it, the composer can't advance.
        let payload = AgentSessionStartedPayload {
            session_id: "01HV0000000000000000000002".to_string(),
            slug: "ws-step-2".to_string(),
            agent: "test-writer".to_string(),
            harness: None,
            model: None,
            workspace: "ws".to_string(),
            branch: "job/ws-step-2".to_string(),
            source_spec_path: None,
            open_tab: false,
            invocation: InvocationMode::Pty,
            keepalive: false,
            prompt: None,
            spec: Some("## Goal\nwrite tests".to_string()),
            resume: false,
            resume_session_id: None,
            depends_on: None,
            vars: None,
            baseline_sha: None,
            parent_pipeline_run_id: Some("01HV-PIPELINE-RUN-ID".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("parent_pipeline_run_id"));
        assert!(json.contains("01HV-PIPELINE-RUN-ID"));
    }

    #[test]
    fn dispatched_payload_roundtrips() {
        let payload = AgentSessionDispatchedPayload {
            session_id: "01HV-SESS".to_string(),
            slug: "ws-001".to_string(),
            worktree: "/home/u/.ship/worktrees/ws-001".to_string(),
            harness: "claude".to_string(),
            model: Some("claude-opus-4-7".to_string()),
            pid: Some(12345),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: AgentSessionDispatchedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, "01HV-SESS");
        assert_eq!(parsed.pid, Some(12345));
    }

    #[test]
    fn dispatched_payload_pid_optional_for_server_mode() {
        let json = r#"{
            "session_id": "01HV-SESS",
            "slug": "ws-001",
            "worktree": "/path",
            "harness": "codex"
        }"#;
        let parsed: AgentSessionDispatchedPayload = serde_json::from_str(json).unwrap();
        assert!(parsed.pid.is_none());
        assert!(parsed.model.is_none());
    }

    #[test]
    fn completed_payload_roundtrips() {
        let payload = AgentSessionCompletedPayload {
            session_id: "01HV-SESS".to_string(),
            slug: "ws-001".to_string(),
            exit_code: Some(0),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: AgentSessionCompletedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.exit_code, Some(0));
    }

    #[test]
    fn failed_payload_carries_stage_classifier() {
        let payload = AgentSessionFailedPayload {
            session_id: "01HV-SESS".to_string(),
            slug: "ws-001".to_string(),
            error: "claude binary not on PATH".to_string(),
            exit_code: None,
            stage: "spawn_failed".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: AgentSessionFailedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stage, "spawn_failed");
        assert_eq!(parsed.exit_code, None);
    }

    #[test]
    fn cancelled_payload_reason_optional() {
        let json = r#"{
            "session_id": "01HV-SESS",
            "slug": "ws-001"
        }"#;
        let parsed: AgentSessionCancelledPayload = serde_json::from_str(json).unwrap();
        assert!(parsed.reason.is_none());
    }

    #[test]
    fn turn_ended_payload_roundtrips() {
        let payload = AgentSessionTurnEndedPayload {
            session_id: "01HV-SESS".to_string(),
            slug: "ws-001".to_string(),
            turn_idx: Some(3),
            stop_reason: Some("end_turn".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: AgentSessionTurnEndedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.turn_idx, Some(3));
        assert_eq!(parsed.stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn event_type_constants_are_namespaced() {
        // Hard pin: every event type lives under "agent.session." so
        // the orchestrator's subscribe filter ("agent.session.")
        // matches all of them without partial-prefix surprises.
        for ty in [
            event_types::AGENT_SESSION_STARTED,
            event_types::AGENT_SESSION_DISPATCHED,
            event_types::AGENT_SESSION_COMPLETED,
            event_types::AGENT_SESSION_FAILED,
            event_types::AGENT_SESSION_CANCELLED,
            event_types::AGENT_SESSION_TURN_ENDED,
        ] {
            assert!(
                ty.starts_with("agent.session."),
                "event type must live under agent.session.: got {ty}"
            );
        }
    }
}
