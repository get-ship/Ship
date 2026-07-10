//! `dispatch.requested` — the sanctioned spawn-intent channel.
//!
//! Runtime-scoped, Ephemeral. Registered in `event_registrations.rs` with a
//! mandatory payload validator (Phase 4a requirement). Carries ALL driver-edge
//! fields (`branch`, `source_spec_path`, `baseline_sha`, plus the full dispatch
//! intent) that MUST NOT appear in canonical `session.*` payloads.
//!
//! This is the ONE sanctioned driver-edge carrier in the event vocabulary.
//! The legacy `agent.session.*` family is retired in favour of this event +
//! the canonical `session.*` family (see `agent.session.*` sever in
//! payload_registry.rs and #26 spawn-intent channel design).
//!
//! Guards note: `dispatch.*` events are EXEMPT from the no-driver-edges guard
//! (see [`crate::guards`]) by design — they ARE the sanctioned carrier of those
//! fields. Do NOT add `DispatchRequestedPayload` to the `all_canonical_payloads_carry_no_driver_edges`
//! test; that test covers `session.*` and `op.*` only.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::invocation::InvocationMode;

pub mod event_types {
    /// A new agent dispatch has been requested. Carries the full dispatch
    /// intent and all driver-edge fields the runtime needs to bring up the
    /// harness exactly once. The `service.agents` actor subscribes to this
    /// event and owns the spawn/register/exit-handler atom.
    ///
    /// Replaces the legacy `agent.session.started` 1:1 as the hard-failing
    /// spawn trigger. `session.started` (canonical) is still emitted as a
    /// best-effort audit row.
    pub const DISPATCH_REQUESTED: &str = "dispatch.requested";
}

/// Payload for `dispatch.requested`.
///
/// The sanctioned driver-edge carrier. Carries everything the agent-session
/// runtime needs: identity, harness override, driver-edge coordinates
/// (`branch`, repo, base ref, spec path, `baseline_sha`), invocation axes, and
/// composition seams.
///
/// **Driver-edge fields** (`branch`, `source_spec_path`, `baseline_sha`,
/// `repo_path`, `base_ref`) MUST NOT be copied to canonical `session.*`
/// payloads — those are runtime-agnostic. This event is the SINGLE place
/// where driver-edge context lives in the event store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchRequestedPayload {
    /// ULID. Primary key; stable for the life of the session.
    pub session_id: String,
    /// Human-readable, url-safe id. Derived from repo + suffix at
    /// dispatch time. Used in CLI output and route paths.
    pub slug: String,

    /// Ship-agent profile id.
    pub agent_id: String,
    /// Harness override. `None` resolves the agent profile's default chain
    /// at dispatch time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    // ── Driver-edge fields (SANCTIONED here, FORBIDDEN in session.*) ──────
    /// Git branch for the worktree — driver-edge field.
    pub branch: String,
    /// Absolute path to the repo working tree this job's worktree forks
    /// from. Per-call; meaningful to the coding-agent execution class only.
    /// Replaces the field-10 `workspace` slug label (a dead concept that
    /// confused "workspace" with "repo" and "project" alike). `None`
    /// falls back to the daemon's default repo. Consumed in-call by the
    /// dispatch path's `GitWorktreeIsolation::prepare`; never persisted
    /// to any table; never exported to env.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<String>,
    /// Ref the job's worktree branches from. Per-call; same lifetime
    /// rules as `repo_path`. `None` / empty falls back to the repo's
    /// HEAD — matches `git worktree add -b <branch> <path>` with no base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
    /// Operator-facing source path for the spec (display only) — driver-edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_spec_path: Option<String>,
    /// Repo HEAD SHA at dispatch time — driver-edge, used by gate evaluation
    /// in composed pipelines (so gates see commits made by earlier steps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_sha: Option<String>,

    // ── The three orthogonal axes ──────────────────────────────────────────
    /// Does the dispatch open an interactive tab/PTY-attach surface?
    pub open_tab: bool,
    /// How the harness is invoked (`pty` / `print` / `stream`).
    pub invocation: InvocationMode,
    /// Does the harness stay alive after the current turn?
    pub keepalive: bool,

    // ── Content ───────────────────────────────────────────────────────────
    /// Initial prompt sent to the harness on first turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Inline spec body — written to `.artifacts/job-spec.md` in the
    /// worktree, used by spec-driven harness flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,

    // ── Lifecycle / composition ───────────────────────────────────────────
    #[serde(default)]
    pub resume: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vars: Option<HashMap<String, String>>,
    // (Engine cut 2026-07-10: `parent_pipeline_run_id` retired with the
    // orchestrator. Per the code-mode ruling, sessions never carry
    // pipeline identity — a step-run "parent run" concept does not
    // exist post-cut. The no-double-spawn check in service.agents
    // (which keyed on this field) is gone too.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_requested_payload_roundtrips() {
        let payload = DispatchRequestedPayload {
            session_id: "01HV0000000000000000000001".to_string(),
            slug: "auth-feature-abc12345".to_string(),
            agent_id: "rust-lane".to_string(),
            harness: Some("claude".to_string()),
            model: Some("claude-opus-4-7".to_string()),
            branch: "job/auth-feature-abc12345".to_string(),
            repo_path: Some("/host/abs/repo".to_string()),
            base_ref: Some("feat/next".to_string()),
            source_spec_path: Some("/host/abs/spec.md".to_string()),
            baseline_sha: Some("abc123".to_string()),
            open_tab: true,
            invocation: InvocationMode::Pty,
            keepalive: true,
            prompt: Some("fix the failing test".to_string()),
            spec: None,
            resume: false,
            resume_session_id: None,
            depends_on: None,
            vars: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: DispatchRequestedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, payload.session_id);
        assert_eq!(parsed.agent_id, "rust-lane");
        assert_eq!(parsed.branch, "job/auth-feature-abc12345");
        assert_eq!(parsed.repo_path.as_deref(), Some("/host/abs/repo"));
    }
    #[test]
    fn dispatch_requested_event_type_is_namespaced() {
        assert!(
            event_types::DISPATCH_REQUESTED.starts_with("dispatch."),
            "dispatch.requested must live under dispatch.*: got {}",
            event_types::DISPATCH_REQUESTED
        );
    }

    #[test]
    fn driver_edge_fields_present_in_serialised_form() {
        // Confirm that branch + baseline_sha + repo_path + base_ref appear
        // in JSON (they are SANCTIONED driver-edge fields in this payload).
        let payload = DispatchRequestedPayload {
            session_id: "s1".to_string(),
            slug: "ws-001".to_string(),
            agent_id: "rust-lane".to_string(),
            harness: None,
            model: None,
            branch: "job/ws-001".to_string(),
            repo_path: Some("/abs/repo".to_string()),
            base_ref: Some("main".to_string()),
            source_spec_path: Some("/spec.md".to_string()),
            baseline_sha: Some("deadbeef".to_string()),
            open_tab: false,
            invocation: InvocationMode::Print,
            keepalive: false,
            prompt: None,
            spec: None,
            resume: false,
            resume_session_id: None,
            depends_on: None,
            vars: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("branch"), "branch must appear in dispatch payload JSON");
        assert!(json.contains("baseline_sha"), "baseline_sha must appear in dispatch payload JSON");
        assert!(json.contains("source_spec_path"), "source_spec_path must appear in dispatch payload JSON");
        assert!(json.contains("repo_path"), "repo_path must appear in dispatch payload JSON");
        assert!(json.contains("base_ref"), "base_ref must appear in dispatch payload JSON");
    }
}
