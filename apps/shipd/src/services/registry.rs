//! Service registry — the single entry point for calling domain
//! services by name. The template SDK (`ship.*` minijinja globals),
//! the forthcoming MCP tool surface, and the TS SDK codegen all
//! resolve through this module.
//!
//! Registered names read like `<domain>.<op>` — `agents.list`,
//! `git.diff`, `permissions.tool_patterns`. Args are a JSON object;
//! unknown keys ignored, required-missing raise `BadArgs`.
//!
//! Read ops only. Writes are exposed as HTTP endpoints (POST/PATCH/
//! PUT/DELETE on each service), not registry entries — keeps the
//! security boundary between "reading" and "mutating" explicit.
//!
//! The registry is synchronous. Every domain state function we call
//! is sync (file I/O via FilesService, libgit2 direct, DB queries).
//! That lets us expose `call` to minijinja without a blocking bridge.

use serde_json::{Value, json};
use std::path::Path;

use crate::rest_api::ApiState;
use crate::services::files::{FilesService, Scope};

/// Structured error returned from a registry call. Rendered verbatim
/// into the template as an `{error, message}` object (per spec §11.3
/// — structured errors, template decides how to render).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryError {
    pub code: &'static str,
    pub message: String,
}

impl RegistryError {
    fn bad_args(msg: impl Into<String>) -> Self {
        Self { code: "bad_args", message: msg.into() }
    }
    fn unknown_service(name: &str) -> Self {
        Self { code: "unknown_service", message: format!("unknown service `{name}`") }
    }
    fn service(msg: impl Into<String>) -> Self {
        Self { code: "service_error", message: msg.into() }
    }
    fn not_found(msg: impl Into<String>) -> Self {
        Self { code: "not_found", message: msg.into() }
    }
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
impl std::error::Error for RegistryError {}

/// Minimal HTML entity escape for interpolated user content in views.
/// The user-view env runs AutoEscape::None (needed for Datastar attrs),
/// so untrusted content (file bodies) must be pre-escaped at the
/// service boundary to prevent `<script>`/`<style>`/`<link>` injection.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

// ── Args helpers ─────────────────────────────────────────────────────

fn parse_scope(args: &Value) -> Result<Option<Scope>, RegistryError> {
    match args.get("scope").and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(RegistryError::bad_args(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn required_str<'a>(args: &'a Value, field: &str) -> Result<&'a str, RegistryError> {
    args.get(field)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| RegistryError::bad_args(format!("missing required arg `{field}`")))
}

fn optional_usize(args: &Value, field: &str, default: usize) -> usize {
    args.get(field)
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(default)
}

// ── Dispatch ─────────────────────────────────────────────────────────

/// Dispatch a registered service call. Synchronous — see module docs.
pub fn call(name: &str, args: &Value, state: &ApiState) -> Result<Value, RegistryError> {
    let files = FilesService::resolve_roots(state.ship_dir());
    match name {
        // ── agents ────────────────────────────────────────────────
        "agents.list" => Ok(json!({
            "items": crate::services::agents::state::list(&files),
        })),
        "agents.summaries" => Ok(json!({
            "items": crate::services::agents::view_state::list_agents(state.ship_dir()),
        })),
        "agents.detail" => {
            let id = required_str(args, "id")?;
            let detail = crate::services::agents::view_state::load_agent(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&detail)
                .map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.catalogs" => {
            let cats = crate::services::agents::view_state::list_agent_catalogs(state.ship_dir());
            serde_json::to_value(&cats)
                .map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, doc) = crate::services::agents::state::load_doc(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::agents::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::agents::state::ScopeTag::from(resolved),
                "doc": doc,
            }))
        }
        "agents.wire" => {
            let id = required_str(args, "id")?;
            let ctx = crate::services::agents::wire::build(id, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.settings" => {
            let id = required_str(args, "id")?;
            let ctx = crate::services::agents::settings_view::build(id, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("agent `{id}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "agents.templates" => Ok(json!({
            "items": crate::services::agents::create::builtin_templates(),
        })),
        "agents.permission_presets" => Ok(json!({
            "items": crate::services::agents::create::builtin_permission_presets(),
        })),
        "agents.providers" => Ok(json!({
            "items": crate::services::agents::create::builtin_providers(),
        })),
        "agents.create_ctx" => {
            let ctx = crate::services::agents::create::build_create_ctx();
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }

        // ── skills ────────────────────────────────────────────────
        "skills.list" => Ok(json!({
            "items": crate::services::skills::state::list(&files),
        })),
        "skills.summaries" => {
            let all = crate::views::state::list_skills(state.ship_dir());
            let q = args.get("q").and_then(|v| v.as_str()).unwrap_or("");
            let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("all");
            let tag = args.get("tag").and_then(|v| v.as_str()).unwrap_or("");
            let needle = q.to_lowercase();
            let total = all.len();
            let mut smart = 0usize;
            let mut docs = 0usize;
            let mut tag_set: std::collections::BTreeSet<String> = Default::default();
            for s in &all {
                match serde_json::to_value(&s.kind).ok().and_then(|v| v.as_str().map(String::from)).as_deref() {
                    Some("smart") => smart += 1,
                    Some("docs") => docs += 1,
                    _ => {}
                }
                for t in &s.tags { tag_set.insert(t.clone()); }
            }
            let filtered: Vec<_> = all.iter().filter(|s| {
                let k = serde_json::to_value(&s.kind).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
                let kind_ok = kind == "all" || kind == k;
                let hay = format!("{} {}", s.name, s.description).to_lowercase();
                let q_ok = needle.is_empty() || hay.contains(&needle);
                let tag_ok = tag.is_empty() || s.tags.iter().any(|t| t == tag);
                kind_ok && q_ok && tag_ok
            }).cloned().collect();
            let mut by_source: std::collections::BTreeMap<String, Vec<_>> = Default::default();
            for s in &filtered { by_source.entry(s.source.clone()).or_default().push(s.clone()); }
            let groups: Vec<_> = by_source.into_iter()
                .map(|(source, skills)| json!({ "source": source, "skills": skills }))
                .collect();
            Ok(json!({
                "items": all,
                "filtered": filtered,
                "groups": groups,
                "kind_counts": [
                    { "id": "all", "label": "All", "count": total, "selected": kind == "all" },
                    { "id": "smart", "label": "Smart", "count": smart, "selected": kind == "smart" },
                    { "id": "docs", "label": "Docs", "count": docs, "selected": kind == "docs" },
                ],
                "tags": tag_set,
                "total": total,
                "filtered_count": filtered.len(),
                "q": q,
                "active_kind": kind,
                "active_tag": tag,
            }))
        }
        "skills.file" => {
            let id = required_str(args, "id")?;
            let path = required_str(args, "path")?;
            let content = crate::views::state::load_skill_file(state.ship_dir(), id, path)
                .ok_or_else(|| RegistryError::not_found(format!("file `{path}` in skill `{id}`")))?;
            // HTML-escape: user-view env runs AutoEscape::None so any
            // <script>/<style>/<link> in user content would otherwise
            // leak into the document. Pre-escape at the service boundary.
            let escaped = html_escape(&content);
            let name = path.rsplit('/').next().unwrap_or(path).to_string();
            let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
            Ok(json!({
                "path": path,
                "name": name,
                "ext": ext,
                "content": escaped,
            }))
        }
        "skills.detail" => {
            let id = required_str(args, "id")?;
            let detail = crate::views::state::load_skill(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("skill `{id}`")))?;
            serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))
        }
        "skills.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let detail = crate::services::skills::state::load(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::skills::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))?)
        }

        // ── rules ─────────────────────────────────────────────────
        "rules.list" => Ok(json!({
            "items": crate::services::rules::state::list(&files),
        })),
        "rules.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, body) = crate::services::rules::state::load_body(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::rules::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::rules::state::ScopeTag::from(resolved),
                "body": body,
            }))
        }

        // ── mcp ───────────────────────────────────────────────────
        "mcp.list" => Ok(json!({
            "items": crate::services::mcp::state::list(&files),
        })),
        "mcp.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = crate::services::mcp::state::load_entry(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::mcp::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::mcp::state::ScopeTag::from(resolved),
                "entry": entry,
            }))
        }

        // ── permissions ───────────────────────────────────────────
        "permissions.list" => Ok(json!({
            "items": crate::services::permissions::state::list(&files),
        })),
        "permissions.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = crate::services::permissions::state::load_entry(&files, id, scope)
                .map_err(|e| match e {
                    crate::services::permissions::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "id": id,
                "scope": crate::services::permissions::state::ScopeTag::from(resolved),
                "entry": entry,
            }))
        }
        "permissions.tool_patterns" => {
            // Reassemble the composition the mod.rs handler does;
            // we can't call the axum handler directly from here.
            let mut out: Vec<serde_json::Value> = Vec::new();
            for t in &["Read", "Write", "Edit", "Glob", "Grep", "Bash",
                      "NotebookEdit", "WebFetch", "WebSearch", "Task"] {
                out.push(json!({ "pattern": t, "label": t, "source": "provider" }));
            }
            for (p, l) in &[
                ("Bash(git *)", "Bash git · all git subcommands"),
                ("Bash(git reset*)", "Bash git · reset (destructive)"),
                ("Bash(git push --force*)", "Bash git · force push (destructive)"),
                ("Bash(npm *)", "Bash npm"),
                ("Bash(pnpm *)", "Bash pnpm"),
                ("Bash(cargo *)", "Bash cargo"),
                ("Bash(ship *)", "Bash ship"),
                ("Bash(just *)", "Bash just"),
                ("Bash(rm -rf *)", "Bash rm -rf (destructive)"),
            ] {
                out.push(json!({ "pattern": p, "label": l, "source": "bash" }));
            }
            // Per-server `mcp__<server>__*` wildcards via McpService.
            let project_root = state.ship_dir().parent().map(Path::to_path_buf)
                .unwrap_or_else(|| state.ship_dir().to_path_buf());
            if let Ok(servers) = runtime::config::list_mcp_servers(Some(project_root)) {
                for server in servers {
                    let sid = if server.id.is_empty() { server.name.clone() } else { server.id };
                    out.push(json!({
                        "pattern": format!("mcp__{sid}__*"),
                        "label": format!("mcp · {sid} · all tools"),
                        "source": format!("mcp-{sid}"),
                    }));
                }
            }
            Ok(json!({ "patterns": out }))
        }

        // ── pipelines ─────────────────────────────────────────────
        "pipelines.list" => Ok(json!({
            "items": crate::services::pipelines::state::list(&files),
        })),
        "pipelines.get" => {
            let n = required_str(args, "name")?;
            let scope = parse_scope(args)?;
            let (resolved, ext, doc) = crate::services::pipelines::state::load_doc(&files, n, scope)
                .map_err(|e| match e {
                    crate::services::pipelines::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(json!({
                "name": n,
                "scope": crate::services::pipelines::state::ScopeTag::from(resolved),
                "format": ext,
                "doc": doc,
            }))
        }
        "pipelines.summaries" => {
            let filter = crate::views::state::Filter {
                q: args.get("q").and_then(|v| v.as_str()).map(String::from),
                agent: args.get("agent").and_then(|v| v.as_str()).map(String::from),
                pipeline: args.get("pipeline").and_then(|v| v.as_str()).map(String::from),
                status: args.get("status").and_then(|v| v.as_str()).map(String::from),
            };
            let tab = args.get("tab").and_then(|v| v.as_str()).map(String::from);
            let ctx = crate::services::pipelines::summaries::build(state.ship_dir(), filter, tab);
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "pipelines.focus" => {
            let n = required_str(args, "name")?;
            let ctx = crate::services::pipelines::focus::build(n, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("pipeline `{n}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "pipelines.run" => {
            let slug = required_str(args, "slug")?;
            let ctx = crate::services::pipelines::run_detail::build(slug, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("run `{slug}`")))?;
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "pipelines.history" => {
            let q = args.get("q").and_then(|v| v.as_str()).map(String::from);
            let pipeline = args.get("pipeline").and_then(|v| v.as_str()).map(String::from);
            let offset = optional_usize(args, "offset", 0);
            let limit = optional_usize(args, "limit", 50);
            let ctx = crate::services::pipelines::history::build(
                state.ship_dir(), q, pipeline, offset, limit,
            );
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }

        // ── events ────────────────────────────────────────────────
        "events.list" => {
            let limit = optional_usize(args, "limit", 200);
            let filter = crate::views::events_state::EventFilter {
                event_type: args.get("event_type").and_then(|v| v.as_str()).map(String::from),
                actor: args.get("actor").and_then(|v| v.as_str()).map(String::from),
                entity: args.get("entity").and_then(|v| v.as_str()).map(String::from),
                q: args.get("q").and_then(|v| v.as_str()).map(String::from),
            };
            let rows = crate::views::events_state::list_events(state.ship_dir(), &filter, limit);
            let types = crate::views::events_state::distinct_types(&rows);
            let actors = crate::views::events_state::distinct_actors(&rows);
            let groups = crate::views::events_state::group_by_entity(rows);
            Ok(json!({
                "groups": groups,
                "types": types,
                "actors": actors,
            }))
        }

        // ── specs ─────────────────────────────────────────────────
        "specs.list" => {
            let specs = runtime::db::specs::list_specs()
                .map_err(|e| RegistryError::service(e.to_string()))?;
            Ok(json!({ "items": specs }))
        }
        "specs.get" => {
            let id = required_str(args, "id")?;
            let spec = runtime::db::specs::get_spec(id)
                .map_err(|e| RegistryError::service(e.to_string()))?
                .ok_or_else(|| RegistryError::not_found(format!("spec `{id}`")))?;
            let body = runtime::db::specs::load_body(&spec.body_path)
                .unwrap_or_default();
            Ok(json!({ "spec": spec, "body": body }))
        }
        "specs.summaries" => {
            let q = args.get("q").and_then(|v| v.as_str()).unwrap_or("");
            let selected = args
                .get("selected")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let ctx = crate::services::specs::view_state::build_summaries(q, selected);
            serde_json::to_value(&ctx).map_err(|e| RegistryError::service(e.to_string()))
        }
        "specs.detail" => {
            let id = required_str(args, "id")?;
            let detail = crate::services::specs::view_state::build_detail(id)
                .ok_or_else(|| RegistryError::not_found(format!("spec `{id}`")))?;
            serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))
        }

        // ── settings ──────────────────────────────────────────────
        "settings.state" => {
            let ctx = crate::services::settings::view_state::build_ctx();
            serde_json::to_value(&ctx)
                .map_err(|e| RegistryError::service(e.to_string()))
        }

        // ── collections ───────────────────────────────────────────
        "collections.state" => {
            let ship_dir = state.ship_dir();
            let project_dir = runtime::project::get_project_dir(None).ok();
            let mut schemas = runtime::collections::builtin::user_schemas();
            schemas.extend(
                runtime::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
                    .unwrap_or_default(),
            );
            let active_id = args.get("active").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            // Same shape build_view_context emitted — field tuples stay
            // as [key, {field_type, required, values, default}] pairs so
            // templates iterate without dict-helpers.
            fn fields_tuples(s: &runtime::collections::schema::CollectionSchema) -> Vec<serde_json::Value> {
                s.fields.iter().map(|(k, v)| json!([k, {
                    "field_type": format!("{:?}", v.field_type).to_lowercase(),
                    "required": v.required,
                    "values": v.values,
                    "default": v.default,
                }])).collect()
            }
            let summaries: Vec<_> = schemas.iter().map(|s| json!({
                "id": s.id, "name": s.name,
                "field_count": s.fields.len(),
                "fields": fields_tuples(s),
            })).collect();
            let active_schema = active_id.as_deref()
                .and_then(|id| schemas.iter().find(|s| s.id == id))
                .or_else(|| schemas.first());
            let (schema, items, total_items, active_collection) = match active_schema {
                Some(first) => {
                    let mut items_out: Vec<serde_json::Value> = Vec::new();
                    if let Ok(mut svc) = runtime::collections::store::open_data_db(ship_dir) {
                        let _ = runtime::collections::store::ensure_collection_table(&mut svc, first);
                        let req = runtime::collections::query::QueryRequest {
                            collection: first.id.clone(),
                            filter: Default::default(),
                            sort: Default::default(),
                            limit: 200, offset: 0,
                        };
                        if let Ok(rows) = runtime::collections::store::query_items(&mut svc, first, &req) {
                            items_out = rows.into_iter()
                                .map(|i| serde_json::to_value(i).unwrap_or_default())
                                .collect();
                        }
                    }
                    let schema_val = json!({
                        "id": first.id, "name": first.name,
                        "fields": fields_tuples(first),
                    });
                    let total = items_out.len();
                    (schema_val, items_out, total, first.id.clone())
                }
                None => (json!(null), Vec::new(), 0, String::new()),
            };
            Ok(json!({
                "collections": summaries,
                "active_collection": active_collection,
                "schema": schema,
                "items": items,
                "total_items": total_items,
            }))
        }

        // ── sessions ──────────────────────────────────────────────
        "sessions.list" => {
            Ok(json!({
                "items": state.pty_sessions().list(),
            }))
        }

        // ── chat ──────────────────────────────────────────────────
        "chat.state" => {
            let ship_dir = state.ship_dir();
            let ws_arg = args.get("workspace_id").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let messages = crate::views::chat_state::list_messages(
                ship_dir, ws_arg, crate::views::chat_state::DEFAULT_LIMIT,
            );
            Ok(json!({
                "workspace_id": ws_arg.unwrap_or(""),
                "messages": messages.clone(),
                "total": messages.len(),
            }))
        }

        // ── dashboard ─────────────────────────────────────────────
        "dashboard.state" => {
            let ship_dir = state.ship_dir();
            // Jobs: split into active / history for stats + recent feed.
            let runs = crate::views::state::list_pipeline_runs(
                ship_dir,
                &crate::views::state::Filter::default(),
            );
            let (active, history) = crate::views::state::partition_runs(runs);
            let stats = json!({ "active": active.len(), "history": history.len() });
            let recent_runs: Vec<_> = if !active.is_empty() {
                active.iter().take(5).cloned().collect()
            } else {
                history.iter().take(5).cloned().collect()
            };
            // Events feed.
            let filter = crate::views::events_state::EventFilter::default();
            let events = crate::views::events_state::list_events(ship_dir, &filter, 10);
            let total_events = events.len();
            // Agents + collections lists.
            let agents = crate::views::state::list_agents(ship_dir);
            let project_dir = runtime::project::get_project_dir(None).ok();
            let mut schemas = runtime::collections::builtin::all_schemas();
            schemas.extend(
                runtime::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
                    .unwrap_or_default(),
            );
            let collections: Vec<_> = schemas.iter().map(|s| json!({
                "id": s.id, "name": s.name, "field_count": s.fields.len(),
            })).collect();
            Ok(json!({
                "stats": stats,
                "recent_runs": recent_runs,
                "events": events,
                "total_events": total_events,
                "agents": agents,
                "collections": collections,
            }))
        }

        // ── git ───────────────────────────────────────────────────
        "git.status" => {
            let root = crate::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            Ok(serde_json::to_value(crate::services::git::read::load_status_buckets(&root))
                .map_err(|e| RegistryError::service(e.to_string()))?)
        }
        "git.log" => {
            let root = crate::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", 50);
            Ok(json!({ "commits": crate::services::git::read::load_log(&root, None, limit) }))
        }
        "git.branches" => {
            let root = crate::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            let (ahead, behind) = crate::services::git::read::load_ahead_behind(&root)
                .unwrap_or((0, 0));
            Ok(json!({
                "branches": crate::services::git::read::load_branches(&root),
                "ahead": ahead,
                "behind": behind,
            }))
        }
        "git.overview" => {
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let root = crate::services::git::state::resolve_root(state.ship_dir(), workspace_id)
                .map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", crate::services::git::read::DEFAULT_LOG_LIMIT);
            let buckets = crate::services::git::read::load_status_buckets(&root);
            let (ahead, behind) = crate::services::git::read::load_ahead_behind(&root)
                .unwrap_or((0, 0));
            Ok(json!({
                "workspace_id": workspace_id,
                "branch": crate::services::git::read::load_branch(&root),
                "head_short": crate::services::git::read::load_head_short(&root),
                "clean": buckets.clean,
                "total_changed": buckets.total,
                "staged": buckets.staged,
                "modified": buckets.modified,
                "untracked": buckets.untracked,
                "commits": crate::services::git::read::load_log(&root, None, limit),
                "branches": crate::services::git::read::load_branches(&root),
                "ahead": ahead,
                "behind": behind,
            }))
        }
        "git.diff" => {
            let commit = required_str(args, "commit")?.trim();
            if commit.is_empty() {
                return Err(RegistryError::bad_args("commit must be non-empty"));
            }
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let root = crate::services::git::state::resolve_root(state.ship_dir(), workspace_id)
                .map_err(RegistryError::service)?;
            let is_working = commit.eq_ignore_ascii_case("WORKING");
            let diff_text = if is_working {
                crate::services::git::read::load_working_diff(&root)
            } else {
                let text = crate::services::git::read::load_commit_diff(&root, commit);
                if text.is_empty() {
                    return Err(RegistryError::not_found(format!("commit `{commit}`")));
                }
                text
            };
            let parsed = crate::services::git::diff::parse_unified_diff(&diff_text);
            let (sha, short_sha): (String, String) = if is_working {
                ("WORKING".into(), "working".into())
            } else {
                (commit.to_string(), commit.chars().take(7).collect())
            };
            Ok(json!({
                "sha": sha,
                "short_sha": short_sha,
                "is_working": is_working,
                "diff_text": diff_text,
                "parsed": parsed,
            }))
        }
        "git.annotations" => {
            let file = required_str(args, "file")?;
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let annotations = crate::services::git::annotations::load_file_annotations(
                state.ship_dir(),
                file,
                workspace_id,
            );
            Ok(serde_json::to_value(&annotations)
                .map_err(|e| RegistryError::service(e.to_string()))?)
        }

        other => Err(RegistryError::unknown_service(other)),
    }
}

/// Enumerate registered service names — derived from the rich op
/// descriptors in `services::schema`. Keep `services::registry::call`'s
/// match arms in sync with `schema::OPS`; drift is visible in diff.
pub fn names() -> Vec<&'static str> {
    super::schema::OPS.iter().map(|op| op.name).collect()
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
