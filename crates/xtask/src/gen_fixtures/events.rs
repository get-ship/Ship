//! Per-event payload fixtures derived from the live event registry.
//!
//! For each registered event in `ship_types::event_registry::registry()`,
//! emits two files under `proto/fixtures/events/`:
//!   - `<event-name>.json`          — deterministic payload stub
//!   - `<event-name>.envelope.json` — full EventEnvelope wrapping the payload
//!
//! Generation strategy: introspect the registry metadata (scope, retention,
//! stability) and the payload type name. For each payload type we generate a
//! deterministic stub using sensible defaults. Payloads that need richer
//! real-world values are marked with NEEDS_AUTHOR and listed by --report-missing.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use ship_types::event_registry::{EventScope, Retention, Stability, registry};

/// NEEDS_AUTHOR marker — these payload types cannot be automatically stubbed
/// because they contain non-trivial nested types, serde_json::Value fields,
/// or fields where a realistic value isn't obvious from the type alone.
/// Operators MUST hand-author the fixture content. Gen-fixtures emits the
/// stub with the marker and lists them in --report-missing.
const NEEDS_AUTHOR_PAYLOADS: &[&str] = &[
    "ArtifactUpdated",              // changed: serde_json::Value
    "PipelineRunStartedPayload",    // steps: Option<Pipeline>, pipeline: Option<Vec<PipelinePhase>>, vars: HashMap
    "PipelineRunDispatchedPayload", // harness_meta: Option<serde_json::Value>
    // AgentSessionStartedPayload removed (#26): agent.session.* family retired.
    // dispatch.requested uses DispatchRequestedPayload; see dispatch_intent.rs.
];

/// Map from `payload_type_name` → deterministic JSON stub.
/// Only payload types that can be generated fully automatically are here.
/// NEEDS_AUTHOR types get a generic stub with the marker comment embedded
/// in the `_note` field of the JSON.
fn payload_stub(payload_type: &str) -> Value {
    match payload_type {
        // ── Empty-payload events (unit structs) ───────────────────────────────
        "ActorWoke"
        | "SessionToolCountIncremented"
        | "AuthSessionExpired"
        | "AccountLoginCompleted"
        | "AccountKeyRefreshed"
        | "DaemonOwnerUnbound" => json!({}),
        "AccountLoginStarted" => json!({ "redirect_uri": "https://auth.ship.example/oauth/callback" }),
        "AccountLogout" => json!({ "had_token": true, "key_revoked": false }),
        "AgentGrantsProvisioned" => json!({ "grants": ["ship.git.read", "ship.rules.write"], "source": "operator-granted" }),
        "DaemonOwnerBound" => json!({ "source": "cli-login", "bound_at_unix": 1717200000_i64 }),
        "AuthSessionCreated" => json!({
            "agent_id": "test-agent",
            "created_at_unix": 1717200000_i64,
            "expires_at_unix": 1717286400_i64,
            "source": "operator"
        }),
        "AuthSessionRevoked" => json!({ "reason": "operator-logout" }),

        // ── Actor events ─────────────────────────────────────────────────────
        "ActorCreated" => json!({
            "kind": "agent-session",
            "environment_type": "worktree",
            "workspace_id": "test-workspace",
            "restart_count": 0
        }),
        "ActorSlept" => json!({ "idle_secs": 30 }),
        "ActorStopped" => json!({ "reason": "natural-exit" }),
        "ActorCrashed" => json!({ "error": "test-crash-error", "restart_count": 0 }),

        // ── Agent events ─────────────────────────────────────────────────────
        "AgentChanged" => json!({ "path": ".ship/agents/test.jsonc", "content_hash": "sha256-test" }),
        "AgentTaskComplete" => json!({ "task_id": "test-task", "task_type": "code-review" }),

        // ── Agent session events (RETIRED #26) ──────────────────────────────
        // agent.session.* family is deregistered (#26 spawn-intent, 2026-06-10).
        // The payload stubs below are kept for historical fixture generation
        // only (replay of old events in DB rows); they are no longer registered.

        // ── Dispatch events (#26 spawn-intent channel) ───────────────────────
        // dispatch.requested is NEEDS_AUTHOR (vars + depends_on fields).
        // These stubs are not needed for auto-generation — handled by the
        // NEEDS_AUTHOR path above.

        // ── Artifact events ──────────────────────────────────────────────────
        // Metadata is event-sourced (ADR D6/D7): these payloads are
        // projection-complete (`ArtifactProjection` materializes the row).
        "ArtifactCreated" => json!({
            "id": "test-artifact-id",
            "scope": "project",
            "name": "test-artifact.md",
            "mime": "text/markdown",
            "size": 1024,
            "path": "blobs/test-artifact-id",
            "created_at": "2026-01-01T00:00:00Z"
        }),
        "ArtifactMoved" => json!({
            "old_id": "test-artifact-id",
            "from_scope": "run",
            "to_scope": "project",
            "new": {
                "id": "test-artifact-id-2",
                "scope": "project",
                "name": "test-artifact.md",
                "mime": "text/markdown",
                "size": 1024,
                "path": "blobs/test-artifact-id-2",
                "created_at": "2026-01-01T00:00:00Z"
            }
        }),
        "ArtifactDeleted" => json!({ "id": "test-artifact-id", "scope": "project" }),

        // ── Collection events ────────────────────────────────────────────────
        // item_created/updated are projection-complete: they carry the full
        // item / patch data + timestamps so `CollectionProjection` rebuilds the
        // row from the event alone (event-sourced, no read-back).
        "CollectionItemCreated" => json!({
            "collection_id": "test-collection",
            "item_id": "test-item",
            "data": { "title": "test" },
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }),
        "CollectionItemUpdated" => json!({
            "collection_id": "test-collection",
            "item_id": "test-item",
            "data": { "title": "updated" },
            "updated_at": "2026-01-01T00:00:00Z"
        }),
        "CollectionItemDeleted" => json!({ "collection_id": "test-collection", "item_id": "test-item" }),

        // ── Compile events ───────────────────────────────────────────────────
        "CompileCompleted" => json!({ "compiled_at": "2026-01-01T00:00:00Z", "ship_version": "0.1.0" }),

        // ── Config events ────────────────────────────────────────────────────
        "ConfigChanged" => json!({ "subject": "test-config", "detail": null }),
        "ConfigFileChanged" => json!({ "domain": "agents", "path": ".ship/agents/test.jsonc", "content_hash": "sha256-test" }),

        // ── Daemon events ────────────────────────────────────────────────────
        "DaemonStartupCompleted" => json!({ "pid": 12345, "version": "0.1.0" }),
        "DaemonShutdownCompleted" => json!({ "pid": 12345, "uptime_secs": 3600 }),
        "DaemonPortBindFailed" => json!({ "port": 8080, "error": "address already in use" }),
        "DaemonCrashed" => json!({ "error": "signal 11" }),
        "DaemonPidfileAcquired" => json!({ "pid": 12345 }),
        "DaemonPidfileStaleTaken" => json!({ "stale_pid": 11111, "new_pid": 12345 }),


        // ── Mesh events ──────────────────────────────────────────────────────

        // ── Permission events ────────────────────────────────────────────────
        "PermissionsUpdated" => json!({
            "scope": "space",
            "content_hash": "sha256-test",
            "permissions_json": "{\"allow\":[\"Read\"]}"
        }),

        // ── Pipeline events ──────────────────────────────────────────────────
        "PipelineChanged" => json!({ "path": ".ship/pipelines/test.jsonc", "content_hash": "sha256-test" }),
        "PipelineGateRequestedPayload" => json!({ "pipeline_run_id": "test-run-id", "gate_agent": "code-reviewer" }),
        "PipelineGatePassedPayload" => json!({ "pipeline_run_id": "test-run-id" }),
        "PipelineGateFailedPayload" => json!({ "pipeline_run_id": "test-run-id", "reason": "tests failed" }),
        "PipelineRunBlockedPayload" => json!({ "pipeline_run_id": "test-run-id", "blocker": "waiting for review", "needs_human": true }),
        "PipelineRunCompletedPayload" => json!({ "pipeline_run_id": "test-run-id", "slug": "test-slug" }),
        "PipelineRunMergedPayload" => json!({ "pipeline_run_id": "test-run-id", "slug": "test-slug" }),
        "PipelineRunFailedPayload" => json!({ "pipeline_run_id": "test-run-id", "error": "agent exited non-zero" }),
        "PipelineRunCancelledPayload" => json!({ "pipeline_run_id": "test-run-id", "slug": "test-slug", "reason": "operator-stop", "requested_by": "operator" }),
        "PipelineRunUpdatedPayload" => json!({ "pipeline_run_id": "test-run-id", "message": "continue with approach B", "sender": "human" }),
        "PipelineStepSpawnFailedPayload" => json!({ "pipeline_run_id": "test-run-id", "slug": "test-slug", "stage": "executor_spawn", "error": "binary not found" }),
        "PipelineStepStartedPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1", "step_name": "test-step", "slug": "test-slug", "worktree": "/tmp/test" }),
        "PipelineStepCompletedPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1", "slug": "test-slug" }),
        "PipelineStepFailedPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1", "slug": "test-slug", "error": "test failure" }),
        "PipelineStepOutputPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1", "output_key": "result", "output_value": "ok" }),
        "PipelineStepDoneDeclaredPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1", "slug": "test-slug" }),
        "PipelineStepRetry" | "PipelineStepRetryPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1", "retry_count": 1, "reason": "transient error" }),
        "PipelineStepObservedPayload" => json!({ "pipeline_run_id": "test-run-id", "step_id": "step-1" }),

        // ── Process events ───────────────────────────────────────────────────
        "ProcessOrphaned" => json!({ "pid": 99999, "cmd": "claude", "last_known_slug": null, "detected_by": "sweep" }),

        // ── Space events ─────────────────────────────────────────────────────
        "SpaceOpened" => json!({ "path": "/home/user/spaces/test", "name": "test", "opened_at": "2026-01-01T00:00:00Z" }),
        "SpaceSeen" => json!({ "seen_at": "2026-01-01T00:00:00Z" }),

        // ── Rule events ──────────────────────────────────────────────────────
        "RuleCreated" => json!({ "file_name": "test-rule.md", "content_hash": "sha256-test" }),
        "RuleUpdated" => json!({ "file_name": "test-rule.md", "content_hash": "sha256-test" }),
        "RuleDeleted" => json!({ "file_name": "test-rule.md" }),
        "RuleChanged" => json!({ "path": ".ship/rules/test-rule.md", "content_hash": "sha256-test" }),

        // ── Session events ───────────────────────────────────────────────────
        "SessionStarted" => json!({ "workspace_id": "test-workspace", "branch": "main" }),
        "SessionProgress" => json!({ "message": "processing..." }),
        "SessionEnded" => json!({ "summary": "completed task" }),
        "SessionDrainStarted" => json!({ "drained_at": "2026-01-01T00:00:00Z" }),
        "SessionDrainCompleted" => json!({ "ended_at": "2026-01-01T00:00:00Z" }),
        "SessionDrainAborted" => json!({ "resumed_at": "2026-01-01T00:00:00Z" }),
        "SessionHeartbeat" => json!({ "last_activity_at": "2026-01-01T00:00:00Z" }),
        "SessionAttached" => json!({ "existing_session_id": "old-id", "workspace_id": "test-workspace", "branch": "main" }),
        "SessionStatusChanged" => json!({ "from": "running", "to": "idle" }),
        "SessionTurnFailed" => json!({ "error": "api error: rate limit exceeded" }),

        // ── Skill events ─────────────────────────────────────────────────────
        "SkillCreated" => json!({ "scope": "space", "name": "test-skill", "content_hash": "sha256-test" }),
        "SkillUpdated" => json!({ "scope": "space", "content_hash": "sha256-test" }),
        "SkillDeleted" => json!({ "scope": "space" }),
        "SkillChanged" => json!({ "path": ".ship/skills/test.md", "content_hash": "sha256-test" }),
        "SkillStarted" => json!({ "skill_id": "test-skill" }),
        "SkillCompleted" => json!({ "skill_id": "test-skill", "duration_ms": 1234 }),
        "SkillFailed" => json!({ "skill_id": "test-skill", "error": "test failure" }),

        // ── Supervisor events ────────────────────────────────────────────────
        "SupervisorShuttingDown" => json!({ "reason": "graceful-shutdown" }),
        "CoherenceRestoredPayload" => json!({ "actor_id": "test-actor" }),
        "DriftDetectedPayload" => json!({ "actor_id": "test-actor", "expected": "running", "actual": "stopped" }),
        "SweepCompletedPayload" => json!({ "checked": 10, "anomalies": 0 }),
        "SweepStartedPayload" => json!({ "actor_count": 10 }),

        // ── Sync events ──────────────────────────────────────────────────────
        "SyncPushCompleted" => json!({ "events_pushed": 42, "duration_ms": 250 }),
        "SyncPushFailed" => json!({ "error": "connection refused", "target": "cloud.ship.example" }),
        "SyncStarted" => json!({ "target": "cloud.ship.example" }),
        "SyncStopped" => json!({ "target": "cloud.ship.example" }),

        // ── View events ──────────────────────────────────────────────────────
        "ViewChanged" => json!({ "path": ".ship/views/test.sui", "content_hash": "sha256-test" }),

        // ── Human events ─────────────────────────────────────────────────────
        "HumanResponse" => json!({ "conversation_id": "test-conversation", "text": "proceed with option A" }),

        // ── Hook events — DEREGISTERED (#25b, 2026-06-10) ─────────────────
        // hook.* family removed; op.* and session.turn.ended are the replacements.
        // op.* fixtures are generated via their payload type names below.

        // ── op.* events (canonical tool-use — Runtime/Ephemeral) ────────────
        "op::OpToolUseStarted" => json!({
            "session_id": "test-session-id",
            "tool": "Bash",
            "started_at": "2026-01-01T00:00:00Z"
        }),
        "op::OpToolUseCompleted" => json!({
            "session_id": "test-session-id",
            "tool": "Bash",
            "status": "ok",
            "ended_at": "2026-01-01T00:00:01Z"
        }),
        "op::OpToolUseDenied" => json!({
            "session_id": "test-session-id",
            "tool": "Bash",
            "reason": "ship-gate: blocked destructive command",
            "denied_by": "ship-gate",
            "denied_at": "2026-01-01T00:00:00Z"
        }),

        // ── Executor events ──────────────────────────────────────────────────
        "ExecutorSessionDestroyedPayload" => json!({ "session_id": "test-session", "reason": "timeout" }),
        "ExecutorWindowDestroyedPayload" => json!({ "window_id": "test-window", "session_id": "test-session" }),

        // ── Process substrate events ─────────────────────────────────────────
        "ProcessExitedPayload" => json!({ "pid": 12345, "exit_code": 0, "cmd": "claude" }),
        "ProcessKilledPayload" => json!({ "pid": 12345, "signal": 15, "cmd": "claude" }),
        "ProcessSignalSentPayload" => json!({ "pid": 12345, "signal": 15 }),
        "ProcessSpawnedPayload" => json!({ "pid": 12345, "cmd": "claude", "args": ["--dangerously-skip-permissions"] }),

        // ── NEEDS_AUTHOR fallback ────────────────────────────────────────────
        other => json!({
            "_note": format!("NEEDS_AUTHOR: {other} — auto-stub not possible; hand-author this fixture"),
            "_payload_type": other
        }),
    }
}

/// Build the full event registry as a list to iterate over.
struct EventEntry {
    name: &'static str,
    scope: EventScope,
    retention: Retention,
    stability: Stability,
    payload_type: &'static str,
}

fn collect_all_events() -> Vec<EventEntry> {
    let mut events: Vec<EventEntry> = registry()
        .map(|d| EventEntry {
            name: d.name,
            scope: d.primary_scope(),
            retention: d.retention,
            stability: d.stability,
            payload_type: d.payload_type_name,
        })
        .collect();
    events.sort_by_key(|e| e.name);
    events
}

fn scope_str(scope: &EventScope) -> &'static str {
    scope.as_str()
}

fn retention_str(retention: &Retention) -> &'static str {
    match retention {
        Retention::Durable => "durable",
        Retention::Ephemeral => "ephemeral",
    }
}

fn stability_str(stability: &Stability) -> &'static str {
    match stability {
        Stability::Stable => "stable",
        Stability::Deprecated => "deprecated",
        Stability::Experimental => "experimental",
    }
}

fn is_needs_author(payload_type: &str) -> bool {
    NEEDS_AUTHOR_PAYLOADS.contains(&payload_type)
}

/// Build the payload fixture JSON for one event.
fn make_payload_fixture(event: &EventEntry) -> Value {
    let payload = payload_stub(event.payload_type);
    json!({
        "_meta": {
            "event": event.name,
            "scope": scope_str(&event.scope),
            "retention": retention_str(&event.retention),
            "stability": stability_str(&event.stability),
            "payload_type": event.payload_type,
            "needs_author": is_needs_author(event.payload_type)
        },
        "payload": payload
    })
}

/// Build the EventEnvelope fixture JSON for one event.
fn make_envelope_fixture(event: &EventEntry) -> Value {
    let payload = payload_stub(event.payload_type);
    json!({
        "_meta": {
            "event": event.name,
            "scope": scope_str(&event.scope),
            "retention": retention_str(&event.retention),
            "stability": stability_str(&event.stability),
            "payload_type": event.payload_type,
            "needs_author": is_needs_author(event.payload_type)
        },
        "envelope": {
            "id": "01JTEST000000000000000EVENT",
            "event_type": event.name,
            "entity_id": "test-entity-id",
            "actor_id": "test-actor-id",
            "space_id": if matches!(event.scope, EventScope::Space) { "test-space-id" } else { "" },
            "occurred_at": "2026-01-01T00:00:00Z",
            "payload": payload
        }
    })
}

/// Event fixture generation result.
pub struct EventsReport {
    pub total: usize,
    pub auto_generated: usize,
    pub needs_author: usize,
    pub needs_author_list: Vec<String>,
}

pub fn run(fixtures_dir: &Path, check_mode: bool) -> Result<EventsReport> {
    let events_dir = fixtures_dir.join("events");
    std::fs::create_dir_all(&events_dir)
        .with_context(|| format!("create {}", events_dir.display()))?;

    let events = collect_all_events();
    let mut report = EventsReport {
        total: events.len(),
        auto_generated: 0,
        needs_author: 0,
        needs_author_list: Vec::new(),
    };

    for event in &events {
        let safe_name = event.name.replace('.', "_");
        let payload_path = events_dir.join(format!("{safe_name}.json"));
        let envelope_path = events_dir.join(format!("{safe_name}.envelope.json"));

        let payload_json = serde_json::to_string_pretty(&make_payload_fixture(event))?;
        let envelope_json = serde_json::to_string_pretty(&make_envelope_fixture(event))?;

        if is_needs_author(event.payload_type) {
            report.needs_author += 1;
            report.needs_author_list.push(format!(
                "{} ({}): NEEDS_AUTHOR — hand-author fixture for `{}`",
                event.name, scope_str(&event.scope), event.payload_type
            ));
        } else {
            report.auto_generated += 1;
        }

        if check_mode {
            let existing_payload = std::fs::read_to_string(&payload_path).unwrap_or_default();
            let existing_envelope = std::fs::read_to_string(&envelope_path).unwrap_or_default();
            let expected_payload = format!("{payload_json}\n");
            let expected_envelope = format!("{envelope_json}\n");
            if existing_payload != expected_payload || existing_envelope != expected_envelope {
                anyhow::bail!(
                    "event fixture drift: {} is out of date (run `cargo xtask gen-fixtures` to regenerate)",
                    event.name
                );
            }
        } else {
            std::fs::write(&payload_path, format!("{payload_json}\n"))
                .with_context(|| format!("write {}", payload_path.display()))?;
            std::fs::write(&envelope_path, format!("{envelope_json}\n"))
                .with_context(|| format!("write {}", envelope_path.display()))?;
        }
    }

    Ok(report)
}

/// Return events where the fixture is NEEDS_AUTHOR (cannot auto-generate).
pub fn needs_author_list() -> Vec<String> {
    collect_all_events()
        .into_iter()
        .filter(|e| is_needs_author(e.payload_type))
        .map(|e| format!("{} ({}): payload `{}`", e.name, scope_str(&e.scope), e.payload_type))
        .collect()
}

/// Stats about the event registry coverage.
pub fn coverage_stats() -> BTreeMap<String, usize> {
    let events = collect_all_events();
    let total = events.len();
    let needs_author = events.iter().filter(|e| is_needs_author(e.payload_type)).count();
    let auto = total - needs_author;
    let mut m = BTreeMap::new();
    m.insert("total".to_string(), total);
    m.insert("auto_generable".to_string(), auto);
    m.insert("needs_author".to_string(), needs_author);
    m
}
