//! Generate proto contract fixtures from the themed corpus (`cargo xtask gen-fixtures`).
//!
//! Source of VALUES: `proto/fixtures/theme.json` — one coherent demo Ship
//! workspace (see its `_theme`). Source of SHAPE: the `ship.ops.v1` proto
//! messages. For each op we decode the corpus value into its generated
//! connectrpc response type (the drift guard — a renamed/retyped field fails
//! here), then emit `proto/fixtures/<service>.json` keyed
//! `{ "<Method>": { "response_ok": <value> } }` — the same shape as the
//! hand-authored gateway fixture (`auth_service.json`).
//!
//! Three consumers: cross-language CONTRACT tests (Rust parses + TS Connect
//! client decodes the same file → wire drift caught), FUZZ seed corpus (the
//! FDS is the schema, these are the seeds), and DESIGN HANDOFF (realistic
//! coherent data, not placeholders).
//!
//! Only typed responses get a fixture; ops still returning the opaque
//! `JsonResponse` have no shape to validate and are skipped until they gain a
//! typed proto message (which the proto-SSOT migration is doing).

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{json, Value};

/// Validate a corpus value against the proto response type `T` (decode-only;
/// `T: Deserialize` is all the generated types guarantee). Non-capturing, so it
/// coerces to a `fn` pointer in the table below.
fn check<T: serde::de::DeserializeOwned>(v: &Value) -> Result<()> {
    serde_json::from_value::<T>(v.clone()).map(|_| ()).map_err(|e| anyhow::anyhow!(e))
}

/// One op's fixture binding: dotted op name → (service, rpc method, validator).
/// The service/method name the wire path uses; the validator pins the proto type.
struct Binding {
    op: &'static str,
    service: &'static str,
    method: &'static str,
    validate: fn(&Value) -> Result<()>,
}

/// All typed-response ops across the authored ship.ops.v1 domain services.
/// JsonResponse ops have no shape to validate and are absent.
fn bindings() -> Vec<Binding> {
    use ship_ops_rpc::*;
    macro_rules! b {
        ($op:literal, $svc:literal, $m:literal, $t:ty) => {
            Binding { op: $op, service: $svc, method: $m, validate: check::<$t> }
        };
    }
    vec![
        // agents
        b!("agents.list", "agents_service", "List", AgentIndexResponse),
        b!("agents.summaries", "agents_service", "Summaries", AgentsSummariesResponse),
        b!("agents.detail", "agents_service", "Detail", AgentDetail),
        b!("agents.templates", "agents_service", "Templates", AgentTemplatesResponse),
        b!("agents.permission_presets", "agents_service", "PermissionPresets", PermissionPresetsResponse),
        b!("agents.harnesses", "agents_service", "Harnesses", HarnessesResponse),
        b!("agents.create", "agents_service", "Create", ScopedMutation),
        b!("agents.put", "agents_service", "Put", ScopedMutation),
        b!("agents.patch", "agents_service", "Patch", ScopedMutation),
        b!("agents.delete", "agents_service", "Delete", ScopedMutation),
        b!("agents.rename", "agents_service", "Rename", ScopedMutation),
        b!("agents.duplicate", "agents_service", "Duplicate", ScopedMutation),
        b!("agents.move_scope", "agents_service", "MoveScope", ScopedMutation),
        // analytics
        b!("analytics.list_sessions", "analytics_service", "ListSessions", AnalyticsSessionsResponse),
        b!("analytics.get_session", "analytics_service", "GetSession", AnalyticsSessionResponse),
        b!("analytics.tool_usage", "analytics_service", "ToolUsage", AnalyticsToolUsageResponse),
        b!("analytics.session_turns", "analytics_service", "SessionTurns", AnalyticsSessionTurnsResponse),
        // artifacts
        b!("artifacts.list", "artifacts_service", "List", ArtifactsListResponse),
        b!("artifacts.patch", "artifacts_service", "Patch", ArtifactRecord),
        b!("artifacts.promote", "artifacts_service", "Promote", ArtifactRecord),
        b!("artifacts.delete", "artifacts_service", "Delete", Deletion),
        // chat
        b!("chat.state", "chat_service", "State", ChatStateResponse),
        // collections
        b!("collections.create_item", "collections_service", "CreateItem", CollectionItem),
        b!("collections.update_item", "collections_service", "UpdateItem", CollectionItem),
        b!("collections.delete_item", "collections_service", "DeleteItem", Deletion),
        // events
        b!("events.list", "events_service", "List", EventsListResponse),
        // git
        b!("git.status", "git_service", "Status", StatusBuckets),
        b!("git.log", "git_service", "Log", GitLogResponse),
        b!("git.branches", "git_service", "Branches", GitBranchesResponse),
        b!("git.overview", "git_service", "Overview", GitOverviewResponse),
        b!("git.diff", "git_service", "Diff", GitDiffResponse),
        // mcp
        b!("mcp.list", "mcp_service", "List", McpIndexResponse),
        b!("mcp.create", "mcp_service", "Create", ScopedMutation),
        b!("mcp.put", "mcp_service", "Put", ScopedMutation),
        b!("mcp.patch", "mcp_service", "Patch", ScopedMutation),
        b!("mcp.delete", "mcp_service", "Delete", ScopedMutation),
        // ops
        b!("ops.meta", "ops_service", "Meta", MetaResponse),
        // permissions
        b!("permissions.list", "permissions_service", "List", PermissionsIndexResponse),
        b!("permissions.tool_patterns", "permissions_service", "ToolPatterns", ToolPatternsResponse),
        b!("permissions.create", "permissions_service", "Create", ScopedMutation),
        b!("permissions.put", "permissions_service", "Put", ScopedMutation),
        b!("permissions.patch", "permissions_service", "Patch", ScopedMutation),
        b!("permissions.delete", "permissions_service", "Delete", ScopedMutation),
        // pipelines
        b!("pipelines.list", "pipelines_service", "List", PipelinesIndexResponse),
        b!("pipelines.put", "pipelines_service", "Put", ScopedMutation),
        b!("pipelines.patch", "pipelines_service", "Patch", ScopedMutation),
        b!("pipelines.delete", "pipelines_service", "Delete", ScopedMutation),
        // project
        b!("project.active_project", "project_service", "ActiveProject", WorkspaceActiveResponse),
        b!("project.add", "project_service", "Add", ProjectRecord),
        // rules
        b!("rules.list", "rules_service", "List", RulesIndexResponse),
        b!("rules.get", "rules_service", "Get", RuleGetResponse),
        b!("rules.create", "rules_service", "Create", ScopedMutation),
        b!("rules.put", "rules_service", "Put", ScopedMutation),
        b!("rules.delete", "rules_service", "Delete", ScopedMutation),
        // runtime
        b!("runtime.list_procs", "runtime_service", "ListProcs", ListProcsResponse),
        // sessions
        b!("sessions.list", "sessions_service", "List", SessionsListResponse),
        // settings
        b!("settings.set", "settings_service", "Set", SettingValue),
        // skills
        b!("skills.list", "skills_service", "List", SkillsIndexResponse),
        b!("skills.file", "skills_service", "File", SkillFileResponse),
        b!("skills.create", "skills_service", "Create", ScopedMutation),
        b!("skills.delete", "skills_service", "Delete", ScopedMutation),
        b!("skills.rename", "skills_service", "Rename", ScopedMutation),
        b!("skills.duplicate", "skills_service", "Duplicate", ScopedMutation),
        b!("skills.write_frontmatter", "skills_service", "WriteFrontmatter", ScopedMutation),
        b!("skills.write_body", "skills_service", "WriteBody", ScopedMutation),
        b!("skills.move_scope", "skills_service", "MoveScope", ScopedMutation),
        b!("skills.write_file", "skills_service", "WriteFile", ScopedMutation),
        b!("skills.delete_file", "skills_service", "DeleteFile", ScopedMutation),
        // specs
        b!("specs.list", "specs_service", "List", SpecsListResponse),
        b!("specs.get", "specs_service", "Get", SpecGetResponse),
        b!("specs.detail", "specs_service", "Detail", SpecDetail),
        // view_errors
        b!("view_errors.post", "view_errors_service", "Post", PostViewErrorResponse),
        b!("view_errors.list", "view_errors_service", "List", ListViewErrorsResponse),
        b!("view_errors.clear", "view_errors_service", "Clear", ClearViewErrorsResponse),
        // workspaces
        b!("workspaces.active", "workspaces_service", "Active", WorkspaceActiveResponse),
    ]
}

pub fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures_dir = root.join("proto/fixtures");
    let theme_path = fixtures_dir.join("theme.json");
    let theme: Value = serde_json::from_slice(
        &std::fs::read(&theme_path).with_context(|| format!("read {}", theme_path.display()))?,
    )
    .with_context(|| format!("parse {}", theme_path.display()))?;
    let responses = theme
        .get("responses")
        .and_then(Value::as_object)
        .context("theme.json: missing `responses` object")?;

    // service file → { Method: { response_ok: value } }
    let mut by_service: BTreeMap<&str, serde_json::Map<String, Value>> = BTreeMap::new();
    let mut count = 0usize;
    for b in bindings() {
        let value = responses
            .get(b.op)
            .with_context(|| format!("theme.json: no `responses.{}` for {}.{}", b.op, b.service, b.method))?;
        (b.validate)(value).with_context(|| {
            format!("fixture `{}` does not match proto {}.{}", b.op, b.service, b.method)
        })?;
        by_service
            .entry(b.service)
            .or_default()
            .insert(b.method.to_string(), json!({ "response_ok": value }));
        count += 1;
    }

    for (service, methods) in &by_service {
        let path = fixtures_dir.join(format!("{service}.json"));
        let body = serde_json::to_string_pretty(&Value::Object(methods.clone()))?;
        std::fs::write(&path, format!("{body}\n"))
            .with_context(|| format!("write {}", path.display()))?;
    }
    eprintln!(
        "gen-fixtures: {count} op fixtures → {} service file(s) in {}",
        by_service.len(),
        fixtures_dir.display()
    );
    Ok(())
}
