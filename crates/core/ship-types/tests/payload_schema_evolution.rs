//! Schema-evolution round-trip test for every event payload.
//!
//! Each payload type has a v1 JSON fixture inlined here. The test deserializes
//! the fixture into the current struct. This catches breaking changes —
//! renamed fields fail to deserialize; removed fields with `#[serde(default)]`
//! are silently OK (which is fine — that's the additive evolution rule);
//! renamed types break the test at compile time.
//!
//! When you intentionally evolve a payload's shape (additive only — see the
//! schema-evolution doc), update the fixture in lock-step.

use ship_types::payload::*;

fn parse<'de, T: serde::Deserialize<'de>>(json: &'de str, label: &str) -> T {
    serde_json::from_str::<T>(json)
        .unwrap_or_else(|e| panic!("[{label}] v1 fixture failed to deserialize: {e}"))
}

// ── Workspace aggregate (legacy — kept for back-compat replay) ───────────────

#[test]
fn workspace_created_v1_round_trips() {
    let json = r#"{"workspace_id": "ws-1", "path": "/repos/proj"}"#;
    let p: WorkspaceCreated = parse(json, "WorkspaceCreated");
    assert_eq!(p.workspace_id, "ws-1");
    assert_eq!(p.path.as_deref(), Some("/repos/proj"));
}

#[test]
fn workspace_activated_v1_round_trips() {
    let json = r#"{"agent_id": "code-lane", "providers": ["claude", "codex"]}"#;
    let p: WorkspaceActivated = parse(json, "WorkspaceActivated");
    assert_eq!(p.agent_id.as_deref(), Some("code-lane"));
    assert_eq!(p.providers, vec!["claude", "codex"]);
}

#[test]
fn workspace_archived_v1_round_trips() {
    let _: WorkspaceArchived = parse("{}", "WorkspaceArchived");
}

// ── Session aggregate ────────────────────────────────────────────────────────

#[test]
fn session_started_v1_round_trips() {
    let json = r#"{
        "goal": "build the thing",
        "workspace_id": "ws-1",
        "branch": "feat/x",
        "agent_id": "lead",
        "primary_provider": "claude",
        "project_id": "proj-1",
        "kind": "direct_attach",
        "pipeline_run_id": null,
        "chat_thread_id": null
    }"#;
    let p: SessionStarted = parse(json, "SessionStarted");
    assert_eq!(p.goal.as_deref(), Some("build the thing"));
    assert_eq!(p.workspace_id, "ws-1");
}

#[test]
fn session_progress_v1_round_trips() {
    let json = r#"{"message": "step 1 done", "escalation": null}"#;
    let p: SessionProgress = parse(json, "SessionProgress");
    assert_eq!(p.message, "step 1 done");
}

#[test]
fn session_ended_v1_round_trips() {
    let json = r#"{
        "summary": "done",
        "duration_secs": 42,
        "files_changed": 3,
        "lines_added": 10,
        "lines_removed": 2,
        "tool_calls": 5,
        "exit_reason": "success"
    }"#;
    let _: SessionEnded = parse(json, "SessionEnded");
}

#[test]
fn session_attached_v1_round_trips() {
    let json = r#"{"existing_session_id": "sess-1", "workspace_id": "ws-1", "branch": "main"}"#;
    let _: SessionAttached = parse(json, "SessionAttached");
}

#[test]
fn session_heartbeat_v1_round_trips() {
    let _: SessionHeartbeat = parse(r#"{"last_activity_at": "2026-05-04T00:00:00Z"}"#, "SessionHeartbeat");
}

#[test]
fn session_drain_started_v1_round_trips() {
    let _: SessionDrainStarted = parse(r#"{"drained_at": "2026-05-04T00:00:00Z"}"#, "SessionDrainStarted");
}

#[test]
fn session_drain_completed_v1_round_trips() {
    let _: SessionDrainCompleted = parse(r#"{"ended_at": "2026-05-04T00:00:00Z"}"#, "SessionDrainCompleted");
}

#[test]
fn session_drain_aborted_v1_round_trips() {
    let _: SessionDrainAborted = parse(r#"{"resumed_at": "2026-05-04T00:00:00Z"}"#, "SessionDrainAborted");
}

#[test]
fn session_tool_count_incremented_v1_round_trips() {
    let _: SessionToolCountIncremented = parse(r#"{}"#, "SessionToolCountIncremented");
}

// ── Skill aggregate ──────────────────────────────────────────────────────────

#[test]
fn skill_started_v1_round_trips() {
    let _: SkillStarted = parse(r#"{"skill_id": "commit"}"#, "SkillStarted");
}

#[test]
fn skill_completed_v1_round_trips() {
    let _: SkillCompleted = parse(r#"{"skill_id": "commit", "duration_ms": 1234}"#, "SkillCompleted");
}

#[test]
fn skill_failed_v1_round_trips() {
    let _: SkillFailed = parse(r#"{"skill_id": "commit", "error": "boom"}"#, "SkillFailed");
}

// ── Actor aggregate ──────────────────────────────────────────────────────────

#[test]
fn actor_created_v1_round_trips() {
    let json = r#"{
        "kind": "agent",
        "environment_type": "local",
        "workspace_id": "ws-1",
        "project_id": "proj-1",
        "cwd": "/tmp/work",
        "parent_actor_id": null,
        "restart_count": 0,
        "agent_id": "lead",
        "provider": "claude",
        "model": "sonnet",
        "transport": "stdio",
        "persistence": "ephemeral",
        "system_prompt_hash": "abc123",
        "pid": 4242,
        "capabilities": ["read", "write"]
    }"#;
    let p: ActorCreated = parse(json, "ActorCreated");
    assert_eq!(p.kind, "agent");
    assert_eq!(p.workspace_id.as_deref(), Some("ws-1"));
    assert_eq!(p.capabilities, vec!["read", "write"]);
}

#[test]
fn actor_woke_v1_round_trips() {
    let _: ActorWoke = parse("{}", "ActorWoke");
}

#[test]
fn actor_slept_v1_round_trips() {
    let _: ActorSlept = parse(r#"{"idle_secs": 30}"#, "ActorSlept");
}

#[test]
fn actor_stopped_v1_round_trips() {
    let json = r#"{"reason": "graceful", "exit_code": 0, "duration_ms": 500}"#;
    let _: ActorStopped = parse(json, "ActorStopped");
}

#[test]
fn actor_crashed_v1_round_trips() {
    let json = r#"{"error": "panic", "restart_count": 1}"#;
    let _: ActorCrashed = parse(json, "ActorCrashed");
}

// ── Gate / Job ────────────────────────────────────────────────────────────────

#[test]
fn gate_passed_v1_round_trips() {
    let _: GatePassed = parse(r#"{"evidence": "all checks pass"}"#, "GatePassed");
}

#[test]
fn gate_failed_v1_round_trips() {
    let _: GateFailed = parse(r#"{"evidence": "missing test"}"#, "GateFailed");
}

#[test]
fn job_created_v1_round_trips() {
    let _: JobCreated = parse(r#"{"title": "build"}"#, "JobCreated");
}

#[test]
fn job_completed_v1_round_trips() {
    let _: JobCompleted = parse(r#"{"summary": "done"}"#, "JobCompleted");
}

#[test]
fn job_failed_v1_round_trips() {
    let _: JobFailed = parse(r#"{"reason": "non-zero exit"}"#, "JobFailed");
}

#[test]
fn job_dispatched_v1_round_trips() {
    let _: JobDispatched = parse(r#"{"agent_id": "worker-1"}"#, "JobDispatched");
}

// ── Artifact aggregate ──────────────────────────────────────────────────────

#[test]
fn artifact_created_v1_round_trips() {
    let json = r#"{
        "id": "a-1",
        "scope": "run",
        "name": "out.txt",
        "mime": "text/plain",
        "size": 10,
        "content_hash": "deadbeef",
        "owner_actor": "lead",
        "project_id": "p-1",
        "run_id": "r-1"
    }"#;
    let _: ArtifactCreated = parse(json, "ArtifactCreated");
}

#[test]
fn artifact_updated_v1_round_trips() {
    let json = r#"{"id": "a-1", "scope": "run", "changed": {"size": 20}}"#;
    let _: ArtifactUpdated = parse(json, "ArtifactUpdated");
}

#[test]
fn artifact_moved_v1_round_trips() {
    let _: ArtifactMoved = parse(
        r#"{"id": "a-1", "from_scope": "run", "to_scope": "project"}"#,
        "ArtifactMoved",
    );
}

#[test]
fn artifact_deleted_v1_round_trips() {
    let _: ArtifactDeleted = parse(r#"{"id": "a-1", "scope": "run"}"#, "ArtifactDeleted");
}

// ── Config / project / supervisor ───────────────────────────────────────────

#[test]
fn config_changed_v1_round_trips() {
    let json = r#"{"subject": "agents", "detail": "agent-X.jsonc"}"#;
    let p: ConfigChanged = parse(json, "ConfigChanged");
    assert_eq!(p.subject, "agents");
    assert_eq!(p.detail.as_deref(), Some("agent-X.jsonc"));
}

#[test]
fn project_log_v1_round_trips() {
    let _: ProjectLog = parse(
        r#"{"action": "compiled", "details": "all providers"}"#,
        "ProjectLog",
    );
}

#[test]
fn supervisor_shutting_down_v1_round_trips() {
    let _: SupervisorShuttingDown = parse(r#"{"reason": "user"}"#, "SupervisorShuttingDown");
}

// ── Skill / Rule / Permission CRUD ──────────────────────────────────────────

#[test]
fn skill_created_v1_round_trips() {
    let json = r#"{
        "scope": "project",
        "name": "Commit Style",
        "content_hash": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    }"#;
    let p: SkillCreated = parse(json, "SkillCreated");
    assert_eq!(p.scope, "project");
    assert_eq!(p.name, "Commit Style");
}

#[test]
fn skill_updated_v1_round_trips() {
    let json = r#"{
        "scope": "user",
        "content_hash": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    }"#;
    let p: SkillUpdated = parse(json, "SkillUpdated");
    assert_eq!(p.scope, "user");
}

#[test]
fn skill_deleted_v1_round_trips() {
    let p: SkillDeleted = parse(r#"{"scope": "project"}"#, "SkillDeleted");
    assert_eq!(p.scope, "project");
}

#[test]
fn rule_created_v1_round_trips() {
    let json = r#"{
        "file_name": "0001-rust.md",
        "content_hash": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    }"#;
    let p: RuleCreated = parse(json, "RuleCreated");
    assert_eq!(p.file_name, "0001-rust.md");
}

#[test]
fn rule_updated_v1_round_trips() {
    let json = r#"{
        "file_name": "rust.md",
        "content_hash": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    }"#;
    let _: RuleUpdated = parse(json, "RuleUpdated");
}

#[test]
fn rule_deleted_v1_round_trips() {
    let p: RuleDeleted = parse(r#"{"file_name": "0001-rust.md"}"#, "RuleDeleted");
    assert_eq!(p.file_name, "0001-rust.md");
}

#[test]
fn permissions_updated_v1_round_trips() {
    let json = r#"{
        "scope": "project",
        "content_hash": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        "permissions_json": "{\"tools\":{}}"
    }"#;
    let p: PermissionsUpdated = parse(json, "PermissionsUpdated");
    assert_eq!(p.scope, "project");
    assert!(p.permissions_json.contains("tools"));
}

// ── Compile aggregate ───────────────────────────────────────────────────────

#[test]
fn compile_completed_v1_round_trips() {
    let json = r#"{
        "active_agent": "lead",
        "compiled_at": "2026-05-04T00:00:00Z",
        "ship_version": "0.1.0",
        "provider": "claude",
        "plugins_installed": ["superpowers"]
    }"#;
    let p: CompileCompleted = parse(json, "CompileCompleted");
    assert_eq!(p.active_agent.as_deref(), Some("lead"));
    assert_eq!(p.ship_version, "0.1.0");
}
