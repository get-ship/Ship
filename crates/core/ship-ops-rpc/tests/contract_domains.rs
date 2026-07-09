//! Contract tests for the domain-service fixture files: artifacts,
//! collections, events, mcp, permissions, pipelines, space, runtime,
//! sessions, skills, specs. One test per distinct
//! read shape; a single representative ScopedMutation/Deletion test each.
//! (chat gutted in agents-arc E6; sessions folded into AgentsService.)

use serde_json::Value;
use ship_ops_rpc::{
    ArtifactsListResponse, CollectionItem, Deletion,
    EventsListResponse, ListProcsResponse, McpIndexResponse, PermissionsIndexResponse,
    PipelinesIndexResponse, SessionsListResponse, SkillFileResponse, SkillsIndexResponse, Space,
    SpaceInfo, SpaceList, SpecDetail, SpecGetResponse, SpecsListResponse,
    ToolPatternsResponse,
};

const ARTIFACTS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/artifacts_service.json"));
const COLLECTIONS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/collections_service.json"));
const EVENTS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/events_service.json"));
const MCP: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/mcp_service.json"));
const PERMISSIONS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/permissions_service.json"));
const PIPELINES: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/pipelines_service.json"));
const SPACE: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/space_service.json"));
const RUNTIME: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/runtime_service.json"));
// sessions folded into AgentsService (agents-arc E6: sessions.list → agents.sessions).
const AGENTS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/agents_service.json"));
const SKILLS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/skills_service.json"));
const SPECS: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/specs_service.json"));

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
    let r: Space = serde_json::from_str(&response_ok(SPACE, "Add")).expect("parse Space");
    assert_eq!(r.id, "harbor");
    assert_eq!(r.path, "/home/dev/harbor");
}

#[test]
fn space_list_parses_spaces() {
    // S1 cutover: `SpaceList.active_id` is GONE (multi-window kills the
    // open-one-at-a-time world). The list is now just the opened-spaces
    // record; identity of the served space ships in `SpaceService.Info`.
    let r: SpaceList = serde_json::from_str(&response_ok(SPACE, "List")).expect("parse SpaceList");
    assert_eq!(r.spaces.len(), 1);
    assert_eq!(r.spaces[0].id, "harbor");
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
        serde_json::from_str(&response_ok(AGENTS, "Sessions")).expect("parse SessionsListResponse");
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].agent_id.as_deref(), Some("rust-lane"));
    assert_eq!(r.items[0].harness.as_deref(), Some("claude"));
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
fn space_info_parses_identity_only() {
    // S4: SpaceInfo is identity-only (id/root/name). The effective settings
    // overlay moved to the unified SettingsService — see settings_contract.rs.
    let r: SpaceInfo =
        serde_json::from_str(&response_ok(SPACE, "Info")).expect("parse SpaceInfo");
    assert_eq!(r.space_id.as_deref(), Some("harbor"));
    assert_eq!(r.root.as_deref(), Some("/home/dev/harbor"));
    // No settings_json field on the proto — per-space settings live on
    // SettingsService.
    assert!(r.space_id.is_some());
}


