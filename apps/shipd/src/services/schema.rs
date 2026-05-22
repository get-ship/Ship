#![allow(clippy::doc_overindented_list_items)]
//! `ship.*` op catalog — the single source of truth for what view
//! templates (`ship.<domain>.<op>`), `window.ship.*` codegen, and MCP
//! tool auto-registration all read.
//!
//! Served at `GET /api/v1/ship/schema`. Each op carries enough metadata
//! for lint, autocomplete, and permission mapping:
//!
//! - `name`         — dotted op path (e.g. `agents.detail`).
//! - `description`  — one-line human summary.
//! - `args`         — list of argument descriptors with kwarg name + kind.
//! - `returns`      — string tag naming the return shape.
//! - `permission`   — capability required to call. Metadata-only today;
//!                    enforcement lands with the caller-identity model.
//! - `sensitive`    — flag that lint / UI can warn on (payloads may
//!                    contain secrets, git content, etc.).
//!
//! Arg / return kinds are string tags today — keeps the surface light
//! (no JSON schema per op). Autocomplete + MCP tool bindings only need
//! name-shape matching; full schemas land when there's a concrete need.

use axum::Json;
use serde::Serialize;

use crate::wire::MeshResponse;

/// `GET /api/v1/ship/schema` — catalog of every `ship.<domain>.<op>` the
/// registry exposes. Consumers:
/// - `@ship/shell` TS SDK: codegen typed `window.ship.*` methods
/// - Lint: flag template calls that don't match a known op
/// - MCP server: auto-register ops as tools
/// - Third-party view authors: editor autocomplete via `/api/v1/ship/schema`
#[utoipa::path(
    operation_id = "serveShipSchema",
    get,
    path = "/api/v1/ship/schema",
    responses(
        (status = 200, description = "ship.* op catalog (names, args, returns, permissions)"),
    ),
    tag = "framework"
)]
pub async fn serve_schema() -> Json<MeshResponse> {
    Json(MeshResponse {
        ok: true,
        data: serde_json::json!({ "ops": OPS }),
    })
}

#[derive(Serialize, Clone, Copy)]
pub struct OpDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    pub returns: &'static str,
    pub permission: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub sensitive: bool,
}

#[derive(Serialize, Clone, Copy)]
pub struct ArgDescriptor {
    pub name: &'static str,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub required: bool,
}

fn is_false(v: &bool) -> bool { !v }

const PROJECT_SCOPE: ArgDescriptor = ArgDescriptor { name: "scope", kind: "string?", required: false };

/// A mutation op. The write-side analog of `OpDescriptor`: same shape minus
/// `returns` (commands return a status JSON), plus `blast_radius` for the gate
/// + the generated proto policy. `services::command::call` dispatches these;
/// `cargo xtask gen-ops` projects them onto `CommandOpsService`. One source for
/// the write surface, exactly as `OPS` is for reads.
#[derive(Serialize, Clone, Copy)]
pub struct CommandDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgDescriptor],
    pub permission: &'static str,
    /// Gate class: "C1_WORKSPACE" | "C2_ELEVATED".
    pub blast_radius: &'static str,
    #[serde(skip_serializing_if = "is_false")]
    pub sensitive: bool,
}

/// The mutation catalog. Order mirrors `services::command::call`'s dispatch.
/// Grows as route-level command logic is lifted into the command layer (#78).
pub const COMMANDS: &[CommandDescriptor] = &[
    // ── pipeline lifecycle ────────────────────────────────────────
    CommandDescriptor {
        name: "pipeline.update_status",
        description: "Advance a pipeline run's lifecycle/gate state (the agent's own run).",
        args: &[
            ArgDescriptor { name: "pipeline_run_id", kind: "string", required: true },
            ArgDescriptor { name: "status", kind: "string", required: true },
            ArgDescriptor { name: "worktree", kind: "string?", required: false },
            ArgDescriptor { name: "pid", kind: "number?", required: false },
            ArgDescriptor { name: "gate_agent", kind: "string?", required: false },
            ArgDescriptor { name: "blocker", kind: "string?", required: false },
            ArgDescriptor { name: "error", kind: "string?", required: false },
        ],
        permission: "ship.runtime.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "pipeline.step_done",
        description: "Agent declares the current pipeline step complete.",
        args: &[
            ArgDescriptor { name: "slug", kind: "string", required: true },
            ArgDescriptor { name: "summary", kind: "string?", required: false },
            ArgDescriptor { name: "outputs", kind: "map", required: false },
        ],
        permission: "ship.runtime.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    // ── project ───────────────────────────────────────────────────
    CommandDescriptor {
        name: "project.activate",
        description: "Switch the daemon's active project.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        permission: "ship.projects.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    // ── mesh ──────────────────────────────────────────────────────
    CommandDescriptor {
        name: "mesh.send",
        description: "Send a directed mesh message to another actor (caller-as-self).",
        args: &[
            ArgDescriptor { name: "from", kind: "string", required: true },
            ArgDescriptor { name: "to", kind: "string", required: true },
            ArgDescriptor { name: "body_json", kind: "string?", required: false },
        ],
        permission: "ship.mesh.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "mesh.broadcast",
        description: "Broadcast a mesh message (optional capability filter; caller-as-self).",
        args: &[
            ArgDescriptor { name: "from", kind: "string", required: true },
            ArgDescriptor { name: "body_json", kind: "string?", required: false },
            ArgDescriptor { name: "capability_filter", kind: "string?", required: false },
        ],
        permission: "ship.mesh.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    // ── git (caller's worktree) ───────────────────────────────────
    CommandDescriptor {
        name: "git.push",
        description: "Push the caller's worktree to a remote.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "remote", kind: "string?", required: false },
            ArgDescriptor { name: "force", kind: "bool?", required: false },
        ],
        permission: "ship.git.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "git.fetch",
        description: "Fetch a remote in the caller's worktree.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "remote", kind: "string?", required: false },
        ],
        permission: "ship.git.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "git.pull",
        description: "Pull (ff-only) in the caller's worktree.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "remote", kind: "string?", required: false },
        ],
        permission: "ship.git.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "git.stage",
        description: "Stage paths in the caller's worktree.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "paths", kind: "string[]", required: true },
        ],
        permission: "ship.git.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "git.unstage",
        description: "Unstage paths in the caller's worktree.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "paths", kind: "string[]", required: true },
        ],
        permission: "ship.git.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "git.commit",
        description: "Commit staged changes in the caller's worktree.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "message", kind: "string", required: true },
            ArgDescriptor { name: "author", kind: "string?", required: false },
        ],
        permission: "ship.git.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    // ── agent control ─────────────────────────────────────────────
    CommandDescriptor {
        name: "agent.stop",
        description: "Stop a running agent by slug.",
        args: &[ArgDescriptor { name: "slug", kind: "string", required: true }],
        permission: "ship.agents.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
    CommandDescriptor {
        name: "agent.steer",
        description: "Inject a message into a running agent.",
        args: &[
            ArgDescriptor { name: "slug", kind: "string", required: true },
            ArgDescriptor { name: "message", kind: "string", required: true },
        ],
        permission: "ship.agents.write",
        blast_radius: "C1_WORKSPACE",
        sensitive: false,
    },
];

/// The full registry op catalog. Order mirrors `services::registry::call`'s
/// match arms so drift between the two is visible in diff.
pub const OPS: &[OpDescriptor] = &[
    // ── agents ────────────────────────────────────────────────────
    OpDescriptor {
        name: "agents.list",
        description: "List all agents as minimal IndexEntry rows.",
        args: &[],
        returns: "{items: [IndexEntry]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.summaries",
        description: "List all agents with display metadata (label, harnesses, tags).",
        args: &[],
        returns: "{items: [AgentSummary]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.detail",
        description: "Full view-shaped agent detail — catalogs, permissions, harnesses, tags.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "AgentDetail",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.catalogs",
        description: "Catalog rows for the agents list (harnesses, skills, etc.).",
        args: &[],
        returns: "AgentCatalogs",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.get",
        description: "Raw agent doc from disk at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, doc}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.wire",
        description: "Graph payload (nodes + edges) for the agent-wire canvas.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "AgentWireCtx",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.settings",
        description: "Tier list + catalog rows for the per-agent settings pane.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "AgentSettingsCtx",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.templates",
        description: "Starter templates for the agent-create wizard.",
        args: &[],
        returns: "{items: [AgentTemplate]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.permission_presets",
        description: "Permission presets for the agent-create wizard.",
        args: &[],
        returns: "{items: [PermissionPreset]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.harnesses",
        description: "Enumerate known harnesses (claude, codex, cursor, ...).",
        args: &[],
        returns: "{items: [string]}",
        permission: "ship.agents.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "agents.create_ctx",
        description: "Convenience: templates + presets + harnesses for the create view.",
        args: &[],
        returns: "AgentsCreateCtx",
        permission: "ship.agents.read",
        sensitive: false,
    },

    // ── skills ────────────────────────────────────────────────────
    OpDescriptor {
        name: "skills.list",
        description: "List all skills as minimal IndexEntry rows.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.summaries",
        description: "Rail shape: filtered + grouped + kind counts + tags.",
        args: &[
            ArgDescriptor { name: "q", kind: "string?", required: false },
            ArgDescriptor { name: "kind", kind: "string?", required: false },
            ArgDescriptor { name: "tag", kind: "string?", required: false },
        ],
        returns: "SkillsRailCtx",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.detail",
        description: "Full view-shaped skill detail — authors, variables, files.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "SkillDetail",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.file",
        description: "HTML-escaped file content within a skill dir.",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "path", kind: "string", required: true },
        ],
        returns: "{path, name, ext, content}",
        permission: "ship.skills.read", sensitive: false,
    },
    OpDescriptor {
        name: "skills.get",
        description: "Raw skill detail at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "SkillDetail",
        permission: "ship.skills.read", sensitive: false,
    },

    // ── rules ─────────────────────────────────────────────────────
    OpDescriptor {
        name: "rules.list",
        description: "List markdown rules across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.rules.read", sensitive: false,
    },
    OpDescriptor {
        name: "rules.get",
        description: "Raw rule body at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, body}",
        permission: "ship.rules.read", sensitive: false,
    },

    // ── mcp ───────────────────────────────────────────────────────
    OpDescriptor {
        name: "mcp.list",
        description: "List MCP server entries across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.mcp.read", sensitive: false,
    },
    OpDescriptor {
        name: "mcp.get",
        description: "Raw MCP entry at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, entry}",
        permission: "ship.mcp.read", sensitive: false,
    },

    // ── permissions ──────────────────────────────────────────────
    OpDescriptor {
        name: "permissions.list",
        description: "List permission presets across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.permissions.read", sensitive: false,
    },
    OpDescriptor {
        name: "permissions.get",
        description: "Raw permission preset at the requested scope.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{id, scope, entry}",
        permission: "ship.permissions.read", sensitive: false,
    },
    OpDescriptor {
        name: "permissions.tool_patterns",
        description: "Enumerate tool-pattern capability names (for agent builders).",
        args: &[], returns: "{patterns: [ToolPattern]}",
        permission: "ship.permissions.read", sensitive: false,
    },

    // ── pipelines ─────────────────────────────────────────────────
    OpDescriptor {
        name: "pipelines.list",
        description: "List pipeline definitions across scopes.",
        args: &[], returns: "{items: [IndexEntry]}",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.get",
        description: "Raw pipeline doc at the requested scope.",
        args: &[ArgDescriptor { name: "name", kind: "string", required: true }, PROJECT_SCOPE],
        returns: "{name, scope, format, doc}",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.summaries",
        description: "Index-view shape: defs + active/history + pills + agents.",
        args: &[], returns: "PipelinesSummariesCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.focus",
        description: "Per-pipeline detail with active + other runs precomputed.",
        args: &[ArgDescriptor { name: "name", kind: "string", required: true }],
        returns: "PipelineFocusCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.run",
        description: "Run detail with steps + timeline.",
        args: &[ArgDescriptor { name: "slug", kind: "string", required: true }],
        returns: "RunDetailCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },
    OpDescriptor {
        name: "pipelines.history",
        description: "Paginated run history with pipeline filter options.",
        args: &[
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "offset", kind: "number?", required: false },
            ArgDescriptor { name: "pipeline", kind: "string?", required: false },
        ],
        returns: "HistoryCtx",
        permission: "ship.pipelines.read", sensitive: false,
    },

    // ── events ────────────────────────────────────────────────────
    OpDescriptor {
        name: "events.list",
        description: "Kernel events with filters, grouped by entity.",
        args: &[
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "event_type", kind: "string?", required: false },
            ArgDescriptor { name: "actor", kind: "string?", required: false },
            ArgDescriptor { name: "entity", kind: "string?", required: false },
            ArgDescriptor { name: "q", kind: "string?", required: false },
        ],
        returns: "{groups, types, actors}",
        permission: "ship.events.read", sensitive: true,
    },

    // ── specs ─────────────────────────────────────────────────────
    OpDescriptor {
        name: "specs.list",
        description: "Raw spec records from the DB.",
        args: &[], returns: "{items: [Spec]}",
        permission: "ship.specs.read", sensitive: false,
    },
    OpDescriptor {
        name: "specs.get",
        description: "Full spec record + body.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "{spec, body}",
        permission: "ship.specs.read", sensitive: false,
    },
    OpDescriptor {
        name: "specs.summaries",
        description: "Rail shape: grouped + counts + pipelines.",
        args: &[
            ArgDescriptor { name: "q", kind: "string?", required: false },
            ArgDescriptor { name: "selected", kind: "string?", required: false },
        ],
        returns: "SpecsSummariesCtx",
        permission: "ship.specs.read", sensitive: false,
    },
    OpDescriptor {
        name: "specs.detail",
        description: "Full view-shaped spec detail: parsed sections, launch ctx.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "SpecDetail",
        permission: "ship.specs.read", sensitive: false,
    },

    // ── git ───────────────────────────────────────────────────────
    OpDescriptor {
        name: "git.status",
        description: "Working tree + index status buckets.",
        args: &[], returns: "GitStatusBuckets",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.log",
        description: "Commit log up to `limit` entries.",
        args: &[ArgDescriptor { name: "limit", kind: "number?", required: false }],
        returns: "{commits: [CommitSummary]}",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.branches",
        description: "Local + remote branches with ahead/behind vs upstream.",
        args: &[], returns: "{branches, ahead, behind}",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.overview",
        description: "One-call shape for the git view's default route.",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "limit", kind: "number?", required: false },
        ],
        returns: "GitCtx",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.diff",
        description: "Diff for a commit or 'WORKING'.",
        args: &[
            ArgDescriptor { name: "commit", kind: "string", required: true },
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
        ],
        returns: "DiffView",
        permission: "ship.git.read", sensitive: true,
    },
    OpDescriptor {
        name: "git.annotations",
        description: "AnnotationsByLine for the active diff file.",
        args: &[
            ArgDescriptor { name: "file", kind: "string", required: true },
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
        ],
        returns: "AnnotationsByLine",
        permission: "ship.git.read", sensitive: false,
    },

    // ── settings / dashboard / collections / chat ────────────────
    OpDescriptor {
        name: "settings.state",
        description: "Harnesses, defaults, terminal, about — everything the settings view renders.",
        args: &[], returns: "SettingsCtx",
        permission: "ship.settings.read", sensitive: false,
    },
    OpDescriptor {
        name: "dashboard.state",
        description: "Aggregate stats + recent runs + events + agents + collections.",
        args: &[], returns: "DashboardCtx",
        permission: "ship.dashboard.read", sensitive: false,
    },
    OpDescriptor {
        name: "collections.state",
        description: "Collections list + active schema + items (tier-3 app data).",
        args: &[ArgDescriptor { name: "active", kind: "string?", required: false }],
        returns: "CollectionsCtx",
        permission: "ship.collections.read", sensitive: false,
    },
    OpDescriptor {
        name: "collections.list",
        description: "Collection summaries with field records (id, name, fields[name,type,required]).",
        args: &[],
        returns: "{items: [CollectionSummary]}",
        permission: "ship.collections.read", sensitive: false,
    },
    OpDescriptor {
        name: "collections.detail",
        description: "Single collection: schema + items + total count.",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        returns: "{schema, items, total_items}",
        permission: "ship.collections.read", sensitive: false,
    },
    OpDescriptor {
        name: "collections.items",
        description: "Rows for a single collection.",
        args: &[ArgDescriptor { name: "collection", kind: "string", required: true }],
        returns: "{items, total_items}",
        permission: "ship.collections.read", sensitive: false,
    },
    OpDescriptor {
        name: "artifacts.list",
        description: "Artifact metadata records, filterable by scope/mime/collection/item/owner.",
        args: &[
            ArgDescriptor { name: "scope", kind: "string?", required: false },
            ArgDescriptor { name: "mime", kind: "string?", required: false },
            ArgDescriptor { name: "collection_id", kind: "string?", required: false },
            ArgDescriptor { name: "item_id", kind: "string?", required: false },
            ArgDescriptor { name: "owner_actor", kind: "string?", required: false },
            ArgDescriptor { name: "content_hash", kind: "string?", required: false },
            ArgDescriptor { name: "sort", kind: "string?", required: false },
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "offset", kind: "number?", required: false },
        ],
        returns: "{items: [ArtifactRecord], count, scope}",
        permission: "ship.artifacts.read", sensitive: false,
    },
    OpDescriptor {
        name: "chat.state",
        description: "Recent messages for the chat drawer, scoped by workspace.",
        args: &[ArgDescriptor { name: "workspace_id", kind: "string?", required: false }],
        returns: "{workspace_id, messages, total}",
        permission: "ship.chat.read", sensitive: true,
    },
    OpDescriptor {
        name: "sessions.list",
        description: "All live PTY sessions (agent chats, pipeline runs, shells) — Mission Control data.",
        args: &[],
        returns: "{items: [SessionMeta]}",
        permission: "ship.sessions.read", sensitive: false,
    },
    // ── workspaces ────────────────────────────────────────────────
    OpDescriptor {
        name: "workspaces.list",
        description: "Known project workspaces with active flag per row.",
        args: &[],
        returns: "{items: [WorkspaceSummary], active: string?}",
        permission: "ship.workspaces.read",
        sensitive: false,
    },
    OpDescriptor {
        name: "workspaces.active",
        description: "Currently-active workspace id (null if none).",
        args: &[],
        returns: "{id: string?}",
        permission: "ship.workspaces.read",
        sensitive: false,
    },

    // ── account ──────────────────────────────────────────────────
    OpDescriptor {
        name: "account.state",
        description: "Ship Cloud account snapshot — logged_out / pending / logged_in.",
        args: &[],
        returns: "AccountState",
        permission: "ship.account.read",
        sensitive: false,
    },
];
