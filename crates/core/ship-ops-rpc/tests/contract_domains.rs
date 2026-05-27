//! Contract tests for the domain-service fixture files: artifacts, chat,
//! collections, events, mcp, permissions, pipelines, project, runtime,
//! sessions, settings, skills, specs, workspaces. One test per distinct
//! read shape; a single representative ScopedMutation/Deletion test each.

use serde_json::Value;
use ship_ops_rpc::{
    ArtifactsListResponse, ChatStateResponse, CollectionItem, Deletion, EventsListResponse,
    ListProcsResponse, McpIndexResponse, PermissionsIndexResponse, PipelinesIndexResponse,
    ProjectRecord, SessionsListResponse, SettingValue, SkillFileResponse, SkillsIndexResponse,
    SpecDetail, SpecGetResponse, SpecsListResponse, ToolPatternsResponse, WorkspaceActiveResponse,
};

const ARTIFACTS: &str = include_str!("../../../../proto/fixtures/artifacts_service.json");
const CHAT: &str = include_str!("../../../../proto/fixtures/chat_service.json");
const COLLECTIONS: &str = include_str!("../../../../proto/fixtures/collections_service.json");
const EVENTS: &str = include_str!("../../../../proto/fixtures/events_service.json");
const MCP: &str = include_str!("../../../../proto/fixtures/mcp_service.json");
const PERMISSIONS: &str = include_str!("../../../../proto/fixtures/permissions_service.json");
const PIPELINES: &str = include_str!("../../../../proto/fixtures/pipelines_service.json");
const SPACE: &str = include_str!("../../../../proto/fixtures/space_service.json");
const RUNTIME: &str = include_str!("../../../../proto/fixtures/runtime_service.json");
const SESSIONS: &str = include_str!("../../../../proto/fixtures/sessions_service.json");
const SETTINGS: &str = include_str!("../../../../proto/fixtures/settings_service.json");
const SKILLS: &str = include_str!("../../../../proto/fixtures/skills_service.json");
const SPECS: &str = include_str!("../../../../proto/fixtures/specs_service.json");
const WORKSPACES: &str = include_str!("../../../../proto/fixtures/workspaces_service.json");

fn response_ok(fixtures_json: &str, method: &str) -> String {
    let v: Value = serde_json::from_str(fixtures_json).expect("fixtures parse");
    serde_json::to_string(&v[method]["response_ok"]).expect("response_ok serialize")
}

#[test]
fn artifacts_list_parses_count_and_scope() {
    let r: ArtifactsListResponse =
        serde_json::from_str(&response_ok(ARTIFACTS, "List")).expect("parse ArtifactsListResponse");
    assert_eq!(r.count, 1);
    assert_eq!(r.scope, "project");
    assert_eq!(r.items[0].id, "artifact-checkout-audit");
}

#[test]
fn artifacts_delete_is_deletion() {
    let r: Deletion =
        serde_json::from_str(&response_ok(ARTIFACTS, "Delete")).expect("parse Deletion");
    assert!(r.deleted);
}

#[test]
fn chat_state_parses_messages() {
    let r: ChatStateResponse =
        serde_json::from_str(&response_ok(CHAT, "State")).expect("parse ChatStateResponse");
    assert_eq!(r.workspace_id, "harbor");
    assert_eq!(r.total, 2);
    assert_eq!(r.messages[0].sender, "user");
}

#[test]
fn collections_create_item_parses_envelope() {
    let r: CollectionItem =
        serde_json::from_str(&response_ok(COLLECTIONS, "CreateItem")).expect("parse CollectionItem");
    assert_eq!(r.collection, "notes");
    assert!(!r.data_json.is_empty());
}

#[test]
fn events_list_parses_groups_and_actors() {
    let r: EventsListResponse =
        serde_json::from_str(&response_ok(EVENTS, "List")).expect("parse EventsListResponse");
    assert_eq!(r.groups.len(), 1);
    assert_eq!(r.groups[0].count, 3);
    assert!(r.actors.contains(&"rust-lane".to_string()));
}

#[test]
fn mcp_list_parses_index() {
    let r: McpIndexResponse =
        serde_json::from_str(&response_ok(MCP, "List")).expect("parse McpIndexResponse");
    assert_eq!(r.items.len(), 2);
    assert_eq!(r.items[0].id, "ship");
}

#[test]
fn permissions_list_parses_index() {
    let r: PermissionsIndexResponse =
        serde_json::from_str(&response_ok(PERMISSIONS, "List")).expect("parse PermissionsIndexResponse");
    assert_eq!(r.items.len(), 3);
}

#[test]
fn permissions_tool_patterns_parses_source() {
    let r: ToolPatternsResponse =
        serde_json::from_str(&response_ok(PERMISSIONS, "ToolPatterns")).expect("parse ToolPatternsResponse");
    assert_eq!(r.patterns[0].pattern, "Bash(cargo *)");
    assert_eq!(r.patterns[0].source, "builtin");
}

#[test]
fn pipelines_list_parses_index() {
    let r: PipelinesIndexResponse =
        serde_json::from_str(&response_ok(PIPELINES, "List")).expect("parse PipelinesIndexResponse");
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].id, "ship-feature");
}

#[test]
fn space_add_parses_record() {
    let r: ProjectRecord =
        serde_json::from_str(&response_ok(SPACE, "Add")).expect("parse ProjectRecord");
    assert_eq!(r.id, "harbor");
    assert_eq!(r.path, "/home/dev/harbor");
}

#[test]
fn runtime_list_procs_parses_executor_kind() {
    let r: ListProcsResponse =
        serde_json::from_str(&response_ok(RUNTIME, "ListProcs")).expect("parse ListProcsResponse");
    assert_eq!(r.processes.len(), 1);
    assert_eq!(r.processes[0].executor_kind, "agent");
    assert_eq!(r.processes[0].status, "exited");
}

#[test]
fn sessions_list_parses_agent_and_harness() {
    let r: SessionsListResponse =
        serde_json::from_str(&response_ok(SESSIONS, "List")).expect("parse SessionsListResponse");
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].agent_id.as_deref(), Some("rust-lane"));
    assert_eq!(r.items[0].harness.as_deref(), Some("claude"));
}

#[test]
fn settings_set_parses_key_and_value() {
    let r: SettingValue =
        serde_json::from_str(&response_ok(SETTINGS, "Set")).expect("parse SettingValue");
    assert_eq!(r.key, "theme");
    assert_eq!(r.value_json, "\"dark\"");
}

#[test]
fn skills_list_parses_scoped_index() {
    let r: SkillsIndexResponse =
        serde_json::from_str(&response_ok(SKILLS, "List")).expect("parse SkillsIndexResponse");
    assert_eq!(r.items.len(), 3);
    assert_eq!(r.items[0].id, "code-review");
    assert_eq!(r.items[2].scope, "user");
}

#[test]
fn skills_file_parses_content() {
    let r: SkillFileResponse =
        serde_json::from_str(&response_ok(SKILLS, "File")).expect("parse SkillFileResponse");
    assert_eq!(r.ext, "md");
    assert!(r.content.contains("Code Review"));
}

#[test]
fn specs_list_parses_vars_map() {
    let r: SpecsListResponse =
        serde_json::from_str(&response_ok(SPECS, "List")).expect("parse SpecsListResponse");
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].vars.get("area").map(String::as_str), Some("platform"));
}

#[test]
fn specs_get_parses_body() {
    let r: SpecGetResponse =
        serde_json::from_str(&response_ok(SPECS, "Get")).expect("parse SpecGetResponse");
    assert!(r.body.contains("Scaffold"));
    assert_eq!(r.spec.id, "checkout-flow");
}

#[test]
fn specs_detail_parses_acceptance() {
    let r: SpecDetail =
        serde_json::from_str(&response_ok(SPECS, "Detail")).expect("parse SpecDetail");
    assert_eq!(r.agent_id, "rust-lane");
    assert_eq!(r.acceptance.len(), 2);
    assert!(r.acceptance[0].done);
    assert!(!r.acceptance[1].done);
}

#[test]
fn workspaces_active_parses_id() {
    let r: WorkspaceActiveResponse =
        serde_json::from_str(&response_ok(WORKSPACES, "Active")).expect("parse WorkspaceActiveResponse");
    assert_eq!(r.id.as_deref(), Some("harbor"));
}
