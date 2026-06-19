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

// Session aggregate types were deleted with the workbench family (R6).
// Canonical session payload schema evolution tests live in ship-substrate
// where the canonical_session types are defined.

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
        "harness": "claude",
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

// ── Job ───────────────────────────────────────────────────────────────────────

// Job event payload evolution is pinned in `ship-substrate/src/events/job.rs`
// where the canonical 18-field structs live (they reference `ship-pipelines`,
// which `ship-types` as a leaf crate cannot depend on).

// ── Artifact aggregate ──────────────────────────────────────────────────────

// Artifact payload evolution tests moved with their structs to
// ship-artifacts/tests/events.rs (keystone-A de-bucket, 2026-06-10).

// ── Config / project / supervisor ───────────────────────────────────────────

#[test]
fn config_changed_v1_round_trips() {
    let json = r#"{"subject": "agents", "detail": "agent-X.jsonc"}"#;
    let p: ConfigChanged = parse(json, "ConfigChanged");
    assert_eq!(p.subject, "agents");
    assert_eq!(p.detail.as_deref(), Some("agent-X.jsonc"));
}

// `agent.def.updated` (Format-2 notification): carries ONLY {path, content_hash}.
// The space_id rides the envelope scope_key (Space-scoped), never the payload —
// mirror ConfigFileChanged. The notification must NEVER carry file content.
#[test]
fn agent_def_updated_v1_round_trips() {
    let json = r#"{"path": "code-reviewer.jsonc", "content_hash": "deadbeef"}"#;
    let p: AgentDefUpdated = parse(json, "AgentDefUpdated");
    assert_eq!(p.path, "code-reviewer.jsonc");
    assert_eq!(p.content_hash, "deadbeef");
}

// Format-2 invariant proof: serializing the notification yields ONLY the path +
// content_hash keys — there is no `content` (or any body) field on the wire, so
// a def's contents can never ride a change notification.
#[test]
fn agent_def_updated_carries_no_content() {
    let p = AgentDefUpdated {
        path: "x.jsonc".into(),
        content_hash: "abc123".into(),
    };
    let v: serde_json::Value = serde_json::to_value(&p).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 2, "Format-2 notification has exactly two fields");
    assert!(obj.contains_key("path"));
    assert!(obj.contains_key("content_hash"));
    assert!(!obj.contains_key("content"), "notification must NOT carry file content");
    assert!(!obj.contains_key("body"), "notification must NOT carry file content");
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
        "harness": "claude",
        "plugins_installed": ["superpowers"]
    }"#;
    let p: CompileCompleted = parse(json, "CompileCompleted");
    assert_eq!(p.active_agent.as_deref(), Some("lead"));
    assert_eq!(p.ship_version, "0.1.0");
}
